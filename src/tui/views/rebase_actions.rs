use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

use crate::app::AppState;

pub fn render_rebase_actions(frame: &mut Frame, _state: &AppState, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5),  // Header
            Constraint::Min(0),     // Actions
            Constraint::Length(3),  // Footer
        ])
        .split(area);

    // Header
    let header = Paragraph::new(vec![
        Line::from(vec![
            Span::styled(
                "All Conflicts Resolved!",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(""),
        Line::from("What would you like to do with the rebase?"),
    ])
    .block(Block::default().borders(Borders::ALL))
    .alignment(Alignment::Center);

    frame.render_widget(header, chunks[0]);

    // Actions
    let actions = vec![
        ListItem::new(Line::from(vec![
            Span::styled(
                "c",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" - Continue rebase (git rebase --continue)"),
        ])),
        ListItem::new(Line::from(vec![
            Span::styled(
                "a",
                Style::default()
                    .fg(Color::Red)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" - Abort rebase (git rebase --abort)"),
        ])),
        ListItem::new(Line::from(vec![
            Span::styled(
                "s",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" - Skip current commit (git rebase --skip)"),
        ])),
    ];

    let list = List::new(actions).block(
        Block::default()
            .borders(Borders::ALL)
            .title("Rebase Actions"),
    );

    frame.render_widget(list, chunks[1]);

    // Footer
    let footer = Paragraph::new(vec![Line::from(vec![
        Span::styled("q", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(" or "),
        Span::styled("Esc", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(" - Exit without taking action"),
    ])])
    .block(Block::default().borders(Borders::ALL))
    .alignment(Alignment::Center);

    frame.render_widget(footer, chunks[2]);
}
