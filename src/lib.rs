pub mod app;
pub mod domain;
pub mod git;
pub mod tui;
pub mod version;

pub use app::{AppMode, AppState, ViewMode};
pub use domain::{ConflictHunk, ConflictedFile, GitOperation, Resolution};
pub use git::{detect_git_operation, find_conflicted_files, parse_conflicts};
pub use tui::run_app;
pub use version::{check_for_updates, UpdateInfo};
