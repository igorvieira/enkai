use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use std::time::Duration;

use crate::app::{AppState, PaneFocus, ViewMode};
use crate::domain::Resolution;
use crate::git::{
    abort_rebase, apply_resolutions, commit_changes, continue_rebase, get_file_diff,
    get_repository_status, restore_file, skip_rebase, stage_file, unstage_file,
};

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
    // Handle commit modal first (highest priority)
    if state.show_commit_modal {
        return handle_commit_modal_keys(state, key);
    }

    match state.view_mode {
        ViewMode::SplitPane { .. } => {
            if state.is_staging_mode() {
                handle_staging_keys(state, key)
            } else {
                handle_split_pane_keys(state, key)
            }
        }
        ViewMode::RebaseActions => handle_rebase_actions_keys(state, key),
    }
}

fn handle_commit_modal_keys(state: &mut AppState, key: KeyEvent) -> Result<()> {
    match key.code {
        KeyCode::Enter => {
            // Submit commit
            if !state.commit_message.is_empty() {
                match commit_changes(&state.commit_message) {
                    Ok(_) => {
                        state.close_commit_modal();
                        // Refresh file statuses after commit
                        refresh_file_statuses(state)?;
                    }
                    Err(e) => {
                        state.set_commit_error(format!("Commit failed: {}", e));
                    }
                }
            }
        }
        KeyCode::Esc => {
            state.close_commit_modal();
        }
        KeyCode::Backspace => {
            state.commit_message.pop();
        }
        KeyCode::Char(c) => {
            state.commit_message.push(c);
        }
        _ => {}
    }
    Ok(())
}

fn handle_staging_keys(state: &mut AppState, key: KeyEvent) -> Result<()> {
    // If help dialog is open, only handle Esc or ? to close it
    if state.show_help {
        match key.code {
            KeyCode::Esc | KeyCode::Char('?') | KeyCode::Char('q') => {
                state.toggle_help();
            }
            _ => {}
        }
        return Ok(());
    }

    // Global keys
    match key.code {
        KeyCode::Char('q') => {
            state.quit();
            return Ok(());
        }
        KeyCode::Char('?') => {
            state.toggle_help();
            return Ok(());
        }
        KeyCode::Tab => {
            state.toggle_focus();
            // Load diff when focusing on code view
            if state.focus == PaneFocus::CodeView {
                load_current_file_diff(state)?;
            }
            return Ok(());
        }
        _ => {}
    }

    // Handle Ctrl+d and Ctrl+u for scrolling
    if key.modifiers.contains(KeyModifiers::CONTROL) {
        match key.code {
            KeyCode::Char('d') => {
                for _ in 0..10 {
                    state.scroll_down();
                }
                return Ok(());
            }
            KeyCode::Char('u') => {
                for _ in 0..10 {
                    state.scroll_up();
                }
                return Ok(());
            }
            _ => {}
        }
    }

    // Staging-specific keys
    match key.code {
        KeyCode::Char('j') | KeyCode::Down => {
            if state.focus == PaneFocus::FileList {
                state.move_selection_down_unified();
                load_current_file_diff(state)?;
            } else {
                state.scroll_down();
            }
        }
        KeyCode::Char('k') | KeyCode::Up => {
            if state.focus == PaneFocus::FileList {
                state.move_selection_up_unified();
                load_current_file_diff(state)?;
            } else {
                state.scroll_up();
            }
        }
        KeyCode::Char('a') => {
            // Stage file
            if let Some(file_status) = state.current_file_status() {
                let path = file_status.path.to_string_lossy().to_string();
                if let Err(e) = stage_file(&path) {
                    eprintln!("Failed to stage file: {}", e);
                } else {
                    refresh_file_statuses(state)?;
                }
            }
        }
        KeyCode::Char('s') => {
            // Unstage file
            if let Some(file_status) = state.current_file_status() {
                let path = file_status.path.to_string_lossy().to_string();
                if let Err(e) = unstage_file(&path) {
                    eprintln!("Failed to unstage file: {}", e);
                } else {
                    refresh_file_statuses(state)?;
                }
            }
        }
        KeyCode::Char('r') => {
            // Restore file (discard changes)
            if let Some(file_status) = state.current_file_status() {
                let path = file_status.path.to_string_lossy().to_string();
                if let Err(e) = restore_file(&path) {
                    eprintln!("Failed to restore file: {}", e);
                } else {
                    refresh_file_statuses(state)?;
                }
            }
        }
        KeyCode::Char('c') => {
            // Open commit modal (only if there are staged files)
            if state.has_staged_files() {
                state.open_commit_modal();
            }
        }
        _ => {}
    }

    Ok(())
}

fn handle_split_pane_keys(state: &mut AppState, key: KeyEvent) -> Result<()> {
    // If help dialog is open, only handle Esc or ? to close it
    if state.show_help {
        match key.code {
            KeyCode::Esc | KeyCode::Char('?') | KeyCode::Char('q') => {
                state.toggle_help();
            }
            _ => {}
        }
        return Ok(());
    }

    // Global keys that work regardless of focus
    match key.code {
        KeyCode::Char('q') => {
            state.quit();
            return Ok(());
        }
        KeyCode::Char('?') => {
            state.toggle_help();
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
            // Auto-save after resolution
            auto_save_if_resolved(state)?;
        }
        KeyCode::Char('i') => {
            // Set resolution for current conflict
            state.set_current_resolution(Resolution::Incoming);
            // Auto-save after resolution
            auto_save_if_resolved(state)?;
        }
        KeyCode::Char('b') => {
            // Set resolution for current conflict
            state.set_current_resolution(Resolution::Both);
            // Auto-save after resolution
            auto_save_if_resolved(state)?;
        }
        KeyCode::Char('u') => {
            // Clear resolution for current conflict (undo)
            state.clear_current_resolution();
        }
        _ => {}
    }
    Ok(())
}

fn auto_save_if_resolved(state: &mut AppState) -> Result<()> {
    if let Some(file) = state.current_file() {
        if file.is_fully_resolved() {
            apply_resolutions(file)?;

            // If all files are resolved and it's a rebase, show rebase actions
            if state.all_files_resolved() && state.git_operation.is_rebase() {
                state.go_to_rebase_actions();
            }
        }
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

fn load_current_file_diff(state: &mut AppState) -> Result<()> {
    if state.file_statuses.is_empty() {
        state.diff_content = None;
        return Ok(());
    }

    if let Some(file_status) = state.current_file_status() {
        let path = file_status.path.to_string_lossy();
        let staged = file_status.is_staged() && !file_status.is_modified_in_workdir();

        match get_file_diff(&path, staged) {
            Ok(diff) => {
                if diff.is_empty() {
                    // If no diff (e.g., new untracked file), try to read file content
                    if let Ok(content) = std::fs::read_to_string(&file_status.path) {
                        state.diff_content = Some(format!("New file:\n\n{}", content));
                    } else {
                        state.diff_content = Some("No changes to display".to_string());
                    }
                } else {
                    state.diff_content = Some(diff);
                }
            }
            Err(e) => {
                state.diff_content = Some(format!("Error getting diff: {}", e));
            }
        }
        state.reset_scroll();
    }

    Ok(())
}

fn refresh_file_statuses(state: &mut AppState) -> Result<()> {
    // Open repository and get updated statuses
    let repo = crate::git::detector::open_repository()?;
    let statuses = get_repository_status(&repo)?;
    state.update_file_statuses(statuses);
    load_current_file_diff(state)?;
    Ok(())
}
