use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use git2::Repository;
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame, Terminal,
};
use std::io;

use crate::git::{
    get_repository_status, restore_file, stage_file, unstage_file,
    restore_all, stage_all, unstage_all, commit_changes, FileStatus,
};

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
        Self {
            files,
            selected_file_index: 0,
            current_view: RightPanelView::Banner,
            file_content: None,
            commit_message: String::new(),
            commit_error: None,
        }
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
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                // Navigate files
                if !self.files.is_empty() {
                    self.selected_file_index =
                        (self.selected_file_index + self.files.len() - 1) % self.files.len();
                }
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
                Constraint::Percentage(20),  // Left file list
                Constraint::Percentage(80),  // Right content
            ])
            .split(area);

        // Render left file list
        self.render_file_list(f, main_chunks[0]);

        // Render right content based on current view
        let right_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(0),      // Main content
                Constraint::Length(3),   // Help
            ])
            .split(main_chunks[1]);

        match self.current_view {
            RightPanelView::Banner => self.render_banner_view(f, right_chunks[0]),
            RightPanelView::FileContent => self.render_file_content_view(f, right_chunks[0]),
            RightPanelView::CommitModal => {
                // Still render banner/content behind modal
                self.render_banner_view(f, right_chunks[0]);
                // Render modal on top
                self.render_commit_modal(f, area);
            }
        }

        // Render help
        self.render_help(f, right_chunks[1]);
    }

    fn render_file_list(&mut self, f: &mut Frame, area: Rect) {
        if self.files.is_empty() {
            let empty = Paragraph::new("No changes")
                .style(Style::default().fg(Color::Gray))
                .alignment(Alignment::Center)
                .block(
                    Block::default()
                        .title("Changed Files")
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(Color::Magenta)),
                );
            f.render_widget(empty, area);
            return;
        }

        // Separate files into staged and unstaged
        let staged: Vec<&FileStatus> = self.files.iter().filter(|f| f.is_staged()).collect();
        let unstaged: Vec<&FileStatus> = self.files.iter().filter(|f| f.is_modified_in_workdir() && !f.is_staged()).collect();
        let both: Vec<&FileStatus> = self.files.iter().filter(|f| f.is_staged() && f.is_modified_in_workdir()).collect();

        let mut items: Vec<ListItem> = Vec::new();
        let mut current_index = 0;

        // Staged changes section
        if !staged.is_empty() {
            items.push(ListItem::new("Staged:").style(Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)));
            for file in &staged {
                let status_display = file.display_status();
                let path = file.path.display().to_string();
                let line = format!(" [{}] {}", status_display, path);

                let style = if current_index == self.selected_file_index {
                    Style::default()
                        .fg(Color::Black)
                        .bg(Color::Green)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::Green)
                };

                items.push(ListItem::new(line).style(style));
                current_index += 1;
            }
        }

        // Files with both staged and unstaged changes
        if !both.is_empty() {
            items.push(ListItem::new("Both:").style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)));
            for file in &both {
                let status_display = file.display_status();
                let path = file.path.display().to_string();
                let line = format!(" [{}] {}", status_display, path);

                let style = if current_index == self.selected_file_index {
                    Style::default()
                        .fg(Color::Black)
                        .bg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::Yellow)
                };

                items.push(ListItem::new(line).style(style));
                current_index += 1;
            }
        }

        // Unstaged changes section
        if !unstaged.is_empty() {
            items.push(ListItem::new("Unstaged:").style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)));
            for file in &unstaged {
                let status_display = file.display_status();
                let path = file.path.display().to_string();
                let line = format!(" [{}] {}", status_display, path);

                let style = if current_index == self.selected_file_index {
                    Style::default()
                        .fg(Color::Black)
                        .bg(Color::Red)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::Red)
                };

                items.push(ListItem::new(line).style(style));
                current_index += 1;
            }
        }

        let mut state = ListState::default();
        state.select(Some(self.selected_file_index));

        let title = format!("Changed Files ({})", self.files.len());

        let list = List::new(items)
            .block(
                Block::default()
                    .title(title)
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Magenta)),
            )
            .highlight_style(
                Style::default()
                    .add_modifier(Modifier::BOLD),
            );

        f.render_stateful_widget(list, area, &mut state);
    }

    fn render_banner_view(&self, f: &mut Frame, area: Rect) {
        // Create vertical centering layout
        let vertical_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(30),  // Top padding
                Constraint::Min(10),         // Banner
                Constraint::Percentage(30),  // Bottom padding
            ])
            .split(area);

        let banner = Paragraph::new(MURASAKI_BANNER)
            .style(Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD))
            .alignment(Alignment::Center);
        f.render_widget(banner, vertical_chunks[1]);
    }

    fn render_file_content_view(&self, f: &mut Frame, area: Rect) {
        let file = &self.files[self.selected_file_index];
        let title = format!("File: {}", file.path.display());

        let content_text = if let Some(ref content) = self.file_content {
            content.clone()
        } else {
            "Loading...".to_string()
        };

        let paragraph = Paragraph::new(content_text)
            .style(Style::default().fg(Color::White))
            .block(
                Block::default()
                    .title(title)
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Cyan)),
            )
            .wrap(ratatui::widgets::Wrap { trim: false });

        f.render_widget(paragraph, area);
    }

    fn render_commit_modal(&self, f: &mut Frame, area: Rect) {
        // Create centered modal
        let modal_width = 60;
        let modal_height = 9;

        let horizontal_margin = (area.width.saturating_sub(modal_width)) / 2;
        let vertical_margin = (area.height.saturating_sub(modal_height)) / 2;

        let modal_area = Rect {
            x: area.x + horizontal_margin,
            y: area.y + vertical_margin,
            width: modal_width.min(area.width),
            height: modal_height.min(area.height),
        };

        // Render background overlay with padding
        let bg_padding = 2;
        let background_area = Rect {
            x: modal_area.x.saturating_sub(bg_padding),
            y: modal_area.y.saturating_sub(bg_padding),
            width: modal_area.width + (bg_padding * 2),
            height: modal_area.height + (bg_padding * 2),
        };

        let background = Block::default()
            .style(Style::default().bg(Color::DarkGray));
        f.render_widget(background, background_area);

        // Clear modal area with darker background
        let modal_bg = Block::default()
            .style(Style::default().bg(Color::Black));
        f.render_widget(modal_bg, modal_area);

        // Create input field text
        let input_text = if self.commit_message.is_empty() {
            "Enter commit message...".to_string()
        } else {
            self.commit_message.clone()
        };

        let input_style = if self.commit_message.is_empty() {
            Style::default().fg(Color::DarkGray)
        } else {
            Style::default().fg(Color::White)
        };

        // Split modal into sections
        let modal_chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([
                Constraint::Length(3),  // Input field
                Constraint::Length(1),  // Error message
                Constraint::Length(1),  // Help text
            ])
            .split(modal_area);

        // Render input field
        let input = Paragraph::new(input_text)
            .style(input_style)
            .block(
                Block::default()
                    .title("Commit Message")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Magenta)),
            );
        f.render_widget(input, modal_chunks[0]);

        // Render error message if present
        if let Some(ref error) = self.commit_error {
            let error_msg = Paragraph::new(error.as_str())
                .style(Style::default().fg(Color::Red));
            f.render_widget(error_msg, modal_chunks[1]);
        }

        // Render help text
        let help = Paragraph::new("Enter: Commit | Esc: Cancel")
            .style(Style::default().fg(Color::Gray))
            .alignment(Alignment::Center);
        f.render_widget(help, modal_chunks[2]);

        // Render modal border
        let modal_block = Block::default()
            .title("Commit Changes")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD))
            .style(Style::default().bg(Color::Black));
        f.render_widget(modal_block, modal_area);
    }

    fn render_help(&self, f: &mut Frame, area: Rect) {
        let help_text = match self.current_view {
            RightPanelView::Banner => "j/k: Navigate | Enter: View File | a: Stage | s: Unstage | r: Restore | A/S/R: All | c: Commit | q: Quit",
            RightPanelView::FileContent => "j/k: Navigate | Esc: Back | a: Stage | s: Unstage | r: Restore | A/S/R: All | c: Commit | q: Quit",
            RightPanelView::CommitModal => "Type message | Enter: Commit | Esc: Cancel",
        };

        let help = Paragraph::new(help_text)
            .style(Style::default().fg(Color::Gray))
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::DarkGray)),
            );
        f.render_widget(help, area);
    }
}

pub fn run_status_view(repo: &Repository) -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let files = get_repository_status(repo)?;
    let mut view = StatusView::new(files);
    let mut should_quit = false;

    while !should_quit {
        terminal.draw(|f| view.render(f))?;

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
            }
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}
