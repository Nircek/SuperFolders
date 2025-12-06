use crate::app::App;
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    widgets::{Block, Borders, Clear, Paragraph, Row, Table, Wrap},
};

/// Renders the TUI interface
pub fn draw(f: &mut Frame, app: &mut App) {
    if app.show_help {
        draw_help_overlay(f, app);
        return;
    }

    let chunks = Layout::vertical([
        Constraint::Min(3),    // Main table
        Constraint::Length(1), // Status bar
    ])
    .split(f.area());

    let header_cells = ["Min Date", "Max Date", "Count", "Path"]
        .iter()
        .map(|h| ratatui::widgets::Cell::from(h.bold()));
    let header = Row::new(header_cells)
        .style(Style::default().add_modifier(Modifier::BOLD))
        .height(1)
        .bottom_margin(1);

    let rows = app.view_items.iter().enumerate().map(|(i, item)| {
        let style = if i == app.selected_index {
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else if item.is_superfolder {
            Style::default().fg(Color::Cyan) // Highlight superfolders
        } else {
            Style::default()
        };

        let path_display = if item.is_superfolder {
            if item.is_atomic {
                format!("📦 {}", item.path)
            } else {
                format!("📦 {} [SF]", item.path)
            }
        } else {
            format!("  {}", item.path)
        };

        let cells = vec![
            item.min_date.clone(),
            item.max_date.clone(),
            item.count_str.clone(),
            path_display,
        ];

        Row::new(cells).style(style).height(1)
    });

    let widths = [
        Constraint::Length(12),
        Constraint::Length(12),
        Constraint::Length(8),
        Constraint::Min(20),
    ];

    let t = Table::new(rows, widths).header(header).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" SuperFolders "),
    );

    // Use the persistent TableState from App for proper scrolling
    f.render_stateful_widget(t, chunks[0], &mut app.table_state);

    // Draw status bar
    let status_text = if let Some(msg) = app.get_status() {
        msg.to_string()
    } else {
        format!("Total entries: {}", app.view_items.len())
    };
    let status_bar = Paragraph::new(status_text).style(Style::default().fg(Color::Gray));
    f.render_widget(status_bar, chunks[1]);
}

fn draw_help_overlay(f: &mut Frame, app: &App) {
    let area = centered_rect(85, 85, f.area());

    // Build system folders list dynamically from config
    let system_folders_list = app.config.system_folders.join(", ");

    let help_text = format!(
        r#"╔═══════════════════════════════════════════════════════════╗
║                   SuperFolders - Help                     ║
╚═══════════════════════════════════════════════════════════╝

KEY CONCEPTS:

• Superfolder: A folder that aggregates all its content as a single unit, showing only statistics (file count, dates).

• Atomic Superfolder: Automatically detected project folders containing system folders. Marked with 📦

• User Superfolder: Any folder you manually collapse. Marked with 📦 [SF]

• Tree Frontier: The current flat view showing the highest level of detail you've chosen to see.

CONTROLS:

  ↑/↓        Navigate up/down in the list
  ←          Collapse parent folder into a User Superfolder
  →          Expand selected Superfolder (User only, not Atomic)
  E          Export current view to CSV file
  H or ?     Show/hide this help screen
  Q or Esc   Quit the application

SYSTEM FOLDERS (Atomic Superfolder Detection):
  {}

CONFIGURATION:
  Set SUPERFOLDERS_SYSTEM_FOLDERS environment variable (comma-separated) or create superfolders.toml next to binary or ~/.superfolders.toml in home directory.

Press H, ? or Esc to close this help."#,
        system_folders_list
    );

    let paragraph = Paragraph::new(help_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan))
                .title(" Help ")
                .title_alignment(Alignment::Center),
        )
        .style(Style::default().bg(Color::Black).fg(Color::White))
        .wrap(Wrap { trim: true })
        .scroll((0, 0));

    f.render_widget(Clear, area);
    f.render_widget(paragraph, area);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::vertical([
        Constraint::Percentage((100 - percent_y) / 2),
        Constraint::Percentage(percent_y),
        Constraint::Percentage((100 - percent_y) / 2),
    ])
    .split(r);

    Layout::horizontal([
        Constraint::Percentage((100 - percent_x) / 2),
        Constraint::Percentage(percent_x),
        Constraint::Percentage((100 - percent_x) / 2),
    ])
    .split(popup_layout[1])[1]
}
