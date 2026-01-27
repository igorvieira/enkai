use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

use crate::app::AppState;
use crate::domain::Resolution;
use crate::tui::colors::MurasakiColors;

pub fn render_conflict_view(frame: &mut Frame, state: &AppState, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(0),    // Content
            Constraint::Length(1), // Footer (single-line, sticks to bottom)
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

    // Footer - keybindings (single line, full-width bar at bottom)
    let _both_selected = matches!(resolution, Some(Resolution::Both));
    let _both_text = if _both_selected {
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
        "({}/{} resolved)",
        file.resolved_count(),
        file.total_conflicts()
    );

    let footer_line = Line::from(vec![
        Span::styled("keys: ", Style::default().fg(Color::White)),
        Span::styled(
            "c",
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("=current ", Style::default().fg(Color::White)),
        Span::styled(
            "i",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("=incoming ", Style::default().fg(Color::White)),
        Span::styled(
            "b",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("=both ", Style::default().fg(Color::White)),
        Span::styled(
            "j/k",
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("=conflicts ", Style::default().fg(Color::White)),
        Span::styled(
            "s",
            save_style
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("=save ", Style::default().fg(Color::White)),
        Span::styled(
            save_text,
            Style::default().fg(save_style.fg.unwrap_or(Color::White)),
        ),
        Span::styled("  Esc", Style::default().add_modifier(Modifier::BOLD)),
        Span::styled("=back ", Style::default().fg(Color::White)),
        Span::styled("q", Style::default().add_modifier(Modifier::BOLD)),
        Span::styled("=quit", Style::default().fg(Color::White)),
    ]);

    let footer = Paragraph::new(footer_line)
        .style(Style::default().bg(MurasakiColors::FOOTER_BG))
        .alignment(Alignment::Left);

    frame.render_widget(footer, chunks[2]);
}
