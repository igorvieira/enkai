use crate::domain::{ConflictedFile, GitOperation, Resolution};

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
}

impl AppState {
    /// Create a new application state
    pub fn new(files: Vec<ConflictedFile>, git_operation: GitOperation) -> Self {
        Self {
            files,
            view_mode: ViewMode::SplitPane { conflict_index: 0 },
            selected_file: 0,
            focus: PaneFocus::FileList,
            git_operation,
            should_quit: false,
            scroll_offset: 0,
        }
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
}
