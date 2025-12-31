use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use std::time::Duration;

use crate::app::{AppState, ViewMode};
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
        ViewMode::FileList => handle_file_list_keys(state, key),
        ViewMode::ConflictResolve { .. } => handle_conflict_resolve_keys(state, key),
        ViewMode::RebaseActions => handle_rebase_actions_keys(state, key),
    }
}

fn handle_file_list_keys(state: &mut AppState, key: KeyEvent) -> Result<()> {
    match key.code {
        KeyCode::Char('q') | KeyCode::Esc => {
            state.quit();
        }
        KeyCode::Char('j') | KeyCode::Down => {
            state.move_selection_down();
        }
        KeyCode::Char('k') | KeyCode::Up => {
            state.move_selection_up();
        }
        KeyCode::Enter => {
            state.open_selected_file();
        }
        _ => {}
    }
    Ok(())
}

fn handle_conflict_resolve_keys(state: &mut AppState, key: KeyEvent) -> Result<()> {
    match key.code {
        KeyCode::Char('q') => {
            state.quit();
        }
        KeyCode::Esc => {
            state.back_to_file_list();
        }
        KeyCode::Char('j') | KeyCode::Down => {
            state.next_conflict();
        }
        KeyCode::Char('k') | KeyCode::Up => {
            state.previous_conflict();
        }
        KeyCode::Char('c') => {
            state.set_current_resolution(Resolution::Current);
        }
        KeyCode::Char('i') => {
            state.set_current_resolution(Resolution::Incoming);
        }
        KeyCode::Char('b') => {
            state.set_current_resolution(Resolution::Both);
        }
        KeyCode::Char('s') => {
            // Save current file
            if let Some(file) = state.current_file() {
                if file.is_fully_resolved() {
                    apply_resolutions(file)?;
                    state.back_to_file_list();

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
