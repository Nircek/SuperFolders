use crate::app::App;
use ratatui::{
    Frame,
    layout::{Constraint, Layout},
    style::{Color, Modifier, Style, Stylize},
    widgets::{Block, Borders, Row, Table, TableState},
};
/// Renders the TUI interface

pub fn draw(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .constraints([Constraint::Percentage(100)])
        .split(f.area());

    let header_cells = ["Min Data", "Max Data", "Ilość", "Ścieżka"]
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

        let path_utils = if item.is_superfolder {
            format!("{} [SF]", item.path)
        } else {
            item.path.clone()
        };

        let cells = vec![
            item.min_date.clone(),
            item.max_date.clone(),
            item.count_str.clone(),
            path_utils,
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

    // Maintain a local TableState for automatic scrolling
    let mut state = TableState::default();
    state.select(Some(app.selected_index));

    f.render_stateful_widget(t, chunks[0], &mut state);
}
