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

    // --- Virtualization / Manual Windowing Logic ---
    let table_area = chunks[0];
    // Reserve lines for header (1), status bar (1), and borders (2)
    let available_height = table_area.height.saturating_sub(4) as usize;

    // Update App's page height for navigation consistency
    app.page_height = available_height as u16;

    // Ensure selected_index is within view
    let mut offset = app.table_state.offset();
    if app.selected_index >= offset + available_height {
        offset = app.selected_index + 1 - available_height;
    } else if app.selected_index < offset {
        offset = app.selected_index;
    }
    // Persist the corrected offset back to state
    *app.table_state.offset_mut() = offset;

    let start_index = offset;
    let end_index = (start_index + available_height).min(app.view_items.len());

    let rows = app.view_items[start_index..end_index]
        .iter()
        .enumerate()
        .map(|(i, item)| {
            // The absolute index in the original list
            let absolute_index = start_index + i;
            let style = if absolute_index == app.selected_index {
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

    // Create a temporary state for the sliced table
    // The slice is 0-indexed relative to itself, so we adjust selection
    let relative_selected = app.selected_index.saturating_sub(start_index);
    let mut temp_state = ratatui::widgets::TableState::default();
    temp_state.select(Some(relative_selected));

    f.render_stateful_widget(t, table_area, &mut temp_state);

    // Draw status bar
    let status_text = if let Some(msg) = app.get_status() {
        msg.to_string()
    } else {
        format!("Total entries: {}", app.view_items.len())
    };
    let status_bar = Paragraph::new(status_text).style(Style::default().fg(Color::Gray));
    f.render_widget(status_bar, chunks[1]);
}

fn draw_help_overlay(f: &mut Frame, app: &mut App) {
    let area = centered_rect(85, 85, f.area());

    // Update help height (subtract 2 for borders)
    app.help_height = area.height.saturating_sub(2);

    // Build system folders list dynamically from config
    let system_folders_list = app.config.system_folders.join(", ");

    let help_text = format!(
        r#"╔═══════════════════════════════════════════════════════════╗
║                   SuperFolders - Help                     ║
╚═══════════════════════════════════════════════════════════╝

KEY CONCEPTS:

• Item: A file or directory.
• Entry: A row in this table (File, Folder, or Superfolder).
• Count: Item count for the entry.
  - File/Folder: 1.
  - Superfolder: Recursive count of contained items.

• Superfolder: A folder aggregated as a single unit.
  - Atomic: System folder wrapper (📦).
  - User: Manually collapsed (📦 [SF]).

• Tree Frontier: The current flat view.
  - Files: No trailing slash.
  - Folders: With trailing slash (e.g. src/).

CONTROLS:

  ↑/↓        Navigate up/down in the list
  ←          Collapse parent folder into a User Superfolder
  →          Expand selected Superfolder (User only, not Atomic)
  E          Export current view to CSV file
  H or ?     Show/hide this help screen
  Q or Esc   Quit the application
  PageUp/Down Use dynamic page scrolling

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
        .wrap(Wrap { trim: true })
        .scroll((app.help_scroll, 0));

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::{App, ViewItem};
    use crate::config::Config;
    use crate::scanner::{EntryKind, FsEntry, FsStats};
    use ratatui::{Terminal, backend::TestBackend};
    use std::path::PathBuf;
    use std::time::Instant;

    #[test]
    fn benchmark_rendering_large_list() {
        // 1. Setup App with 100k items
        let mut view_items = Vec::with_capacity(100_000);
        for i in 0..100_000 {
            view_items.push(ViewItem {
                path: format!("item_{}", i),
                full_path: PathBuf::from(format!("/tmp/item_{}", i)),
                min_date: "2023-01-01".to_string(),
                max_date: "2023-12-31".to_string(),
                count_str: "123".to_string(),
                is_superfolder: i % 10 == 0,
                is_atomic: i % 20 == 0,
                is_selected: false,
            });
        }

        let mut app = App {
            root_path: PathBuf::from("."),
            root: FsEntry {
                path: PathBuf::from("."),
                name: "root".to_string(),
                kind: EntryKind::Directory,
                stats: FsStats::new(),
                children: vec![],
                is_user_superfolder: false,
            },
            view_items,
            selected_index: 50_000,
            table_state: ratatui::widgets::TableState::default(),
            config: Config::new(),
            show_help: false,
            status_message: None,
            help_scroll: 0,
            page_height: 15,
            help_height: 15,
        };

        // 2. Setup Test Backend
        let backend = TestBackend::new(100, 50);
        let mut terminal = Terminal::new(backend).unwrap();

        // 3. Measure Draw Time
        let start = Instant::now();
        terminal
            .draw(|f| {
                draw(f, &mut app);
            })
            .unwrap();
        let duration = start.elapsed();

        assert!(
            duration.as_secs_f64() < 0.1,
            "Rendering took too long: {:?}",
            duration
        );
    }
}
