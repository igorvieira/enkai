use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{List, ListItem, Paragraph},
    Frame,
};

use crate::app::AppState;
use crate::tui::colors::MurasakiColors;

pub fn render_rebase_actions(frame: &mut Frame, _state: &AppState, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(4), // Header
            Constraint::Min(0),    // Actions
            Constraint::Length(2), // Footer
        ])
        .split(area);

    // Header - no borders
    let header = Paragraph::new(vec![
        Line::from(vec![Span::styled(
            "All Conflicts Resolved!",
            Style::default()
                .fg(MurasakiColors::SUCCESS)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from(Span::styled(
            "What would you like to do with the rebase?",
            Style::default().fg(MurasakiColors::TEXT_NORMAL),
        )),
    ])
    .alignment(Alignment::Center);

    frame.render_widget(header, chunks[0]);

    // Actions
    let actions = vec![
        ListItem::new(Line::from(vec![
            Span::styled(
                "c",
                Style::default()
                    .fg(MurasakiColors::CONFLICT_CURRENT)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                " - Continue rebase (git rebase --continue)",
                Style::default().fg(MurasakiColors::TEXT_NORMAL),
            ),
        ])),
        ListItem::new(Line::from(vec![
            Span::styled(
                "a",
                Style::default()
                    .fg(MurasakiColors::ERROR)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                " - Abort rebase (git rebase --abort)",
                Style::default().fg(MurasakiColors::TEXT_NORMAL),
            ),
        ])),
        ListItem::new(Line::from(vec![
            Span::styled(
                "s",
                Style::default()
                    .fg(MurasakiColors::WARNING)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                " - Skip current commit (git rebase --skip)",
                Style::default().fg(MurasakiColors::TEXT_NORMAL),
            ),
        ])),
    ];

    // No borders on list
    let list = List::new(actions);

    frame.render_widget(list, chunks[1]);

    // Footer - no borders
    let footer = Paragraph::new(Line::from(vec![
        Span::styled(
            "q",
            Style::default()
                .fg(MurasakiColors::PINK_HOT)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("=Exit  ", Style::default().fg(MurasakiColors::TEXT_DIM)),
        Span::styled(
            "Esc",
            Style::default()
                .fg(MurasakiColors::PINK_HOT)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("=Cancel", Style::default().fg(MurasakiColors::TEXT_DIM)),
    ]))
    .alignment(Alignment::Center);

    frame.render_widget(footer, chunks[2]);
}
