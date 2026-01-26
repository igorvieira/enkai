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
    restore_all, stage_all, unstage_all, FileStatus,
};

const MURASAKI_BANNER: &str = r#"

           ███╗   ███╗██╗   ██╗██████╗  █████╗ ███████╗ █████╗ ██╗  ██╗██╗
           ████╗ ████║██║   ██║██╔══██╗██╔══██╗██╔════╝██╔══██╗██║ ██╔╝██║
           ██╔████╔██║██║   ██║██████╔╝███████║███████╗███████║█████╔╝ ██║
           ██║╚██╔╝██║██║   ██║██╔══██╗██╔══██║╚════██║██╔══██║██╔═██╗ ██║
           ██║ ╚═╝ ██║╚██████╔╝██║  ██║██║  ██║███████║██║  ██║██║  ██╗██║
           ╚═╝     ╚═╝ ╚═════╝ ╚═╝  ╚═╝╚═╝  ╚═╝╚══════╝╚═╝  ╚═╝╚═╝  ╚═╝╚═╝

"#;

#[derive(Debug, PartialEq)]
enum MenuOption {
    ViewChanges,
    Quit,
}

pub struct StatusView {
    files: Vec<FileStatus>,
    selected_file_index: usize,
    menu_selected: usize,
    current_view: RightPanelView,
}

#[derive(Debug, PartialEq)]
enum RightPanelView {
    Banner,
    FileList,
}

impl StatusView {
    pub fn new(files: Vec<FileStatus>) -> Self {
        Self {
            files,
            selected_file_index: 0,
            menu_selected: 0,
            current_view: RightPanelView::Banner,
        }
    }

    fn menu_options() -> Vec<MenuOption> {
        vec![
            MenuOption::ViewChanges,
            MenuOption::Quit,
        ]
    }

    fn menu_option_text(option: &MenuOption) -> &str {
        match option {
            MenuOption::ViewChanges => "View Changes",
            MenuOption::Quit => "Quit",
        }
    }

    fn handle_key(&mut self, key: KeyEvent) -> Result<(bool, bool)> {
        // Returns (should_quit, should_refresh)
        match key.code {
            KeyCode::Char('q') => return Ok((true, false)),
            KeyCode::Char('j') | KeyCode::Down => {
                match self.current_view {
                    RightPanelView::Banner => {
                        // Navigate menu
                        let options_len = Self::menu_options().len();
                        self.menu_selected = (self.menu_selected + 1) % options_len;
                    }
                    RightPanelView::FileList => {
                        // Navigate files
                        if !self.files.is_empty() {
                            self.selected_file_index = (self.selected_file_index + 1) % self.files.len();
                        }
                    }
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                match self.current_view {
                    RightPanelView::Banner => {
                        // Navigate menu
                        let options_len = Self::menu_options().len();
                        self.menu_selected = (self.menu_selected + options_len - 1) % options_len;
                    }
                    RightPanelView::FileList => {
                        // Navigate files
                        if !self.files.is_empty() {
                            self.selected_file_index =
                                (self.selected_file_index + self.files.len() - 1) % self.files.len();
                        }
                    }
                }
            }
            KeyCode::Enter => {
                if self.current_view == RightPanelView::Banner {
                    let options = Self::menu_options();
                    match &options[self.menu_selected] {
                        MenuOption::ViewChanges => {
                            self.current_view = RightPanelView::FileList;
                        }
                        MenuOption::Quit => return Ok((true, false)),
                    }
                }
            }
            KeyCode::Esc => {
                // Go back to banner view
                self.current_view = RightPanelView::Banner;
            }
            KeyCode::Char('a') => {
                // Stage file (git add)
                if self.current_view == RightPanelView::FileList && !self.files.is_empty() {
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
                if self.current_view == RightPanelView::FileList && !self.files.is_empty() {
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
                if self.current_view == RightPanelView::FileList && !self.files.is_empty() {
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
                if self.current_view == RightPanelView::FileList {
                    if let Err(e) = stage_all() {
                        eprintln!("Failed to stage all files: {}", e);
                    }
                    return Ok((false, true)); // Refresh after staging all
                }
            }
            KeyCode::Char('S') => {
                // Unstage all files (git restore --staged .)
                if self.current_view == RightPanelView::FileList {
                    if let Err(e) = unstage_all() {
                        eprintln!("Failed to unstage all files: {}", e);
                    }
                    return Ok((false, true)); // Refresh after unstaging all
                }
            }
            KeyCode::Char('R') => {
                // Restore all files (git restore .)
                if self.current_view == RightPanelView::FileList {
                    if let Err(e) = restore_all() {
                        eprintln!("Failed to restore all files: {}", e);
                    }
                    return Ok((false, true)); // Refresh after restoring all
                }
            }
            _ => {}
        }
        Ok((false, false))
    }

    fn render(&mut self, f: &mut Frame) {
        let area = f.size();

        // Split into left (menu) and right (content)
        let main_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(20),  // Left menu
                Constraint::Percentage(80),  // Right content
            ])
            .split(area);

        // Render left menu
        self.render_left_menu(f, main_chunks[0]);

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
            RightPanelView::FileList => self.render_file_list_view(f, right_chunks[0]),
        }

        // Render help
        self.render_help(f, right_chunks[1]);
    }

    fn render_left_menu(&self, f: &mut Frame, area: Rect) {
        let options = Self::menu_options();

        let title = if self.files.is_empty() {
            "Menu - No Changes".to_string()
        } else {
            format!("Menu - {} Changes", self.files.len())
        };

        let items: Vec<ListItem> = options
            .iter()
            .enumerate()
            .map(|(i, opt)| {
                let text = Self::menu_option_text(opt);
                let style = if i == self.menu_selected && self.current_view == RightPanelView::Banner {
                    Style::default()
                        .fg(Color::Black)
                        .bg(Color::Magenta)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::White)
                };
                ListItem::new(text).style(style)
            })
            .collect();

        let list = List::new(items)
            .block(
                Block::default()
                    .title(title)
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Magenta)),
            )
            .highlight_style(
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Magenta)
                    .add_modifier(Modifier::BOLD),
            );

        f.render_widget(list, area);
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

    fn render_file_list_view(&mut self, f: &mut Frame, area: Rect) {
        if self.files.is_empty() {
            let empty = Paragraph::new("No files to display")
                .style(Style::default().fg(Color::Gray))
                .alignment(Alignment::Center)
                .block(
                    Block::default()
                        .title("Changed Files")
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(Color::Cyan)),
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
            items.push(ListItem::new("Staged Changes:").style(Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)));
            for file in &staged {
                let status_display = file.display_status();
                let path = file.path.display().to_string();
                let line = format!("  [{}] {}", status_display, path);

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
            items.push(ListItem::new("Staged + Unstaged Changes:").style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)));
            for file in &both {
                let status_display = file.display_status();
                let path = file.path.display().to_string();
                let line = format!("  [{}] {}", status_display, path);

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
            items.push(ListItem::new("Unstaged Changes:").style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)));
            for file in &unstaged {
                let status_display = file.display_status();
                let path = file.path.display().to_string();
                let line = format!("  [{}] {}", status_display, path);

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

        let title = format!("Changed Files (Staged: {}, Unstaged: {}, Both: {})",
                           staged.len(), unstaged.len(), both.len());

        let list = List::new(items)
            .block(
                Block::default()
                    .title(title)
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Cyan)),
            )
            .highlight_style(
                Style::default()
                    .add_modifier(Modifier::BOLD),
            );

        f.render_stateful_widget(list, area, &mut state);
    }

    fn render_help(&self, f: &mut Frame, area: Rect) {
        let help_text = match self.current_view {
            RightPanelView::Banner => "j/k: Navigate Menu | Enter: Select | q: Quit",
            RightPanelView::FileList => "j/k: Navigate | a: Stage | A: Stage All | s: Unstage | S: Unstage All | r: Restore | R: Restore All | Esc: Back | q: Quit",
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
