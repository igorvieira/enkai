use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use std::time::Duration;

use crate::app::{AppState, PaneFocus, ViewMode};
use crate::domain::Resolution;
use crate::git::{abort_rebase, apply_resolutions, continue_rebase, skip_rebase};

/// Handle keyboard events
pub fn handle_events(state: &mut AppState) -> Result<()> {
    if event::poll(Duration::from_millis(100))? {
        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                handle_key_event(state, key)?;
            }
        }
    }
    Ok(())
}

fn handle_key_event(state: &mut AppState, key: KeyEvent) -> Result<()> {
    match state.view_mode {
        ViewMode::SplitPane { .. } => handle_split_pane_keys(state, key),
        ViewMode::RebaseActions => handle_rebase_actions_keys(state, key),
    }
}

fn handle_split_pane_keys(state: &mut AppState, key: KeyEvent) -> Result<()> {
    // Global keys that work regardless of focus
    match key.code {
        KeyCode::Char('q') => {
            state.quit();
            return Ok(());
        }
        KeyCode::Tab => {
            state.toggle_focus();
            return Ok(());
        }
        _ => {}
    }

    // Keys specific to current focus
    match state.focus {
        PaneFocus::FileList => handle_file_list_focus_keys(state, key),
        PaneFocus::CodeView => handle_code_view_focus_keys(state, key),
    }
}

fn handle_file_list_focus_keys(state: &mut AppState, key: KeyEvent) -> Result<()> {
    match key.code {
        KeyCode::Char('j') | KeyCode::Down => {
            state.move_selection_down();
        }
        KeyCode::Char('k') | KeyCode::Up => {
            state.move_selection_up();
        }
        _ => {}
    }
    Ok(())
}

fn handle_code_view_focus_keys(state: &mut AppState, key: KeyEvent) -> Result<()> {
    // Handle Ctrl+d and Ctrl+u for scrolling (half page)
    if key.modifiers.contains(KeyModifiers::CONTROL) {
        match key.code {
            KeyCode::Char('d') => {
                // Scroll down (half page)
                for _ in 0..10 {
                    state.scroll_down();
                }
                return Ok(());
            }
            KeyCode::Char('u') => {
                // Scroll up (half page)
                for _ in 0..10 {
                    state.scroll_up();
                }
                return Ok(());
            }
            _ => {}
        }
    }

    match key.code {
        KeyCode::Char('j') | KeyCode::Down => {
            // Scroll down one line
            state.scroll_down();
        }
        KeyCode::Char('k') | KeyCode::Up => {
            // Scroll up one line
            state.scroll_up();
        }
        KeyCode::Char('n') => {
            // Navigate to next conflict in current file
            state.next_conflict();
        }
        KeyCode::Char('p') => {
            // Navigate to previous conflict in current file
            state.previous_conflict();
        }
        KeyCode::Char('c') => {
            // Set resolution for current conflict
            state.set_current_resolution(Resolution::Current);
        }
        KeyCode::Char('i') => {
            // Set resolution for current conflict
            state.set_current_resolution(Resolution::Incoming);
        }
        KeyCode::Char('b') => {
            // Set resolution for current conflict
            state.set_current_resolution(Resolution::Both);
        }
        KeyCode::Char('u') => {
            // Clear resolution for current conflict (undo)
            state.clear_current_resolution();
        }
        KeyCode::Char('s') => {
            // Save file (only if all conflicts resolved)
            if let Some(file) = state.current_file() {
                if file.is_fully_resolved() {
                    apply_resolutions(file)?;

                    // If all files are resolved and it's a rebase, show rebase actions
                    if state.all_files_resolved() && state.git_operation.is_rebase() {
                        state.go_to_rebase_actions();
                    }
                }
            }
        }
        _ => {}
    }
    Ok(())
}

fn handle_rebase_actions_keys(state: &mut AppState, key: KeyEvent) -> Result<()> {
    match key.code {
        KeyCode::Char('q') | KeyCode::Esc => {
            state.quit();
        }
        KeyCode::Char('c') => {
            continue_rebase()?;
            state.quit();
        }
        KeyCode::Char('a') => {
            abort_rebase()?;
            state.quit();
        }
        KeyCode::Char('s') => {
            skip_rebase()?;
            state.quit();
        }
        _ => {}
    }
    Ok(())
}
