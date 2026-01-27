use anyhow::Result;
use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use git2::Repository;
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, List, ListItem, ListState, Paragraph},
    Frame, Terminal,
};
use std::io;

use crate::git::{
    commit_changes, get_file_diff, get_repository_status, restore_all, restore_file, stage_all,
    stage_file, unstage_all, unstage_file, FileStatus,
};
use crate::tui::colors::MurasakiColors;

const MURASAKI_BANNER: &str = r#"

           ███╗   ███╗██╗   ██╗██████╗  █████╗ ███████╗ █████╗ ██╗  ██╗██╗
           ████╗ ████║██║   ██║██╔══██╗██╔══██╗██╔════╝██╔══██╗██║ ██╔╝██║
           ██╔████╔██║██║   ██║██████╔╝███████║███████╗███████║█████╔╝ ██║
           ██║╚██╔╝██║██║   ██║██╔══██╗██╔══██║╚════██║██╔══██║██╔═██╗ ██║
           ██║ ╚═╝ ██║╚██████╔╝██║  ██║██║  ██║███████║██║  ██║██║  ██╗██║
           ╚═╝     ╚═╝ ╚═════╝ ╚═╝  ╚═╝╚═╝  ╚═╝╚══════╝╚═╝  ╚═╝╚═╝  ╚═╝╚═╝

"#;

pub struct StatusView {
    files: Vec<FileStatus>,
    selected_file_index: usize,
    current_view: RightPanelView,
    file_content: Option<String>,
    file_diff: Option<String>,
    diff_scroll: usize,
    commit_message: String,
    commit_error: Option<String>,
}

#[derive(Debug, PartialEq)]
enum RightPanelView {
    Banner,
    FileContent,
    CommitModal,
}

impl StatusView {
    pub fn new(files: Vec<FileStatus>) -> Self {
        let mut view = Self {
            files,
            selected_file_index: 0,
            current_view: RightPanelView::Banner,
            file_content: None,
            file_diff: None,
            diff_scroll: 0,
            commit_message: String::new(),
            commit_error: None,
        };
        view.load_file_diff();
        view
    }

    fn load_file_diff(&mut self) {
        if self.files.is_empty() {
            self.file_diff = None;
            return;
        }

        let file = &self.files[self.selected_file_index];
        let path = file.path.to_string_lossy();

        // Get diff based on file status (staged or unstaged)
        let staged = file.is_staged() && !file.is_modified_in_workdir();

        match get_file_diff(&path, staged) {
            Ok(diff) => {
                if diff.is_empty() {
                    // If no diff (e.g., new untracked file), try to read file content
                    if let Ok(content) = std::fs::read_to_string(&file.path) {
                        self.file_diff = Some(format!("New file:\n\n{}", content));
                    } else {
                        self.file_diff = Some("No changes to display".to_string());
                    }
                } else {
                    self.file_diff = Some(diff);
                }
            }
            Err(e) => {
                self.file_diff = Some(format!("Error getting diff: {}", e));
            }
        }
        self.diff_scroll = 0;
    }

    fn load_file_content(&mut self) -> Result<()> {
        if self.files.is_empty() {
            return Ok(());
        }

        let file = &self.files[self.selected_file_index];
        let path = &file.path;

        // Read file content
        match std::fs::read_to_string(path) {
            Ok(content) => {
                self.file_content = Some(content);
            }
            Err(e) => {
                self.file_content = Some(format!("Error reading file: {}", e));
            }
        }

        Ok(())
    }

    fn handle_key(&mut self, key: KeyEvent) -> Result<(bool, bool)> {
        // Returns (should_quit, should_refresh)

        // Handle commit modal separately
        if self.current_view == RightPanelView::CommitModal {
            match key.code {
                KeyCode::Enter => {
                    // Submit commit
                    if !self.commit_message.is_empty() {
                        match commit_changes(&self.commit_message) {
                            Ok(_) => {
                                self.commit_message.clear();
                                self.commit_error = None;
                                self.current_view = RightPanelView::Banner;
                                return Ok((false, true)); // Refresh after commit
                            }
                            Err(e) => {
                                self.commit_error = Some(format!("Commit failed: {}", e));
                            }
                        }
                    }
                }
                KeyCode::Esc => {
                    // Cancel commit
                    self.commit_message.clear();
                    self.commit_error = None;
                    self.current_view = RightPanelView::Banner;
                }
                KeyCode::Backspace => {
                    self.commit_message.pop();
                }
                KeyCode::Char(c) => {
                    self.commit_message.push(c);
                }
                _ => {}
            }
            return Ok((false, false));
        }

        // Normal key handling
        match key.code {
            KeyCode::Char('q') => return Ok((true, false)),
            KeyCode::Char('j') | KeyCode::Down => {
                // Navigate files
                if !self.files.is_empty() {
                    self.selected_file_index = (self.selected_file_index + 1) % self.files.len();
                    self.load_file_diff();
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                // Navigate files
                if !self.files.is_empty() {
                    self.selected_file_index =
                        (self.selected_file_index + self.files.len() - 1) % self.files.len();
                    self.load_file_diff();
                }
            }
            KeyCode::Char('d') if key.modifiers.contains(event::KeyModifiers::CONTROL) => {
                // Scroll diff down (half page)
                self.diff_scroll = self.diff_scroll.saturating_add(10);
            }
            KeyCode::Char('u') if key.modifiers.contains(event::KeyModifiers::CONTROL) => {
                // Scroll diff up (half page)
                self.diff_scroll = self.diff_scroll.saturating_sub(10);
            }
            KeyCode::PageDown => {
                self.diff_scroll = self.diff_scroll.saturating_add(20);
            }
            KeyCode::PageUp => {
                self.diff_scroll = self.diff_scroll.saturating_sub(20);
            }
            KeyCode::Enter => {
                // Open file content in right panel
                if !self.files.is_empty() {
                    self.load_file_content()?;
                    self.current_view = RightPanelView::FileContent;
                }
            }
            KeyCode::Esc => {
                // Go back to banner view
                self.current_view = RightPanelView::Banner;
                self.file_content = None;
            }
            KeyCode::Char('c') => {
                // Open commit modal
                self.commit_message.clear();
                self.commit_error = None;
                self.current_view = RightPanelView::CommitModal;
            }
            KeyCode::Char('a') => {
                // Stage file (git add)
                if !self.files.is_empty() {
                    let file = &self.files[self.selected_file_index];
                    let path = file.path.to_string_lossy();
                    if let Err(e) = stage_file(&path) {
                        eprintln!("Failed to stage file: {}", e);
                    }
                    return Ok((false, true)); // Refresh after staging
                }
            }
            KeyCode::Char('s') => {
                // Unstage file (git restore --staged)
                if !self.files.is_empty() {
                    let file = &self.files[self.selected_file_index];
                    let path = file.path.to_string_lossy();
                    if let Err(e) = unstage_file(&path) {
                        eprintln!("Failed to unstage file: {}", e);
                    }
                    return Ok((false, true)); // Refresh after unstaging
                }
            }
            KeyCode::Char('r') => {
                // Restore file (git restore)
                if !self.files.is_empty() {
                    let file = &self.files[self.selected_file_index];
                    let path = file.path.to_string_lossy();
                    if let Err(e) = restore_file(&path) {
                        eprintln!("Failed to restore file: {}", e);
                    }
                    return Ok((false, true)); // Refresh after restoring
                }
            }
            KeyCode::Char('A') => {
                // Stage all files (git add --all)
                if let Err(e) = stage_all() {
                    eprintln!("Failed to stage all files: {}", e);
                }
                return Ok((false, true)); // Refresh after staging all
            }
            KeyCode::Char('S') => {
                // Unstage all files (git restore --staged .)
                if let Err(e) = unstage_all() {
                    eprintln!("Failed to unstage all files: {}", e);
                }
                return Ok((false, true)); // Refresh after unstaging all
            }
            KeyCode::Char('R') => {
                // Restore all files (git restore .)
                if let Err(e) = restore_all() {
                    eprintln!("Failed to restore all files: {}", e);
                }
                return Ok((false, true)); // Refresh after restoring all
            }
            _ => {}
        }
        Ok((false, false))
    }

    fn render(&mut self, f: &mut Frame) {
        let area = f.size();

        // Split into left (file list) and right (content)
        let main_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(20), // Left file list
                Constraint::Percentage(80), // Right content
            ])
            .split(area);

        // Render left file list
        self.render_file_list(f, main_chunks[0]);

        // Render right content based on current view
        let right_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(0),    // Main content
                Constraint::Length(2), // Help (2 lines)
            ])
            .split(main_chunks[1]);

        match self.current_view {
            RightPanelView::Banner => self.render_banner_view(f, right_chunks[0]),
            RightPanelView::FileContent => self.render_file_content_view(f, right_chunks[0]),
            RightPanelView::CommitModal => {
                // Render solid background instead of content
                let bg = Block::default().style(Style::default().bg(Color::Black));
                f.render_widget(bg, right_chunks[0]);
                // Render modal on top
                self.render_commit_modal(f, area);
            }
        }

        // Render help
        self.render_help(f, right_chunks[1]);
    }

    fn render_file_list(&mut self, f: &mut Frame, area: Rect) {
        // Render background - same as code view in conflict manager
        let bg = Block::default().style(Style::default().bg(Color::Rgb(40, 40, 45)));
        f.render_widget(bg, area);

        if self.files.is_empty() {
            let empty = Paragraph::new("No changes")
                .style(Style::default().fg(Color::Gray))
                .alignment(Alignment::Center);
            f.render_widget(empty, area);
            return;
        }

        // Separate files into staged and unstaged
        let staged: Vec<&FileStatus> = self.files.iter().filter(|f| f.is_staged()).collect();
        let unstaged: Vec<&FileStatus> = self
            .files
            .iter()
            .filter(|f| f.is_modified_in_workdir() && !f.is_staged())
            .collect();
        let both: Vec<&FileStatus> = self
            .files
            .iter()
            .filter(|f| f.is_staged() && f.is_modified_in_workdir())
            .collect();

        let mut items: Vec<ListItem> = Vec::new();
        let mut current_index = 0;

        // Staged changes section
        if !staged.is_empty() {
            items.push(ListItem::new(Line::from(vec![Span::styled(
                "▎STAGED",
                Style::default()
                    .fg(MurasakiColors::SUCCESS)
                    .add_modifier(Modifier::BOLD),
            )])));
            for file in &staged {
                let status_display = file.display_status();
                let path = file.path.display().to_string();
                let line = format!("  [{}] {}", status_display, path);

                let style = if current_index == self.selected_file_index {
                    Style::default()
                        .fg(Color::Black)
                        .bg(MurasakiColors::SUCCESS)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(MurasakiColors::SUCCESS)
                };

                items.push(ListItem::new(line).style(style));
                current_index += 1;
            }
        }

        // Files with both staged and unstaged changes
        if !both.is_empty() {
            items.push(ListItem::new(Line::from(vec![Span::styled(
                "▎BOTH",
                Style::default()
                    .fg(MurasakiColors::WARNING)
                    .add_modifier(Modifier::BOLD),
            )])));
            for file in &both {
                let status_display = file.display_status();
                let path = file.path.display().to_string();
                let line = format!("  [{}] {}", status_display, path);

                let style = if current_index == self.selected_file_index {
                    Style::default()
                        .fg(Color::Black)
                        .bg(MurasakiColors::WARNING)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(MurasakiColors::WARNING)
                };

                items.push(ListItem::new(line).style(style));
                current_index += 1;
            }
        }

        // Unstaged changes section
        if !unstaged.is_empty() {
            items.push(ListItem::new(Line::from(vec![Span::styled(
                "▎UNSTAGED",
                Style::default()
                    .fg(MurasakiColors::ERROR)
                    .add_modifier(Modifier::BOLD),
            )])));
            for file in &unstaged {
                let status_display = file.display_status();
                let path = file.path.display().to_string();
                let line = format!("  [{}] {}", status_display, path);

                let style = if current_index == self.selected_file_index {
                    Style::default()
                        .fg(Color::Black)
                        .bg(MurasakiColors::ERROR)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(MurasakiColors::ERROR)
                };

                items.push(ListItem::new(line).style(style));
                current_index += 1;
            }
        }

        let mut state = ListState::default();
        state.select(Some(self.selected_file_index));

        let list = List::new(items).highlight_style(Style::default().add_modifier(Modifier::BOLD));

        f.render_stateful_widget(list, area, &mut state);
    }

    fn render_banner_view(&self, f: &mut Frame, area: Rect) {
        // If there's a diff to show, render it instead of the banner
        if let Some(ref diff) = self.file_diff {
            self.render_diff_view(f, area, diff);
            return;
        }

        // Create vertical centering layout
        let vertical_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(30), // Top padding
                Constraint::Min(10),        // Banner
                Constraint::Percentage(30), // Bottom padding
            ])
            .split(area);

        let banner = Paragraph::new(MURASAKI_BANNER)
            .style(
                Style::default()
                    .fg(Color::Magenta)
                    .add_modifier(Modifier::BOLD),
            )
            .alignment(Alignment::Center);
        f.render_widget(banner, vertical_chunks[1]);
    }

    fn render_diff_view(&self, f: &mut Frame, area: Rect, diff: &str) {
        // Render background
        let bg = Block::default().style(Style::default().bg(Color::Rgb(30, 30, 35)));
        f.render_widget(bg, area);

        // Get file name for header
        let file_name = if !self.files.is_empty() {
            self.files[self.selected_file_index]
                .path
                .display()
                .to_string()
        } else {
            "No file selected".to_string()
        };

        // Split into header and content
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(2), // Header
                Constraint::Min(0),    // Diff content
            ])
            .split(area);

        // Render header with file name
        let header = Paragraph::new(Line::from(vec![
            Span::styled(
                " Diff: ",
                Style::default()
                    .fg(Color::Magenta)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(file_name, Style::default().fg(Color::White)),
        ]))
        .style(Style::default().bg(Color::Rgb(50, 50, 55)));
        f.render_widget(header, chunks[0]);

        // Parse and render diff lines with colors
        let lines: Vec<Line> = diff
            .lines()
            .skip(self.diff_scroll)
            .map(|line| {
                if line.starts_with('+') && !line.starts_with("+++") {
                    Line::from(Span::styled(
                        line,
                        Style::default().fg(Color::Green).bg(Color::Rgb(20, 40, 20)),
                    ))
                } else if line.starts_with('-') && !line.starts_with("---") {
                    Line::from(Span::styled(
                        line,
                        Style::default().fg(Color::Red).bg(Color::Rgb(50, 20, 20)),
                    ))
                } else if line.starts_with("@@") {
                    Line::from(Span::styled(
                        line,
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    ))
                } else if line.starts_with("diff ") || line.starts_with("index ") {
                    Line::from(Span::styled(line, Style::default().fg(Color::Yellow)))
                } else if line.starts_with("---") || line.starts_with("+++") {
                    Line::from(Span::styled(
                        line,
                        Style::default()
                            .fg(Color::White)
                            .add_modifier(Modifier::BOLD),
                    ))
                } else {
                    Line::from(Span::styled(line, Style::default().fg(Color::Gray)))
                }
            })
            .collect();

        let diff_paragraph =
            Paragraph::new(lines).style(Style::default().bg(Color::Rgb(30, 30, 35)));
        f.render_widget(diff_paragraph, chunks[1]);
    }

    fn render_file_content_view(&self, f: &mut Frame, area: Rect) {
        let content_text = if let Some(ref content) = self.file_content {
            content.clone()
        } else {
            "Loading...".to_string()
        };

        let paragraph = Paragraph::new(content_text)
            .style(Style::default().fg(Color::White))
            .wrap(ratatui::widgets::Wrap { trim: false });

        f.render_widget(paragraph, area);
    }

    fn render_commit_modal(&self, f: &mut Frame, area: Rect) {
        // Split into sections (no borders, like rebase actions)
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Header
                Constraint::Length(1), // Input area (1 row for text and cursor)
                Constraint::Length(1), // Error message
                Constraint::Min(0),    // Spacer
                Constraint::Length(2), // Footer help
            ])
            .split(area);

        // Header - no borders
        let header = Paragraph::new(vec![
            Line::from(Span::styled(
                "Commit Changes",
                Style::default()
                    .fg(Color::Magenta)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "Enter your commit message:",
                Style::default().fg(Color::White),
            )),
        ])
        .alignment(Alignment::Center);
        f.render_widget(header, chunks[0]);

        // Input field - no borders
        let input_text = if self.commit_message.is_empty() {
            "> ".to_string()
        } else {
            format!("> {}", self.commit_message)
        };

        let input_style = Style::default().fg(Color::White);

        let input = Paragraph::new(input_text)
            .style(input_style)
            .alignment(Alignment::Center);
        f.render_widget(input, chunks[1]);

        // Render error message if present
        if let Some(ref error) = self.commit_error {
            let error_msg = Paragraph::new(error.as_str())
                .style(Style::default().fg(Color::Red))
                .alignment(Alignment::Center);
            f.render_widget(error_msg, chunks[2]);
        }

        // Footer - no borders
        let footer = Paragraph::new(Line::from(vec![
            Span::styled(
                "Enter",
                Style::default()
                    .fg(Color::Magenta)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("=Commit  ", Style::default().fg(Color::Gray)),
            Span::styled(
                "Esc",
                Style::default()
                    .fg(Color::Magenta)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("=Cancel", Style::default().fg(Color::Gray)),
        ]))
        .alignment(Alignment::Center);
        f.render_widget(footer, chunks[4]);
    }

    fn render_help(&self, f: &mut Frame, area: Rect) {
        // Split help text into 2 lines
        let (line1, line2) = match self.current_view {
            RightPanelView::Banner => (
                "j/k: Navigate | Ctrl+d/u: Scroll | a: Stage | s: Unstage | r: Restore",
                "A/S/R: All | c: Commit | Enter: View File | q: Quit",
            ),
            RightPanelView::FileContent => (
                "j/k: Navigate | Esc: Back | a: Stage | s: Unstage | r: Restore",
                "A/S/R: All | c: Commit | q: Quit",
            ),
            RightPanelView::CommitModal => ("Type message | Enter: Commit | Esc: Cancel", ""),
        };

        let help = Paragraph::new(vec![
            Line::from(Span::styled(line1, Style::default().fg(Color::White))),
            Line::from(Span::styled(line2, Style::default().fg(Color::White))),
        ])
        .style(Style::default().bg(Color::Rgb(60, 60, 70))) // FOOTER_BG background
        .alignment(Alignment::Center);
        f.render_widget(help, area);
    }
}

pub fn run_status_view(repo: &Repository) -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, cursor::Hide)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let files = get_repository_status(repo)?;
    let mut view = StatusView::new(files);
    let mut should_quit = false;

    while !should_quit {
        terminal.draw(|f| {
            view.render(f);

            // Position cursor for commit modal
            if view.current_view == RightPanelView::CommitModal {
                let area = f.size();
                // Recalculate the modal layout to get input chunk position
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Length(3), // Header
                        Constraint::Length(1), // Input area (1 row for text and cursor)
                        Constraint::Length(1), // Error message
                        Constraint::Min(0),    // Spacer
                        Constraint::Length(2), // Footer help
                    ])
                    .split(area);

                let input_chunk = chunks[1];
                // Input area is now 1 row, so text and cursor are on the same line
                let input_y = input_chunk.y;

                // Calculate X position to match Paragraph center alignment exactly
                let prompt_len = 2; // "> "
                let text_len = view.commit_message.len() as u16;
                let total_text_len = prompt_len + text_len;
                // Paragraph centers text: text starts at x + (width - text_len) / 2
                // Cursor should be at the end of the text: start + text_len
                let text_start_x =
                    input_chunk.x + (input_chunk.width.saturating_sub(total_text_len)) / 2;
                let cursor_x = text_start_x + total_text_len;

                f.set_cursor(cursor_x, input_y);
            }
        })?;

        // Show/hide cursor based on view
        if view.current_view == RightPanelView::CommitModal {
            execute!(terminal.backend_mut(), cursor::Show)?;
        } else {
            execute!(terminal.backend_mut(), cursor::Hide)?;
        }

        if let Event::Key(key) = event::read()? {
            let (quit, refresh) = view.handle_key(key)?;
            should_quit = quit;

            if refresh {
                // Reload files from git status
                let updated_files = get_repository_status(repo)?;
                view.files = updated_files;
                // Adjust selected index if needed
                if view.selected_file_index >= view.files.len() && !view.files.is_empty() {
                    view.selected_file_index = view.files.len() - 1;
                }
                // Reload diff for current file
                view.load_file_diff();
            }
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}
