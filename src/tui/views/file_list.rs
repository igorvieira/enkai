use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

use crate::app::AppState;

pub fn render_file_list(frame: &mut Frame, state: &AppState, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Header
            Constraint::Min(0),     // File list
            Constraint::Length(3),  // Footer
        ])
        .split(area);

    // Header
    let header = Paragraph::new(vec![
        Line::from(vec![
            Span::styled("Enkai", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::raw(" - Git Conflict Resolution Tool"),
        ]),
        Line::from(format!("Operation: {}", state.git_operation.as_str())),
    ])
    .block(Block::default().borders(Borders::ALL))
    .alignment(Alignment::Center);

    frame.render_widget(header, chunks[0]);

    // File list
    let items: Vec<ListItem> = state
        .files
        .iter()
        .enumerate()
        .map(|(i, file)| {
            let status_icon = if file.is_fully_resolved() {
                "✓"
            } else {
                "✗"
            };

            let status_color = if file.is_fully_resolved() {
                Color::Green
            } else {
                Color::Red
            };

            let conflict_info = format!(
                "{}/{} resolved",
                file.resolved_count(),
                file.total_conflicts()
            );

            let line = Line::from(vec![
                Span::styled(
                    format!(" {} ", status_icon),
                    Style::default().fg(status_color).add_modifier(Modifier::BOLD),
                ),
                Span::raw(file.path_string()),
                Span::raw("  "),
                Span::styled(
                    format!("({})", conflict_info),
                    Style::default().fg(Color::DarkGray),
                ),
            ]);

            let style = if i == state.selected_file {
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            ListItem::new(line).style(style)
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title("Conflicted Files (j/k or ↑/↓ to navigate, Enter to open, q to quit)"),
    );

    frame.render_widget(list, chunks[1]);

    // Footer
    let footer_text = if state.all_files_resolved() {
        if state.git_operation.is_rebase() {
            "All conflicts resolved! Press Enter to continue with rebase actions."
        } else {
            "All conflicts resolved! You can now commit your changes."
        }
    } else {
        "Select a file to resolve its conflicts"
    };

    let footer = Paragraph::new(footer_text)
        .block(Block::default().borders(Borders::ALL))
        .alignment(Alignment::Center)
        .style(
            Style::default()
                .fg(if state.all_files_resolved() {
                    Color::Green
                } else {
                    Color::Yellow
                })
                .add_modifier(Modifier::BOLD),
        );

    frame.render_widget(footer, chunks[2]);
}
