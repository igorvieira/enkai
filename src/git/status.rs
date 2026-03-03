use anyhow::{Context, Result};
use git2::{Repository, Status, StatusOptions};
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct FileStatus {
    pub path: PathBuf,
    pub index_status: Option<StatusChange>,
    pub workdir_status: Option<StatusChange>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum StatusChange {
    Modified,
    Added,
    Deleted,
    Renamed,
    Untracked,
    Conflicted,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FileStatusType {
    Modified,
    Added,
    Deleted,
    Renamed,
    Untracked,
    Conflicted,
}

impl StatusChange {
    pub fn icon(&self) -> &str {
        match self {
            StatusChange::Modified => "M",
            StatusChange::Added => "A",
            StatusChange::Deleted => "D",
            StatusChange::Renamed => "R",
            StatusChange::Untracked => "?",
            StatusChange::Conflicted => "C",
        }
    }
}

impl FileStatus {
    pub fn display_status(&self) -> String {
        let index_icon = self.index_status.as_ref().map(|s| s.icon()).unwrap_or(" ");
        let workdir_icon = self
            .workdir_status
            .as_ref()
            .map(|s| s.icon())
            .unwrap_or(" ");
        format!("{}{}", index_icon, workdir_icon)
    }

    pub fn is_staged(&self) -> bool {
        self.index_status.is_some()
    }

    pub fn is_modified_in_workdir(&self) -> bool {
        self.workdir_status.is_some()
    }

    pub fn is_conflicted(&self) -> bool {
        matches!(self.index_status, Some(StatusChange::Conflicted))
            || matches!(self.workdir_status, Some(StatusChange::Conflicted))
    }
}

impl FileStatusType {
    pub fn icon(&self) -> &str {
        match self {
            FileStatusType::Modified => "M",
            FileStatusType::Added => "A",
            FileStatusType::Deleted => "D",
            FileStatusType::Renamed => "R",
            FileStatusType::Untracked => "?",
            FileStatusType::Conflicted => "C",
        }
    }

    pub fn color(&self) -> &str {
        match self {
            FileStatusType::Modified => "\x1b[33m",   // Yellow
            FileStatusType::Added => "\x1b[32m",      // Green
            FileStatusType::Deleted => "\x1b[31m",    // Red
            FileStatusType::Renamed => "\x1b[36m",    // Cyan
            FileStatusType::Untracked => "\x1b[37m",  // White
            FileStatusType::Conflicted => "\x1b[35m", // Magenta
        }
    }
}

pub fn get_repository_status(repo: &Repository) -> Result<Vec<FileStatus>> {
    let mut opts = StatusOptions::new();
    opts.include_untracked(true);
    opts.recurse_untracked_dirs(false);

    let statuses = repo
        .statuses(Some(&mut opts))
        .context("Failed to get repository status")?;

    let mut file_statuses = Vec::new();

    for entry in statuses.iter() {
        let path = PathBuf::from(entry.path().context("Invalid UTF-8 in path")?);
        let status = entry.status();

        // Determine index (staged) status
        let index_status = if status.contains(Status::CONFLICTED) {
            Some(StatusChange::Conflicted)
        } else if status.contains(Status::INDEX_NEW) {
            Some(StatusChange::Added)
        } else if status.contains(Status::INDEX_DELETED) {
            Some(StatusChange::Deleted)
        } else if status.contains(Status::INDEX_RENAMED) {
            Some(StatusChange::Renamed)
        } else if status.contains(Status::INDEX_MODIFIED) {
            Some(StatusChange::Modified)
        } else {
            None
        };

        // Determine workdir (unstaged) status
        let workdir_status = if status.contains(Status::WT_NEW) {
            Some(StatusChange::Untracked)
        } else if status.contains(Status::WT_DELETED) {
            Some(StatusChange::Deleted)
        } else if status.contains(Status::WT_RENAMED) {
            Some(StatusChange::Renamed)
        } else if status.contains(Status::WT_MODIFIED) {
            Some(StatusChange::Modified)
        } else {
            None
        };

        // Only add if there's at least one status
        if index_status.is_some() || workdir_status.is_some() {
            file_statuses.push(FileStatus {
                path,
                index_status,
                workdir_status,
            });
        }
    }

    Ok(file_statuses)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_status_change_icons() {
        assert_eq!(StatusChange::Modified.icon(), "M");
        assert_eq!(StatusChange::Added.icon(), "A");
        assert_eq!(StatusChange::Deleted.icon(), "D");
        assert_eq!(StatusChange::Renamed.icon(), "R");
        assert_eq!(StatusChange::Untracked.icon(), "?");
        assert_eq!(StatusChange::Conflicted.icon(), "C");
    }

    #[test]
    fn test_file_status_type_icons() {
        assert_eq!(FileStatusType::Modified.icon(), "M");
        assert_eq!(FileStatusType::Added.icon(), "A");
        assert_eq!(FileStatusType::Deleted.icon(), "D");
        assert_eq!(FileStatusType::Renamed.icon(), "R");
        assert_eq!(FileStatusType::Untracked.icon(), "?");
        assert_eq!(FileStatusType::Conflicted.icon(), "C");
    }

    #[test]
    fn test_file_status_display_staged_only() {
        let status = FileStatus {
            path: PathBuf::from("test.rs"),
            index_status: Some(StatusChange::Modified),
            workdir_status: None,
        };
        assert_eq!(status.display_status(), "M ");
    }

    #[test]
    fn test_file_status_display_unstaged_only() {
        let status = FileStatus {
            path: PathBuf::from("test.rs"),
            index_status: None,
            workdir_status: Some(StatusChange::Modified),
        };
        assert_eq!(status.display_status(), " M");
    }

    #[test]
    fn test_file_status_display_both() {
        let status = FileStatus {
            path: PathBuf::from("test.rs"),
            index_status: Some(StatusChange::Added),
            workdir_status: Some(StatusChange::Modified),
        };
        assert_eq!(status.display_status(), "AM");
    }

    #[test]
    fn test_file_status_is_staged() {
        let staged = FileStatus {
            path: PathBuf::from("test.rs"),
            index_status: Some(StatusChange::Modified),
            workdir_status: None,
        };
        assert!(staged.is_staged());

        let unstaged = FileStatus {
            path: PathBuf::from("test.rs"),
            index_status: None,
            workdir_status: Some(StatusChange::Modified),
        };
        assert!(!unstaged.is_staged());
    }

    #[test]
    fn test_file_status_is_modified_in_workdir() {
        let modified = FileStatus {
            path: PathBuf::from("test.rs"),
            index_status: None,
            workdir_status: Some(StatusChange::Modified),
        };
        assert!(modified.is_modified_in_workdir());

        let staged_only = FileStatus {
            path: PathBuf::from("test.rs"),
            index_status: Some(StatusChange::Modified),
            workdir_status: None,
        };
        assert!(!staged_only.is_modified_in_workdir());
    }

    #[test]
    fn test_file_status_is_conflicted_index() {
        let conflicted = FileStatus {
            path: PathBuf::from("test.rs"),
            index_status: Some(StatusChange::Conflicted),
            workdir_status: None,
        };
        assert!(conflicted.is_conflicted());
    }

    #[test]
    fn test_file_status_is_conflicted_workdir() {
        let conflicted = FileStatus {
            path: PathBuf::from("test.rs"),
            index_status: None,
            workdir_status: Some(StatusChange::Conflicted),
        };
        assert!(conflicted.is_conflicted());
    }

    #[test]
    fn test_file_status_not_conflicted() {
        let normal = FileStatus {
            path: PathBuf::from("test.rs"),
            index_status: Some(StatusChange::Modified),
            workdir_status: Some(StatusChange::Modified),
        };
        assert!(!normal.is_conflicted());
    }

    #[test]
    fn test_file_status_type_colors() {
        // Just verify that colors return valid ANSI escape codes
        assert!(FileStatusType::Modified.color().starts_with("\x1b["));
        assert!(FileStatusType::Added.color().starts_with("\x1b["));
        assert!(FileStatusType::Deleted.color().starts_with("\x1b["));
        assert!(FileStatusType::Renamed.color().starts_with("\x1b["));
        assert!(FileStatusType::Untracked.color().starts_with("\x1b["));
        assert!(FileStatusType::Conflicted.color().starts_with("\x1b["));
    }
}
