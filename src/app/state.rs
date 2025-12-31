use crate::domain::{ConflictedFile, GitOperation, Resolution};

/// Represents the current view mode in the application
#[derive(Debug, Clone, PartialEq)]
pub enum ViewMode {
    /// Showing the list of conflicted files
    FileList,
    /// Resolving conflicts in a specific file
    ConflictResolve {
        file_index: usize,
        conflict_index: usize,
    },
    /// Showing rebase actions (continue/abort/skip)
    RebaseActions,
}

/// Main application state
pub struct AppState {
    /// All conflicted files
    pub files: Vec<ConflictedFile>,
    /// Current view mode
    pub view_mode: ViewMode,
    /// Currently selected file index (in FileList view)
    pub selected_file: usize,
    /// The type of git operation in progress
    pub git_operation: GitOperation,
    /// Whether the application should quit
    pub should_quit: bool,
}

impl AppState {
    /// Create a new application state
    pub fn new(files: Vec<ConflictedFile>, git_operation: GitOperation) -> Self {
        Self {
            files,
            view_mode: ViewMode::FileList,
            selected_file: 0,
            git_operation,
            should_quit: false,
        }
    }

    /// Move selection up in file list
    pub fn move_selection_up(&mut self) {
        if let ViewMode::FileList = self.view_mode {
            if self.selected_file > 0 {
                self.selected_file -= 1;
            }
        }
    }

    /// Move selection down in file list
    pub fn move_selection_down(&mut self) {
        if let ViewMode::FileList = self.view_mode {
            if self.selected_file < self.files.len().saturating_sub(1) {
                self.selected_file += 1;
            }
        }
    }

    /// Open the selected file for conflict resolution
    pub fn open_selected_file(&mut self) {
        if let ViewMode::FileList = self.view_mode {
            if self.selected_file < self.files.len() {
                self.view_mode = ViewMode::ConflictResolve {
                    file_index: self.selected_file,
                    conflict_index: 0,
                };
            }
        }
    }

    /// Move to the next conflict in the current file
    pub fn next_conflict(&mut self) {
        if let ViewMode::ConflictResolve {
            file_index,
            conflict_index,
        } = &mut self.view_mode
        {
            let file = &self.files[*file_index];
            if *conflict_index < file.conflicts.len().saturating_sub(1) {
                *conflict_index += 1;
            }
        }
    }

    /// Move to the previous conflict in the current file
    pub fn previous_conflict(&mut self) {
        if let ViewMode::ConflictResolve {
            conflict_index, ..
        } = &mut self.view_mode
        {
            if *conflict_index > 0 {
                *conflict_index -= 1;
            }
        }
    }

    /// Set resolution for the current conflict
    pub fn set_current_resolution(&mut self, resolution: Resolution) {
        if let ViewMode::ConflictResolve {
            file_index,
            conflict_index,
        } = self.view_mode
        {
            if file_index < self.files.len() {
                self.files[file_index].set_resolution(conflict_index, resolution);
            }
        }
    }

    /// Go back to file list view
    pub fn back_to_file_list(&mut self) {
        self.view_mode = ViewMode::FileList;
    }

    /// Go to rebase actions view
    pub fn go_to_rebase_actions(&mut self) {
        self.view_mode = ViewMode::RebaseActions;
    }

    /// Check if all files are fully resolved
    pub fn all_files_resolved(&self) -> bool {
        self.files.iter().all(|f| f.is_fully_resolved())
    }

    /// Get the current file being edited (if in ConflictResolve mode)
    pub fn current_file(&self) -> Option<&ConflictedFile> {
        if let ViewMode::ConflictResolve { file_index, .. } = self.view_mode {
            self.files.get(file_index)
        } else {
            None
        }
    }

    /// Get the current file being edited (mutable)
    pub fn current_file_mut(&mut self) -> Option<&mut ConflictedFile> {
        if let ViewMode::ConflictResolve { file_index, .. } = self.view_mode {
            self.files.get_mut(file_index)
        } else {
            None
        }
    }

    /// Get the current conflict index
    pub fn current_conflict_index(&self) -> Option<usize> {
        if let ViewMode::ConflictResolve { conflict_index, .. } = self.view_mode {
            Some(conflict_index)
        } else {
            None
        }
    }

    /// Mark the application to quit
    pub fn quit(&mut self) {
        self.should_quit = true;
    }
}
