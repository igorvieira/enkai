use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{List, ListItem, Paragraph, Wrap},
    Frame,
};

use crate::app::{AppState, PaneFocus};
use crate::domain::Resolution;
use crate::tui::colors::EnkaiColors;
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
}

fn render_file_list_pane(frame: &mut Frame, state: &AppState, area: Rect) {
    let is_focused = state.focus == PaneFocus::FileList;

    // Add focus indicator at the top if focused
    let chunks = if is_focused {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),  // Focus indicator
                Constraint::Min(0),     // File list
            ])
            .split(area);

        // Render focus indicator
        let focus_line = Line::from(vec![
            Span::styled(
                "▎FILES",
                Style::default()
                    .fg(EnkaiColors::CYAN_BRIGHT)
                    .add_modifier(Modifier::BOLD),
            ),
        ]);
        let focus_indicator = Paragraph::new(focus_line);
        frame.render_widget(focus_indicator, layout[0]);

        layout
    } else {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),  // Empty space
                Constraint::Min(0),     // File list
            ])
            .split(area);

        // Render dimmed indicator
        let focus_line = Line::from(vec![
            Span::styled(
                " Files",
                Style::default().fg(EnkaiColors::TEXT_DIM),
            ),
        ]);
        let focus_indicator = Paragraph::new(focus_line);
        frame.render_widget(focus_indicator, layout[0]);

        layout
    };

    let items: Vec<ListItem> = state
        .files
        .iter()
        .enumerate()
        .map(|(i, file)| {
            let status_icon = if file.is_fully_resolved() {
                "✓ "
            } else {
                "○ " // Circle for unresolved
            };

            let status_color = if file.is_fully_resolved() {
                EnkaiColors::STATUS_RESOLVED
            } else {
                EnkaiColors::TEXT_DIM
            };

            let is_selected = i == state.selected_file;

            let line = if is_selected {
                // Selected item: cyan background, BLACK text
                Line::from(vec![
                    Span::styled(
                        status_icon,
                        Style::default().fg(status_color).add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        format!(" {}", file.file_name()),
                        Style::default(), // Will use black due to bg override
                    ),
                ])
            } else {
                // Unselected item
                Line::from(vec![
                    Span::styled(
                        status_icon,
                        Style::default().fg(status_color),
                    ),
                    Span::styled(
                        format!(" {}", file.file_name()),
                        Style::default().fg(EnkaiColors::TEXT_DIM),
                    ),
                ])
            };

            let style = if is_selected {
                use ratatui::style::Color;
                Style::default()
                    .bg(EnkaiColors::CYAN_BRIGHT)
                    .fg(Color::Black)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            ListItem::new(line).style(style)
        })
        .collect();

    // No borders, just the list
    let list = List::new(items);

    frame.render_widget(list, chunks[1]);
}

fn render_code_pane(frame: &mut Frame, state: &AppState, area: Rect) {
    let is_focused = state.focus == PaneFocus::CodeView;

    // Get current file
    let file = match state.current_file() {
        Some(f) => f,
        None => {
            let empty = Paragraph::new("No file selected")
                .style(Style::default().fg(EnkaiColors::TEXT_DIM));
            frame.render_widget(empty, area);
            return;
        }
    };

    let conflict_index = state.current_conflict_index().unwrap_or(0);

    // Split code pane into header, content, and footer
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),  // Header (with focus indicator)
            Constraint::Min(0),     // Content
            Constraint::Length(1),  // Footer (1 line, no padding)
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
        EnkaiColors::CYAN_BRIGHT
    } else {
        EnkaiColors::TEXT_DIM
    };

    let header = Paragraph::new(Line::from(vec![
        Span::styled(header_text, Style::default().fg(header_color).add_modifier(if is_focused { Modifier::BOLD } else { Modifier::empty() })),
    ]));

    frame.render_widget(header, chunks[0]);

    // Content - show full file with conflict highlighted
    render_file_content(frame, state, file, conflict_index, chunks[1]);

    // Footer - keybindings
    render_footer(frame, state, file, chunks[2]);
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
            .style(Style::default().fg(EnkaiColors::TEXT_DIM));
        frame.render_widget(content, area);
        return;
    }

    // Initialize syntax highlighter
    let highlighter = SyntaxHighlighter::new();
    let syntax = highlighter.detect_syntax(&file.path);

    let mut line_idx = 0;

    while line_idx < lines.len() {
        // Check if this line starts any conflict
        let conflict_at_line = file.conflicts.iter().enumerate().find(|(_, c)| c.start_line == line_idx);

        if let Some((idx, conflict)) = conflict_at_line {
            // This is a conflict
            let resolution = file.resolutions[idx];
            let is_current_conflict = idx == conflict_index;
            let current_selected = matches!(resolution, Some(Resolution::Current) | Some(Resolution::Both));
            let incoming_selected = matches!(resolution, Some(Resolution::Incoming) | Some(Resolution::Both));
            let is_resolved = resolution.is_some();

            if !is_resolved {
                // Show conflict markers only when not resolved
                let marker_text = if is_current_conflict {
                    "<<<<<<< CURRENT (HEAD) ◀"  // Indicator for active conflict
                } else {
                    "<<<<<<< CURRENT (HEAD)"
                };
                display_lines.push(Line::from(Span::styled(
                    marker_text,
                    Style::default().fg(EnkaiColors::CONFLICT_MARKER).add_modifier(Modifier::BOLD),
                )));
            }

            // Show current content
            let show_current = resolution.is_none() || current_selected;

            if show_current {
                for line in conflict.current.lines() {
                    // Highlight the code with syntax colors
                    let highlighted = highlighter.highlight_line(line, syntax);
                    let spans: Vec<Span> = highlighted
                        .into_iter()
                        .map(|(style, text)| {
                            let final_style = if is_resolved {
                                // Resolved: subtle green background, no other styling
                                style.bg(EnkaiColors::RESOLVED_BG)
                            } else if current_selected {
                                // Unresolved but hovering: cyan background
                                style.bg(EnkaiColors::CONFLICT_CURRENT_BG)
                            } else {
                                // Unresolved: cyan background
                                style.bg(EnkaiColors::CONFLICT_CURRENT_BG)
                            };
                            Span::styled(text, final_style)
                        })
                        .collect();

                    display_lines.push(Line::from(spans));
                }
            }

            if !is_resolved {
                // Show separator only when not resolved
                display_lines.push(Line::from(Span::styled(
                    "=======",
                    Style::default().fg(EnkaiColors::CONFLICT_MARKER).add_modifier(Modifier::BOLD),
                )));
            }

            // Show incoming content
            let show_incoming = resolution.is_none() || incoming_selected;

            if show_incoming {
                for line in conflict.incoming.lines() {
                    // Highlight the code with syntax colors
                    let highlighted = highlighter.highlight_line(line, syntax);
                    let spans: Vec<Span> = highlighted
                        .into_iter()
                        .map(|(style, text)| {
                            let final_style = if is_resolved {
                                // Resolved: subtle green background, no other styling
                                style.bg(EnkaiColors::RESOLVED_BG)
                            } else if incoming_selected {
                                // Unresolved but hovering: orange background
                                style.bg(EnkaiColors::CONFLICT_INCOMING_BG)
                            } else {
                                // Unresolved: orange background
                                style.bg(EnkaiColors::CONFLICT_INCOMING_BG)
                            };
                            Span::styled(text, final_style)
                        })
                        .collect();

                    display_lines.push(Line::from(spans));
                }
            }

            if !is_resolved {
                // Show conflict end marker only when not resolved
                let marker_text = if is_current_conflict {
                    ">>>>>>> INCOMING ◀"  // Indicator for active conflict
                } else {
                    ">>>>>>> INCOMING"
                };
                display_lines.push(Line::from(Span::styled(
                    marker_text,
                    Style::default().fg(EnkaiColors::CONFLICT_MARKER).add_modifier(Modifier::BOLD),
                )));
            }

            line_idx = conflict.end_line + 1;
        } else {
            // Regular line (with syntax highlighting)
            let highlighted = highlighter.highlight_line(lines[line_idx], syntax);
            let spans: Vec<Span> = highlighted
                .into_iter()
                .map(|(style, text)| Span::styled(text, style))
                .collect();

            display_lines.push(Line::from(spans));
            line_idx += 1;
        }
    }

    // No borders, just the content with gray background
    let content = Paragraph::new(display_lines)
        .style(Style::default().bg(CODE_BG))
        .wrap(Wrap { trim: false })
        .scroll((state.scroll_offset, 0)); // Vertical scroll

    frame.render_widget(content, area);
}

fn render_footer(
    frame: &mut Frame,
    state: &AppState,
    file: &crate::domain::ConflictedFile,
    area: Rect,
) {
    let _can_save = file.is_fully_resolved();

    // Fill the entire footer area with background color first
    let background = Paragraph::new("")
        .style(Style::default().bg(EnkaiColors::FOOTER_BG));
    frame.render_widget(background, area);

    // Create horizontal layout with minimal left padding (no top/bottom padding)
    let horizontal_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(2),  // Minimal left padding
            Constraint::Min(0),     // Content area
        ])
        .split(area);

    let white = Color::White;

    let keybindings = if state.focus == PaneFocus::CodeView {
        Line::from(vec![
            Span::styled("keys: ", Style::default().fg(white)),
            Span::styled("j/k", Style::default().fg(white).add_modifier(Modifier::BOLD)),
            Span::styled("=navigate ", Style::default().fg(white)),
            Span::styled("ctrl+d/u", Style::default().fg(white).add_modifier(Modifier::BOLD)),
            Span::styled("=scroll ", Style::default().fg(white)),
            Span::styled("c", Style::default().fg(white).add_modifier(Modifier::BOLD)),
            Span::styled("=current ", Style::default().fg(white)),
            Span::styled("i", Style::default().fg(white).add_modifier(Modifier::BOLD)),
            Span::styled("=incoming ", Style::default().fg(white)),
            Span::styled("b", Style::default().fg(white).add_modifier(Modifier::BOLD)),
            Span::styled("=both ", Style::default().fg(white)),
            Span::styled("u", Style::default().fg(white).add_modifier(Modifier::BOLD)),
            Span::styled("=undo ", Style::default().fg(white)),
            Span::styled("s", Style::default().fg(white).add_modifier(Modifier::BOLD)),
            Span::styled("=save ", Style::default().fg(white)),
            Span::styled("tab", Style::default().fg(white).add_modifier(Modifier::BOLD)),
            Span::styled("=files ", Style::default().fg(white)),
            Span::styled("q", Style::default().fg(white).add_modifier(Modifier::BOLD)),
            Span::styled("=quit", Style::default().fg(white)),
        ])
    } else {
        Line::from(vec![
            Span::styled("keys: ", Style::default().fg(white)),
            Span::styled("j/k", Style::default().fg(white).add_modifier(Modifier::BOLD)),
            Span::styled("=navigate ", Style::default().fg(white)),
            Span::styled("tab", Style::default().fg(white).add_modifier(Modifier::BOLD)),
            Span::styled("=code ", Style::default().fg(white)),
            Span::styled("q", Style::default().fg(white).add_modifier(Modifier::BOLD)),
            Span::styled("=quit", Style::default().fg(white)),
        ])
    };

    // Footer content with left alignment, single line
    let footer = Paragraph::new(keybindings)
        .alignment(Alignment::Left);

    frame.render_widget(footer, horizontal_chunks[1]);
}
