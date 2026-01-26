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
