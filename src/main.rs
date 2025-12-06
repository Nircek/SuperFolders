use anyhow::Result;
use clap::Parser;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};
use std::io::{self, Write};
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;
use std::time::{Duration, Instant};

mod app;
mod config;
mod scanner;
mod ui;

use app::App;

/// Command line arguments
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to the root folder to scan
    #[arg(default_value = ".")]
    path: PathBuf,
}

fn main() -> Result<()> {
    let args = Args::parse();
    // Helper to get absolute path, assuming it exists
    let root_path = if args.path.exists() {
        args.path.canonicalize()?
    } else {
        return Err(anyhow::anyhow!("Path does not exist: {:?}", args.path));
    };

    // Show progress message during initial scan
    print!("Scanning directory tree...");
    io::stdout().flush()?;

    let counter = Arc::new(AtomicUsize::new(0));
    let counter_clone = Arc::clone(&counter);
    let root_path_clone = root_path.clone();

    // Start scanning in a separate thread
    let scan_handle =
        thread::spawn(move || App::new_with_progress(root_path_clone, Some(counter_clone)));

    // Show progress updates
    let start_time = Instant::now();
    while !scan_handle.is_finished() {
        let count = counter.load(Ordering::Relaxed);
        let elapsed = start_time.elapsed().as_secs();
        print!(
            "\rScanning directory tree... {} items found ({} seconds)",
            count, elapsed
        );
        io::stdout().flush()?;
        thread::sleep(Duration::from_millis(100));
    }

    let mut app = scan_handle
        .join()
        .map_err(|_| anyhow::anyhow!("Scan thread panicked"))?;
    let final_count = counter.load(Ordering::Relaxed);
    println!(
        "\rScan complete! Found {} items.                    ",
        final_count
    );
    thread::sleep(Duration::from_millis(500)); // Brief pause to see the message

    // Setup Terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let res = run_app(&mut terminal, &mut app);

    // Restore Terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err);
    }

    Ok(())
}

fn run_app(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, app: &mut App) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui::draw(f, app))?;

        if event::poll(Duration::from_millis(250))?
            && let Event::Key(key) = event::read()?
        {
            match key.code {
                KeyCode::Char('q') => return Ok(()),
                KeyCode::Esc => {
                    if app.show_help {
                        app.toggle_help();
                    } else {
                        return Ok(());
                    }
                }
                KeyCode::Down => app.next(),
                KeyCode::Up => app.previous(),
                KeyCode::Left => app.collapse_current()?,
                KeyCode::Right => app.expand_current()?,
                KeyCode::Char('e') | KeyCode::Char('E') => app.export_to_csv()?,
                KeyCode::Char('h') | KeyCode::Char('H') | KeyCode::Char('?') => app.toggle_help(),
                _ => {}
            }
        }
    }
}
