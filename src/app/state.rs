use crate::domain::{ConflictedFile, GitOperation, Resolution};
use crate::git::FileStatus;

/// Represents the application mode
#[derive(Debug, Clone, PartialEq)]
pub enum AppMode {
    /// Resolving conflicts during merge/rebase
    Conflict,
    /// Staging/unstaging files (no conflicts)
    Staging,
}

/// Represents the current view mode in the application
#[derive(Debug, Clone, PartialEq)]
pub enum ViewMode {
    /// Split-pane view: file list on left, code view on right
    SplitPane {
        /// Currently selected conflict index (for navigation)
        conflict_index: usize,
    },
    /// Showing rebase actions (continue/abort/skip)
    RebaseActions,
}

/// Focus in split-pane view
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PaneFocus {
    FileList,
    CodeView,
}

/// Main application state
pub struct AppState {
    /// All conflicted files
    pub files: Vec<ConflictedFile>,
    /// Current view mode
    pub view_mode: ViewMode,
    /// Currently selected file index
    pub selected_file: usize,
    /// Current focus (file list or code view)
    pub focus: PaneFocus,
    /// The type of git operation in progress
    pub git_operation: GitOperation,
    /// Whether the application should quit
    pub should_quit: bool,
    /// Vertical scroll offset for code view
    pub scroll_offset: u16,
    /// Whether to show the help dialog
    pub show_help: bool,
    /// Application mode (Conflict or Staging)
    pub mode: AppMode,
    /// Status of files for staging mode
    pub file_statuses: Vec<FileStatus>,
    /// Diff content for the selected file
    pub diff_content: Option<String>,
    /// Commit message for the commit modal
    pub commit_message: String,
    /// Whether to show the commit modal
    pub show_commit_modal: bool,
    /// Commit error message (if any)
    pub commit_error: Option<String>,
}

impl AppState {
    /// Create a new application state for conflict resolution mode
    pub fn new(files: Vec<ConflictedFile>, git_operation: GitOperation) -> Self {
        Self {
            files,
            view_mode: ViewMode::SplitPane { conflict_index: 0 },
            selected_file: 0,
            focus: PaneFocus::FileList,
            git_operation,
            should_quit: false,
            scroll_offset: 0,
            show_help: false,
            mode: AppMode::Conflict,
            file_statuses: Vec::new(),
            diff_content: None,
            commit_message: String::new(),
            show_commit_modal: false,
            commit_error: None,
        }
    }

    /// Create a new application state for staging mode (no conflicts)
    pub fn new_staging(file_statuses: Vec<FileStatus>) -> Self {
        Self {
            files: Vec::new(),
            view_mode: ViewMode::SplitPane { conflict_index: 0 },
            selected_file: 0,
            focus: PaneFocus::FileList,
            git_operation: GitOperation::None,
            should_quit: false,
            scroll_offset: 0,
            show_help: false,
            mode: AppMode::Staging,
            file_statuses,
            diff_content: None,
            commit_message: String::new(),
            show_commit_modal: false,
            commit_error: None,
        }
    }

    /// Toggle the help dialog
    pub fn toggle_help(&mut self) {
        self.show_help = !self.show_help;
    }

    /// Toggle focus between file list and code view
    pub fn toggle_focus(&mut self) {
        self.focus = match self.focus {
            PaneFocus::FileList => PaneFocus::CodeView,
            PaneFocus::CodeView => PaneFocus::FileList,
        };
    }

    /// Move selection up in file list
    pub fn move_selection_up(&mut self) {
        if self.focus == PaneFocus::FileList && self.selected_file > 0 {
            self.selected_file -= 1;
            // Reset conflict index and scroll when changing files
            if let ViewMode::SplitPane { conflict_index } = &mut self.view_mode {
                *conflict_index = 0;
            }
            self.reset_scroll();
        }
    }

    /// Move selection down in file list
    pub fn move_selection_down(&mut self) {
        if self.focus == PaneFocus::FileList
            && self.selected_file < self.files.len().saturating_sub(1)
        {
            self.selected_file += 1;
            // Reset conflict index and scroll when changing files
            if let ViewMode::SplitPane { conflict_index } = &mut self.view_mode {
                *conflict_index = 0;
            }
            self.reset_scroll();
        }
    }

    /// Move to the next conflict in the current file
    pub fn next_conflict(&mut self) {
        if self.focus == PaneFocus::CodeView {
            if let ViewMode::SplitPane { conflict_index } = &mut self.view_mode {
                if self.selected_file < self.files.len() {
                    let file = &self.files[self.selected_file];
                    if *conflict_index < file.conflicts.len().saturating_sub(1) {
                        *conflict_index += 1;
                    }
                }
            }
        }
    }

    /// Move to the previous conflict in the current file
    pub fn previous_conflict(&mut self) {
        if self.focus == PaneFocus::CodeView {
            if let ViewMode::SplitPane { conflict_index } = &mut self.view_mode {
                if *conflict_index > 0 {
                    *conflict_index -= 1;
                }
            }
        }
    }

    /// Set resolution for the current conflict
    pub fn set_current_resolution(&mut self, resolution: Resolution) {
        if self.focus == PaneFocus::CodeView {
            if let ViewMode::SplitPane { conflict_index } = self.view_mode {
                if self.selected_file < self.files.len() {
                    self.files[self.selected_file].set_resolution(conflict_index, resolution);
                }
            }
        }
    }

    /// Clear resolution for the current conflict (undo)
    pub fn clear_current_resolution(&mut self) {
        if self.focus == PaneFocus::CodeView {
            if let ViewMode::SplitPane { conflict_index } = self.view_mode {
                if self.selected_file < self.files.len() {
                    self.files[self.selected_file].clear_resolution(conflict_index);
                }
            }
        }
    }

    /// Go back to file list focus
    pub fn back_to_file_list(&mut self) {
        self.focus = PaneFocus::FileList;
    }

    /// Go to rebase actions view
    pub fn go_to_rebase_actions(&mut self) {
        self.view_mode = ViewMode::RebaseActions;
    }

    /// Check if all files are fully resolved
    pub fn all_files_resolved(&self) -> bool {
        self.files.iter().all(|f| f.is_fully_resolved())
    }

    /// Get the currently selected file
    pub fn current_file(&self) -> Option<&ConflictedFile> {
        self.files.get(self.selected_file)
    }

    /// Get the currently selected file (mutable)
    pub fn current_file_mut(&mut self) -> Option<&mut ConflictedFile> {
        self.files.get_mut(self.selected_file)
    }

    /// Get the current conflict index
    pub fn current_conflict_index(&self) -> Option<usize> {
        if let ViewMode::SplitPane { conflict_index } = self.view_mode {
            Some(conflict_index)
        } else {
            None
        }
    }

    /// Save the current file
    pub fn save_current_file(&mut self) -> bool {
        if let Some(file) = self.current_file() {
            file.is_fully_resolved()
        } else {
            false
        }
    }

    /// Mark the application to quit
    pub fn quit(&mut self) {
        self.should_quit = true;
    }

    /// Scroll down in code view
    pub fn scroll_down(&mut self) {
        self.scroll_offset = self.scroll_offset.saturating_add(1);
    }

    /// Scroll up in code view
    pub fn scroll_up(&mut self) {
        self.scroll_offset = self.scroll_offset.saturating_sub(1);
    }

    /// Reset scroll when changing files or conflicts
    pub fn reset_scroll(&mut self) {
        self.scroll_offset = 0;
    }

    // --- Staging mode methods ---

    /// Check if we're in staging mode
    pub fn is_staging_mode(&self) -> bool {
        self.mode == AppMode::Staging
    }

    /// Check if we're in conflict mode
    pub fn is_conflict_mode(&self) -> bool {
        self.mode == AppMode::Conflict
    }

    /// Get current file status in staging mode
    pub fn current_file_status(&self) -> Option<&FileStatus> {
        self.file_statuses.get(self.selected_file)
    }

    /// Toggle commit modal
    pub fn toggle_commit_modal(&mut self) {
        self.show_commit_modal = !self.show_commit_modal;
        if !self.show_commit_modal {
            self.commit_message.clear();
            self.commit_error = None;
        }
    }

    /// Open commit modal
    pub fn open_commit_modal(&mut self) {
        self.show_commit_modal = true;
        self.commit_message.clear();
        self.commit_error = None;
    }

    /// Close commit modal
    pub fn close_commit_modal(&mut self) {
        self.show_commit_modal = false;
        self.commit_message.clear();
        self.commit_error = None;
    }

    /// Set commit error message
    pub fn set_commit_error(&mut self, error: String) {
        self.commit_error = Some(error);
    }

    /// Update file statuses (after staging/unstaging)
    pub fn update_file_statuses(&mut self, statuses: Vec<FileStatus>) {
        self.file_statuses = statuses;
        // Adjust selection if needed
        if self.selected_file >= self.file_statuses.len() && !self.file_statuses.is_empty() {
            self.selected_file = self.file_statuses.len() - 1;
        }
    }

    /// Get total file count based on mode
    pub fn total_files(&self) -> usize {
        if self.is_staging_mode() {
            self.file_statuses.len()
        } else {
            self.files.len()
        }
    }

    /// Check if current file has conflicts (in either mode)
    pub fn current_file_has_conflicts(&self) -> bool {
        if self.is_conflict_mode() {
            if let Some(file) = self.current_file() {
                return !file.conflicts.is_empty();
            }
        }
        false
    }

    /// Move selection up - updated to work with both modes
    pub fn move_selection_up_unified(&mut self) {
        if self.focus == PaneFocus::FileList {
            let max = self.total_files();
            if max > 0 && self.selected_file > 0 {
                self.selected_file -= 1;
                // Reset conflict index and scroll when changing files
                if let ViewMode::SplitPane { conflict_index } = &mut self.view_mode {
                    *conflict_index = 0;
                }
                self.reset_scroll();
            }
        }
    }

    /// Move selection down - updated to work with both modes
    pub fn move_selection_down_unified(&mut self) {
        if self.focus == PaneFocus::FileList {
            let max = self.total_files();
            if max > 0 && self.selected_file < max.saturating_sub(1) {
                self.selected_file += 1;
                // Reset conflict index and scroll when changing files
                if let ViewMode::SplitPane { conflict_index } = &mut self.view_mode {
                    *conflict_index = 0;
                }
                self.reset_scroll();
            }
        }
    }

    /// Transition from conflict mode to staging mode after all conflicts resolved
    pub fn transition_to_staging(&mut self, file_statuses: Vec<FileStatus>) {
        self.mode = AppMode::Staging;
        self.file_statuses = file_statuses;
        self.files.clear();
        self.selected_file = 0;
        self.reset_scroll();
    }

    /// Check if there are any staged files
    pub fn has_staged_files(&self) -> bool {
        self.file_statuses.iter().any(|f| f.is_staged())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::ConflictHunk;
    use crate::git::StatusChange;
    use std::path::PathBuf;

    // Helper functions to create test data
    fn create_test_file_status(path: &str, staged: bool, modified: bool) -> FileStatus {
        FileStatus {
            path: PathBuf::from(path),
            index_status: if staged {
                Some(StatusChange::Modified)
            } else {
                None
            },
            workdir_status: if modified {
                Some(StatusChange::Modified)
            } else {
                None
            },
        }
    }

    fn create_test_conflicted_file(path: &str) -> ConflictedFile {
        let conflict = ConflictHunk {
            start_line: 0,
            end_line: 5,
            current: "current content".to_string(),
            incoming: "incoming content".to_string(),
        };
        ConflictedFile::new(
            PathBuf::from(path),
            vec![conflict],
            "test content".to_string(),
        )
    }

    // --- Staging Mode Tests ---

    #[test]
    fn test_new_staging_creates_staging_mode() {
        let statuses = vec![
            create_test_file_status("file1.rs", true, false),
            create_test_file_status("file2.rs", false, true),
        ];
        let state = AppState::new_staging(statuses);

        assert!(state.is_staging_mode());
        assert!(!state.is_conflict_mode());
        assert_eq!(state.mode, AppMode::Staging);
        assert_eq!(state.file_statuses.len(), 2);
        assert!(state.files.is_empty());
        assert_eq!(state.git_operation, GitOperation::None);
    }

    #[test]
    fn test_new_conflict_creates_conflict_mode() {
        let files = vec![create_test_conflicted_file("conflict.rs")];
        let state = AppState::new(files, GitOperation::Merge);

        assert!(state.is_conflict_mode());
        assert!(!state.is_staging_mode());
        assert_eq!(state.mode, AppMode::Conflict);
        assert_eq!(state.files.len(), 1);
        assert!(state.file_statuses.is_empty());
    }

    #[test]
    fn test_total_files_in_staging_mode() {
        let statuses = vec![
            create_test_file_status("file1.rs", true, false),
            create_test_file_status("file2.rs", false, true),
            create_test_file_status("file3.rs", true, true),
        ];
        let state = AppState::new_staging(statuses);

        assert_eq!(state.total_files(), 3);
    }

    #[test]
    fn test_total_files_in_conflict_mode() {
        let files = vec![
            create_test_conflicted_file("file1.rs"),
            create_test_conflicted_file("file2.rs"),
        ];
        let state = AppState::new(files, GitOperation::Merge);

        assert_eq!(state.total_files(), 2);
    }

    #[test]
    fn test_current_file_status() {
        let statuses = vec![
            create_test_file_status("file1.rs", true, false),
            create_test_file_status("file2.rs", false, true),
        ];
        let state = AppState::new_staging(statuses);

        let current = state.current_file_status().unwrap();
        assert_eq!(current.path, PathBuf::from("file1.rs"));
    }

    #[test]
    fn test_has_staged_files_true() {
        let statuses = vec![
            create_test_file_status("file1.rs", true, false),
            create_test_file_status("file2.rs", false, true),
        ];
        let state = AppState::new_staging(statuses);

        assert!(state.has_staged_files());
    }

    #[test]
    fn test_has_staged_files_false() {
        let statuses = vec![
            create_test_file_status("file1.rs", false, true),
            create_test_file_status("file2.rs", false, true),
        ];
        let state = AppState::new_staging(statuses);

        assert!(!state.has_staged_files());
    }

    // --- Commit Modal Tests ---

    #[test]
    fn test_open_commit_modal() {
        let mut state = AppState::new_staging(vec![]);
        state.commit_message = "old message".to_string();
        state.commit_error = Some("old error".to_string());

        state.open_commit_modal();

        assert!(state.show_commit_modal);
        assert!(state.commit_message.is_empty());
        assert!(state.commit_error.is_none());
    }

    #[test]
    fn test_close_commit_modal() {
        let mut state = AppState::new_staging(vec![]);
        state.show_commit_modal = true;
        state.commit_message = "test message".to_string();
        state.commit_error = Some("error".to_string());

        state.close_commit_modal();

        assert!(!state.show_commit_modal);
        assert!(state.commit_message.is_empty());
        assert!(state.commit_error.is_none());
    }

    #[test]
    fn test_toggle_commit_modal() {
        let mut state = AppState::new_staging(vec![]);

        state.toggle_commit_modal();
        assert!(state.show_commit_modal);

        state.commit_message = "test".to_string();
        state.toggle_commit_modal();
        assert!(!state.show_commit_modal);
        assert!(state.commit_message.is_empty());
    }

    #[test]
    fn test_set_commit_error() {
        let mut state = AppState::new_staging(vec![]);
        state.set_commit_error("Commit failed".to_string());

        assert_eq!(state.commit_error, Some("Commit failed".to_string()));
    }

    // --- Navigation Tests (Staging Mode) ---

    #[test]
    fn test_move_selection_down_unified() {
        let statuses = vec![
            create_test_file_status("file1.rs", true, false),
            create_test_file_status("file2.rs", false, true),
            create_test_file_status("file3.rs", true, true),
        ];
        let mut state = AppState::new_staging(statuses);
        assert_eq!(state.selected_file, 0);

        state.move_selection_down_unified();
        assert_eq!(state.selected_file, 1);

        state.move_selection_down_unified();
        assert_eq!(state.selected_file, 2);

        // Should not go beyond last file
        state.move_selection_down_unified();
        assert_eq!(state.selected_file, 2);
    }

    #[test]
    fn test_move_selection_up_unified() {
        let statuses = vec![
            create_test_file_status("file1.rs", true, false),
            create_test_file_status("file2.rs", false, true),
        ];
        let mut state = AppState::new_staging(statuses);
        state.selected_file = 1;

        state.move_selection_up_unified();
        assert_eq!(state.selected_file, 0);

        // Should not go below 0
        state.move_selection_up_unified();
        assert_eq!(state.selected_file, 0);
    }

    #[test]
    fn test_navigation_requires_file_list_focus() {
        let statuses = vec![
            create_test_file_status("file1.rs", true, false),
            create_test_file_status("file2.rs", false, true),
        ];
        let mut state = AppState::new_staging(statuses);
        state.focus = PaneFocus::CodeView;

        state.move_selection_down_unified();
        assert_eq!(state.selected_file, 0); // Should not change
    }

    // --- File Status Update Tests ---

    #[test]
    fn test_update_file_statuses() {
        let initial = vec![create_test_file_status("file1.rs", true, false)];
        let mut state = AppState::new_staging(initial);

        let new_statuses = vec![
            create_test_file_status("file2.rs", false, true),
            create_test_file_status("file3.rs", true, true),
        ];
        state.update_file_statuses(new_statuses);

        assert_eq!(state.file_statuses.len(), 2);
    }

    #[test]
    fn test_update_file_statuses_adjusts_selection() {
        let initial = vec![
            create_test_file_status("file1.rs", true, false),
            create_test_file_status("file2.rs", false, true),
            create_test_file_status("file3.rs", true, true),
        ];
        let mut state = AppState::new_staging(initial);
        state.selected_file = 2;

        // Reduce to 1 file
        let new_statuses = vec![create_test_file_status("file1.rs", true, false)];
        state.update_file_statuses(new_statuses);

        assert_eq!(state.selected_file, 0);
    }

    // --- Mode Transition Tests ---

    #[test]
    fn test_transition_to_staging() {
        let files = vec![create_test_conflicted_file("conflict.rs")];
        let mut state = AppState::new(files, GitOperation::Merge);
        state.selected_file = 0;
        state.scroll_offset = 10;

        let new_statuses = vec![create_test_file_status("file1.rs", true, false)];
        state.transition_to_staging(new_statuses);

        assert!(state.is_staging_mode());
        assert!(state.files.is_empty());
        assert_eq!(state.file_statuses.len(), 1);
        assert_eq!(state.selected_file, 0);
        assert_eq!(state.scroll_offset, 0);
    }

    // --- Focus and Help Tests ---

    #[test]
    fn test_toggle_focus() {
        let mut state = AppState::new_staging(vec![]);
        assert_eq!(state.focus, PaneFocus::FileList);

        state.toggle_focus();
        assert_eq!(state.focus, PaneFocus::CodeView);

        state.toggle_focus();
        assert_eq!(state.focus, PaneFocus::FileList);
    }

    #[test]
    fn test_toggle_help() {
        let mut state = AppState::new_staging(vec![]);
        assert!(!state.show_help);

        state.toggle_help();
        assert!(state.show_help);

        state.toggle_help();
        assert!(!state.show_help);
    }

    // --- Scroll Tests ---

    #[test]
    fn test_scroll_down() {
        let mut state = AppState::new_staging(vec![]);
        assert_eq!(state.scroll_offset, 0);

        state.scroll_down();
        assert_eq!(state.scroll_offset, 1);

        state.scroll_down();
        assert_eq!(state.scroll_offset, 2);
    }

    #[test]
    fn test_scroll_up() {
        let mut state = AppState::new_staging(vec![]);
        state.scroll_offset = 5;

        state.scroll_up();
        assert_eq!(state.scroll_offset, 4);

        // Should not go below 0
        state.scroll_offset = 0;
        state.scroll_up();
        assert_eq!(state.scroll_offset, 0);
    }

    #[test]
    fn test_reset_scroll() {
        let mut state = AppState::new_staging(vec![]);
        state.scroll_offset = 10;

        state.reset_scroll();
        assert_eq!(state.scroll_offset, 0);
    }

    // --- Quit Test ---

    #[test]
    fn test_quit() {
        let mut state = AppState::new_staging(vec![]);
        assert!(!state.should_quit);

        state.quit();
        assert!(state.should_quit);
    }

    // --- Diff Content Tests ---

    #[test]
    fn test_diff_content_initially_none() {
        let state = AppState::new_staging(vec![]);
        assert!(state.diff_content.is_none());
    }

    #[test]
    fn test_diff_content_can_be_set() {
        let mut state = AppState::new_staging(vec![]);
        state.diff_content = Some("diff --git a/file.rs".to_string());

        assert_eq!(state.diff_content, Some("diff --git a/file.rs".to_string()));
    }
}
