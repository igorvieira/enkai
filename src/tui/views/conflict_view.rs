use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

use crate::app::AppState;
use crate::domain::Resolution;

pub fn render_conflict_view(frame: &mut Frame, state: &AppState, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Header
            Constraint::Min(0),     // Content
            Constraint::Length(5),  // Footer
        ])
        .split(area);

    // Get current file and conflict
    let (file, conflict_index) = match state.current_file() {
        Some(file) => (file, state.current_conflict_index().unwrap_or(0)),
        None => return,
    };

    if conflict_index >= file.conflicts.len() {
        return;
    }

    let conflict = &file.conflicts[conflict_index];
    let resolution = file.resolutions[conflict_index];

    // Header
    let header = Paragraph::new(vec![
        Line::from(vec![
            Span::styled("File: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(file.path_string()),
        ]),
        Line::from(format!(
            "Conflict {}/{} | Lines {}-{}",
            conflict_index + 1,
            file.total_conflicts(),
            conflict.start_line + 1,
            conflict.end_line + 1
        )),
    ])
    .block(Block::default().borders(Borders::ALL))
    .alignment(Alignment::Left);

    frame.render_widget(header, chunks[0]);

    // Content - show current and incoming side by side
    let content_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(50), // Current
            Constraint::Percentage(50), // Incoming
        ])
        .split(chunks[1]);

    // Current content
    let current_selected = matches!(resolution, Some(Resolution::Current));
    let current_style = if current_selected {
        Style::default()
            .fg(Color::Green)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };

    let current_title = if current_selected {
        "CURRENT (HEAD) ✓ [selected]"
    } else {
        "CURRENT (HEAD) - Press 'c' to select"
    };

    let current = Paragraph::new(conflict.current.as_str())
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(current_title)
                .style(current_style),
        )
        .wrap(Wrap { trim: false });

    frame.render_widget(current, content_chunks[0]);

    // Incoming content
    let incoming_selected = matches!(resolution, Some(Resolution::Incoming));
    let incoming_style = if incoming_selected {
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };

    let incoming_title = if incoming_selected {
        "INCOMING ✓ [selected]"
    } else {
        "INCOMING - Press 'i' to select"
    };

    let incoming = Paragraph::new(conflict.incoming.as_str())
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(incoming_title)
                .style(incoming_style),
        )
        .wrap(Wrap { trim: false });

    frame.render_widget(incoming, content_chunks[1]);

    // Footer - keybindings and status
    let both_selected = matches!(resolution, Some(Resolution::Both));
    let both_text = if both_selected {
        "=Both ✓".to_string()
    } else {
        "=Both".to_string()
    };

    let can_save = file.is_fully_resolved();
    let save_style = if can_save {
        Style::default().fg(Color::Green)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let save_text = format!(
        "=Save ({}/{} resolved) | ",
        file.resolved_count(),
        file.total_conflicts()
    );

    let footer = Paragraph::new(vec![
        Line::from(vec![
            Span::raw("Keys: "),
            Span::styled("c", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
            Span::raw("=Current | "),
            Span::styled("i", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::raw("=Incoming | "),
            Span::styled("b", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::raw(both_text),
        ]),
        Line::from(vec![
            Span::styled("j/k", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(" or "),
            Span::styled("↑/↓", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw("=Navigate conflicts | "),
            Span::styled("s", save_style.add_modifier(Modifier::BOLD)),
            Span::raw(save_text),
            Span::styled("Esc", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw("=Back | "),
            Span::styled("q", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw("=Quit"),
        ]),
        Line::from(if can_save {
            Span::styled(
                "All conflicts resolved! Press 's' to save.",
                Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
            )
        } else {
            Span::raw("")
        }),
    ])
    .block(Block::default().borders(Borders::ALL))
    .alignment(Alignment::Left);

    frame.render_widget(footer, chunks[2]);
}
