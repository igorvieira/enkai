use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
    Frame,
};

use crate::app::{AppState, PaneFocus};
use crate::domain::Resolution;
use crate::git::FileStatus;
use crate::tui::colors::MurasakiColors;
use crate::tui::syntax::{SyntaxHighlighter, CODE_BG};

pub fn render_split_pane(frame: &mut Frame, state: &AppState, area: Rect) {
    // Split into left (file list) and right (code view)
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(30), // File list (30%)
            Constraint::Percentage(70), // Code view (70%)
        ])
        .split(area);

    render_file_list_pane(frame, state, chunks[0]);
    render_code_pane(frame, state, chunks[1]);

    // Render modals on top
    if state.show_commit_modal {
        render_commit_modal(frame, state, area);
    } else if state.show_help {
        render_help_modal(frame, state, area);
    }
}

fn render_file_list_pane(frame: &mut Frame, state: &AppState, area: Rect) {
    let is_focused = state.focus == PaneFocus::FileList;

    // Add focus indicator at the top
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Focus indicator
            Constraint::Min(0),    // File list
        ])
        .split(area);

    // Render focus indicator
    let focus_text = if is_focused { "▎FILES" } else { " Files" };
    let focus_color = if is_focused {
        MurasakiColors::CYAN_BRIGHT
    } else {
        MurasakiColors::TEXT_DIM
    };
    let focus_line = Line::from(vec![Span::styled(
        focus_text,
        Style::default()
            .fg(focus_color)
            .add_modifier(if is_focused {
                Modifier::BOLD
            } else {
                Modifier::empty()
            }),
    )]);
    frame.render_widget(Paragraph::new(focus_line), layout[0]);

    // Render file list based on mode
    if state.is_staging_mode() {
        render_staging_file_list(frame, state, layout[1]);
    } else {
        render_conflict_file_list(frame, state, layout[1]);
    }
}

fn render_conflict_file_list(frame: &mut Frame, state: &AppState, area: Rect) {
    let items: Vec<ListItem> = state
        .files
        .iter()
        .enumerate()
        .map(|(i, file)| {
            let status_icon = if file.is_fully_resolved() {
                "✓ "
            } else {
                "○ "
            };

            let status_color = if file.is_fully_resolved() {
                MurasakiColors::STATUS_RESOLVED
            } else {
                MurasakiColors::TEXT_DIM
            };

            let is_selected = i == state.selected_file;

            let line = if is_selected {
                Line::from(vec![
                    Span::styled(
                        status_icon,
                        Style::default()
                            .fg(status_color)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(format!(" {}", file.file_name()), Style::default()),
                ])
            } else {
                Line::from(vec![
                    Span::styled(status_icon, Style::default().fg(status_color)),
                    Span::styled(
                        format!(" {}", file.file_name()),
                        Style::default().fg(MurasakiColors::TEXT_DIM),
                    ),
                ])
            };

            let style = if is_selected {
                Style::default()
                    .bg(MurasakiColors::CYAN_BRIGHT)
                    .fg(Color::Black)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            ListItem::new(line).style(style)
        })
        .collect();

    frame.render_widget(List::new(items), area);
}

fn render_staging_file_list(frame: &mut Frame, state: &AppState, area: Rect) {
    // Separate files into categories
    let staged: Vec<(usize, &FileStatus)> = state
        .file_statuses
        .iter()
        .enumerate()
        .filter(|(_, f)| f.is_staged() && !f.is_modified_in_workdir())
        .collect();

    let both: Vec<(usize, &FileStatus)> = state
        .file_statuses
        .iter()
        .enumerate()
        .filter(|(_, f)| f.is_staged() && f.is_modified_in_workdir())
        .collect();

    let unstaged: Vec<(usize, &FileStatus)> = state
        .file_statuses
        .iter()
        .enumerate()
        .filter(|(_, f)| !f.is_staged() && f.is_modified_in_workdir())
        .collect();

    let mut items: Vec<ListItem> = Vec::new();

    // Staged section
    if !staged.is_empty() {
        items.push(ListItem::new(Line::from(vec![Span::styled(
            "▎STAGED",
            Style::default()
                .fg(MurasakiColors::SUCCESS)
                .add_modifier(Modifier::BOLD),
        )])));
        for (idx, file) in &staged {
            items.push(create_staging_list_item(
                *idx,
                file,
                state.selected_file,
                MurasakiColors::SUCCESS,
            ));
        }
    }

    // Both staged and modified section
    if !both.is_empty() {
        items.push(ListItem::new(Line::from(vec![Span::styled(
            "▎BOTH",
            Style::default()
                .fg(MurasakiColors::WARNING)
                .add_modifier(Modifier::BOLD),
        )])));
        for (idx, file) in &both {
            items.push(create_staging_list_item(
                *idx,
                file,
                state.selected_file,
                MurasakiColors::WARNING,
            ));
        }
    }

    // Unstaged section
    if !unstaged.is_empty() {
        items.push(ListItem::new(Line::from(vec![Span::styled(
            "▎UNSTAGED",
            Style::default()
                .fg(MurasakiColors::ERROR)
                .add_modifier(Modifier::BOLD),
        )])));
        for (idx, file) in &unstaged {
            items.push(create_staging_list_item(
                *idx,
                file,
                state.selected_file,
                MurasakiColors::ERROR,
            ));
        }
    }

    if items.is_empty() {
        let empty = Paragraph::new("No changes")
            .style(Style::default().fg(MurasakiColors::TEXT_DIM))
            .alignment(Alignment::Center);
        frame.render_widget(empty, area);
    } else {
        frame.render_widget(List::new(items), area);
    }
}

fn create_staging_list_item(
    idx: usize,
    file: &FileStatus,
    selected: usize,
    color: Color,
) -> ListItem<'static> {
    let is_selected = idx == selected;
    let status_display = file.display_status();
    let path = file.path.display().to_string();
    let text = format!("  [{}] {}", status_display, path);

    let style = if is_selected {
        Style::default()
            .fg(Color::Black)
            .bg(color)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(color)
    };

    ListItem::new(Line::from(text)).style(style)
}

fn render_code_pane(frame: &mut Frame, state: &AppState, area: Rect) {
    if state.is_staging_mode() {
        render_diff_pane(frame, state, area);
    } else {
        render_conflict_code_pane(frame, state, area);
    }
}

fn render_diff_pane(frame: &mut Frame, state: &AppState, area: Rect) {
    let is_focused = state.focus == PaneFocus::CodeView;

    // Split into header, content, footer
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2), // Header
            Constraint::Min(0),    // Content
            Constraint::Length(1), // Footer
        ])
        .split(area);

    // Get file name for header
    let file_name = state
        .current_file_status()
        .map(|f| f.path.display().to_string())
        .unwrap_or_else(|| "No file selected".to_string());

    // Header
    let header_text = if is_focused {
        format!("▎DIFF  •  {}", file_name)
    } else {
        format!(" Diff  •  {}", file_name)
    };

    let header_color = if is_focused {
        MurasakiColors::CYAN_BRIGHT
    } else {
        MurasakiColors::TEXT_DIM
    };

    let header = Paragraph::new(Line::from(vec![Span::styled(
        header_text,
        Style::default()
            .fg(header_color)
            .add_modifier(if is_focused {
                Modifier::BOLD
            } else {
                Modifier::empty()
            }),
    )]));
    frame.render_widget(header, chunks[0]);

    // Content - render diff
    if let Some(ref diff) = state.diff_content {
        render_diff_content(frame, diff, state.scroll_offset, chunks[1]);
    } else {
        let empty = Paragraph::new("No changes to display")
            .style(Style::default().fg(MurasakiColors::TEXT_DIM).bg(CODE_BG));
        frame.render_widget(empty, chunks[1]);
    }

    // Footer
    render_footer(frame, chunks[2]);
}

fn render_diff_content(frame: &mut Frame, diff: &str, scroll_offset: u16, area: Rect) {
    let lines: Vec<Line> = diff
        .lines()
        .map(|line| {
            if line.starts_with('+') && !line.starts_with("+++") {
                Line::from(Span::styled(
                    line.to_string(),
                    Style::default().fg(Color::Green).bg(Color::Rgb(20, 40, 20)),
                ))
            } else if line.starts_with('-') && !line.starts_with("---") {
                Line::from(Span::styled(
                    line.to_string(),
                    Style::default().fg(Color::Red).bg(Color::Rgb(50, 20, 20)),
                ))
            } else if line.starts_with("@@") {
                Line::from(Span::styled(
                    line.to_string(),
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ))
            } else if line.starts_with("diff ") || line.starts_with("index ") {
                Line::from(Span::styled(
                    line.to_string(),
                    Style::default().fg(Color::Yellow),
                ))
            } else if line.starts_with("---") || line.starts_with("+++") {
                Line::from(Span::styled(
                    line.to_string(),
                    Style::default()
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD),
                ))
            } else {
                Line::from(Span::styled(
                    line.to_string(),
                    Style::default().fg(Color::Gray),
                ))
            }
        })
        .collect();

    let paragraph = Paragraph::new(lines)
        .style(Style::default().bg(CODE_BG))
        .scroll((scroll_offset, 0));

    frame.render_widget(paragraph, area);
}

fn render_conflict_code_pane(frame: &mut Frame, state: &AppState, area: Rect) {
    let is_focused = state.focus == PaneFocus::CodeView;

    // Get current file
    let file = match state.current_file() {
        Some(f) => f,
        None => {
            let empty = Paragraph::new("No file selected")
                .style(Style::default().fg(MurasakiColors::TEXT_DIM));
            frame.render_widget(empty, area);
            return;
        }
    };

    let conflict_index = state.current_conflict_index().unwrap_or(0);

    // Split code pane into header, content, and footer
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2), // Header
            Constraint::Min(0),    // Content
            Constraint::Length(1), // Footer
        ])
        .split(area);

    // Header with focus indicator
    let header_text = if is_focused {
        format!(
            "▎CODE  •  {}  •  Conflict {}/{}  •  Resolved {}/{}",
            file.file_name(),
            conflict_index + 1,
            file.total_conflicts(),
            file.resolved_count(),
            file.total_conflicts()
        )
    } else {
        format!(
            " Code  •  {}  •  Conflict {}/{}  •  Resolved {}/{}",
            file.file_name(),
            conflict_index + 1,
            file.total_conflicts(),
            file.resolved_count(),
            file.total_conflicts()
        )
    };

    let header_color = if is_focused {
        MurasakiColors::CYAN_BRIGHT
    } else {
        MurasakiColors::TEXT_DIM
    };

    let header = Paragraph::new(Line::from(vec![Span::styled(
        header_text,
        Style::default()
            .fg(header_color)
            .add_modifier(if is_focused {
                Modifier::BOLD
            } else {
                Modifier::empty()
            }),
    )]));
    frame.render_widget(header, chunks[0]);

    // Content - show full file with conflict highlighted
    render_file_content(frame, state, file, conflict_index, chunks[1]);

    // Footer
    render_footer(frame, chunks[2]);
}

fn render_file_content(
    frame: &mut Frame,
    state: &AppState,
    file: &crate::domain::ConflictedFile,
    conflict_index: usize,
    area: Rect,
) {
    let lines: Vec<&str> = file.original_content.lines().collect();
    let mut display_lines: Vec<Line> = Vec::new();

    if conflict_index >= file.conflicts.len() {
        let content = Paragraph::new("No conflicts in this file")
            .style(Style::default().fg(MurasakiColors::TEXT_DIM));
        frame.render_widget(content, area);
        return;
    }

    // Initialize syntax highlighter
    let highlighter = SyntaxHighlighter::new();
    let syntax = highlighter.detect_syntax(&file.path);

    let mut line_idx = 0;

    while line_idx < lines.len() {
        // Check if this line starts any conflict
        let conflict_at_line = file
            .conflicts
            .iter()
            .enumerate()
            .find(|(_, c)| c.start_line == line_idx);

        if let Some((idx, conflict)) = conflict_at_line {
            // This is a conflict
            let resolution = file.resolutions[idx];
            let is_current_conflict = idx == conflict_index;
            let both_selected = matches!(resolution, Some(Resolution::Both));
            let current_selected = matches!(
                resolution,
                Some(Resolution::Current) | Some(Resolution::Both)
            );
            let incoming_selected = matches!(
                resolution,
                Some(Resolution::Incoming) | Some(Resolution::Both)
            );
            let is_resolved = resolution.is_some();

            if !is_resolved {
                let marker_text = if is_current_conflict {
                    "<<<<<<< CURRENT (HEAD) ◀"
                } else {
                    "<<<<<<< CURRENT (HEAD)"
                };
                display_lines.push(Line::from(Span::styled(
                    marker_text,
                    Style::default()
                        .fg(MurasakiColors::CONFLICT_MARKER)
                        .add_modifier(Modifier::BOLD),
                )));
            }

            let show_current = resolution.is_none() || current_selected;

            if show_current {
                for line in conflict.current.lines() {
                    let highlighted = highlighter.highlight_line(line, syntax);
                    let spans: Vec<Span> = highlighted
                        .into_iter()
                        .map(|(style, text)| {
                            let final_style = if is_resolved {
                                if both_selected {
                                    style.bg(MurasakiColors::CONFLICT_BOTH_BG)
                                } else {
                                    style.bg(MurasakiColors::RESOLVED_BG)
                                }
                            } else {
                                style.bg(MurasakiColors::CONFLICT_CURRENT_BG)
                            };
                            Span::styled(text, final_style)
                        })
                        .collect();

                    display_lines.push(Line::from(spans));
                }
            }

            if !is_resolved {
                display_lines.push(Line::from(Span::styled(
                    "=======",
                    Style::default()
                        .fg(MurasakiColors::CONFLICT_MARKER)
                        .add_modifier(Modifier::BOLD),
                )));
            }

            let show_incoming = resolution.is_none() || incoming_selected;

            if show_incoming {
                for line in conflict.incoming.lines() {
                    let highlighted = highlighter.highlight_line(line, syntax);
                    let spans: Vec<Span> = highlighted
                        .into_iter()
                        .map(|(style, text)| {
                            let final_style = if is_resolved {
                                if both_selected {
                                    style.bg(MurasakiColors::CONFLICT_BOTH_BG)
                                } else {
                                    style.bg(MurasakiColors::RESOLVED_BG)
                                }
                            } else {
                                style.bg(MurasakiColors::CONFLICT_INCOMING_BG)
                            };
                            Span::styled(text, final_style)
                        })
                        .collect();

                    display_lines.push(Line::from(spans));
                }
            }

            if !is_resolved {
                let marker_text = if is_current_conflict {
                    ">>>>>>> INCOMING ◀"
                } else {
                    ">>>>>>> INCOMING"
                };
                display_lines.push(Line::from(Span::styled(
                    marker_text,
                    Style::default()
                        .fg(MurasakiColors::CONFLICT_MARKER)
                        .add_modifier(Modifier::BOLD),
                )));
            }

            line_idx = conflict.end_line + 1;
        } else {
            let highlighted = highlighter.highlight_line(lines[line_idx], syntax);
            let spans: Vec<Span> = highlighted
                .into_iter()
                .map(|(style, text)| Span::styled(text, style))
                .collect();

            display_lines.push(Line::from(spans));
            line_idx += 1;
        }
    }

    let content = Paragraph::new(display_lines)
        .style(Style::default().bg(CODE_BG))
        .wrap(Wrap { trim: false })
        .scroll((state.scroll_offset, 0));

    frame.render_widget(content, area);
}

fn render_footer(frame: &mut Frame, area: Rect) {
    let hint = Line::from(vec![
        Span::styled("Press ", Style::default().fg(MurasakiColors::TEXT_DIM)),
        Span::styled(
            "?",
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" for help", Style::default().fg(MurasakiColors::TEXT_DIM)),
    ]);

    let footer = Paragraph::new(hint)
        .style(Style::default().bg(MurasakiColors::FOOTER_BG))
        .alignment(Alignment::Center);

    frame.render_widget(footer, area);
}

fn render_commit_modal(frame: &mut Frame, state: &AppState, area: Rect) {
    // Calculate modal size and position
    let modal_width = 60u16;
    let modal_height = 10u16;
    let x = (area.width.saturating_sub(modal_width)) / 2;
    let y = (area.height.saturating_sub(modal_height)) / 2;

    let modal_area = Rect::new(
        x,
        y,
        modal_width.min(area.width),
        modal_height.min(area.height),
    );

    // Clear the area behind the modal
    frame.render_widget(Clear, modal_area);

    // Modal content
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2), // Header
            Constraint::Length(1), // Input prompt
            Constraint::Length(1), // Input field
            Constraint::Length(1), // Error message
            Constraint::Min(0),    // Spacer
            Constraint::Length(2), // Footer
        ])
        .split(modal_area);

    // Background
    let bg = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(MurasakiColors::CYAN_BRIGHT))
        .style(Style::default().bg(MurasakiColors::FOOTER_BG));
    frame.render_widget(bg, modal_area);

    // Header
    let header = Paragraph::new(Line::from(vec![Span::styled(
        " Commit Changes",
        Style::default()
            .fg(MurasakiColors::CYAN_BRIGHT)
            .add_modifier(Modifier::BOLD),
    )]));
    frame.render_widget(header, chunks[0]);

    // Input prompt
    let prompt = Paragraph::new(Line::from(vec![Span::styled(
        "  Enter commit message:",
        Style::default().fg(Color::White),
    )]));
    frame.render_widget(prompt, chunks[1]);

    // Input field
    let input_text = format!("  > {}_", state.commit_message);
    let input = Paragraph::new(Line::from(vec![Span::styled(
        input_text,
        Style::default().fg(MurasakiColors::CYAN_BRIGHT),
    )]));
    frame.render_widget(input, chunks[2]);

    // Error message
    if let Some(ref error) = state.commit_error {
        let error_line = Paragraph::new(Line::from(vec![Span::styled(
            format!("  {}", error),
            Style::default().fg(MurasakiColors::ERROR),
        )]));
        frame.render_widget(error_line, chunks[3]);
    }

    // Footer
    let footer = Paragraph::new(Line::from(vec![
        Span::styled(
            "  Enter",
            Style::default()
                .fg(MurasakiColors::CYAN_BRIGHT)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("=Commit  ", Style::default().fg(MurasakiColors::TEXT_DIM)),
        Span::styled(
            "Esc",
            Style::default()
                .fg(MurasakiColors::CYAN_BRIGHT)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("=Cancel", Style::default().fg(MurasakiColors::TEXT_DIM)),
    ]));
    frame.render_widget(footer, chunks[5]);
}

fn render_help_modal(frame: &mut Frame, state: &AppState, area: Rect) {
    // Calculate modal size and position (centered)
    let modal_width = 52u16;
    let modal_height = 24u16;
    let x = (area.width.saturating_sub(modal_width)) / 2;
    let y = (area.height.saturating_sub(modal_height)) / 2;

    let modal_area = Rect::new(
        x,
        y,
        modal_width.min(area.width),
        modal_height.min(area.height),
    );

    // Clear the area behind the modal
    frame.render_widget(Clear, modal_area);

    // Build help content based on mode
    let mut help_lines = vec![
        Line::from(vec![Span::styled(
            "Keyboard Shortcuts",
            Style::default()
                .fg(MurasakiColors::CYAN_BRIGHT)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "  Navigation",
            Style::default().fg(MurasakiColors::PURPLE_BRIGHT),
        )]),
        Line::from(vec![
            Span::styled(
                "    j/k    ",
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "Scroll up/down",
                Style::default().fg(MurasakiColors::TEXT_DIM),
            ),
        ]),
        Line::from(vec![
            Span::styled(
                "    Tab    ",
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "Switch focus",
                Style::default().fg(MurasakiColors::TEXT_DIM),
            ),
        ]),
    ];

    if state.is_staging_mode() {
        // Staging mode help
        help_lines.push(Line::from(""));
        help_lines.push(Line::from(vec![Span::styled(
            "  Staging",
            Style::default().fg(MurasakiColors::PURPLE_BRIGHT),
        )]));
        help_lines.push(Line::from(vec![
            Span::styled(
                "    a      ",
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("Stage file", Style::default().fg(MurasakiColors::TEXT_DIM)),
        ]));
        help_lines.push(Line::from(vec![
            Span::styled(
                "    s      ",
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "Unstage file",
                Style::default().fg(MurasakiColors::TEXT_DIM),
            ),
        ]));
        help_lines.push(Line::from(vec![
            Span::styled(
                "    r      ",
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "Restore file (discard changes)",
                Style::default().fg(MurasakiColors::TEXT_DIM),
            ),
        ]));
        help_lines.push(Line::from(vec![
            Span::styled(
                "    c      ",
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "Commit staged changes",
                Style::default().fg(MurasakiColors::TEXT_DIM),
            ),
        ]));
    } else {
        // Conflict mode help
        help_lines.push(Line::from(vec![
            Span::styled(
                "    n/p    ",
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "Next/previous conflict",
                Style::default().fg(MurasakiColors::TEXT_DIM),
            ),
        ]));
        help_lines.push(Line::from(""));
        help_lines.push(Line::from(vec![Span::styled(
            "  Conflict Resolution",
            Style::default().fg(MurasakiColors::PURPLE_BRIGHT),
        )]));
        help_lines.push(Line::from(vec![
            Span::styled(
                "    c      ",
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "Choose current (HEAD)",
                Style::default().fg(MurasakiColors::TEXT_DIM),
            ),
        ]));
        help_lines.push(Line::from(vec![
            Span::styled(
                "    i      ",
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "Choose incoming",
                Style::default().fg(MurasakiColors::TEXT_DIM),
            ),
        ]));
        help_lines.push(Line::from(vec![
            Span::styled(
                "    b      ",
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("Choose both", Style::default().fg(MurasakiColors::TEXT_DIM)),
        ]));
        help_lines.push(Line::from(vec![
            Span::styled(
                "    u      ",
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "Undo resolution",
                Style::default().fg(MurasakiColors::TEXT_DIM),
            ),
        ]));
    }

    // Common actions
    help_lines.push(Line::from(""));
    help_lines.push(Line::from(vec![Span::styled(
        "  Actions",
        Style::default().fg(MurasakiColors::PURPLE_BRIGHT),
    )]));
    help_lines.push(Line::from(vec![
        Span::styled(
            "    q      ",
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("Quit", Style::default().fg(MurasakiColors::TEXT_DIM)),
    ]));

    let help_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(MurasakiColors::CYAN_BRIGHT))
        .style(Style::default().bg(MurasakiColors::FOOTER_BG));

    let help_content = Paragraph::new(help_lines)
        .block(help_block)
        .alignment(Alignment::Left);

    frame.render_widget(help_content, modal_area);
}
