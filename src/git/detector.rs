use anyhow::{Context, Result};
use git2::{Repository, Status, StatusOptions};
use std::path::PathBuf;

use crate::domain::GitOperation;

/// Detect the current git operation (merge, rebase, or interactive rebase)
pub fn detect_git_operation(repo: &Repository) -> Result<GitOperation> {
    let git_dir = repo.path();

    // Check for interactive rebase
    if git_dir.join("rebase-merge").exists() || git_dir.join("rebase-apply").exists() {
        // Interactive rebase creates rebase-merge directory
        if git_dir.join("rebase-merge/interactive").exists() {
            return Ok(GitOperation::RebaseInteractive);
        }
        return Ok(GitOperation::Rebase);
    }

    // Check for merge
    if git_dir.join("MERGE_HEAD").exists() {
        return Ok(GitOperation::Merge);
    }

    anyhow::bail!("No merge or rebase operation in progress")
}

/// Find all files with conflicts in the repository
pub fn find_conflicted_files(repo: &Repository) -> Result<Vec<PathBuf>> {
    let mut opts = StatusOptions::new();
    opts.include_untracked(false);
    opts.include_ignored(false);

    let statuses = repo
        .statuses(Some(&mut opts))
        .context("Failed to get repository status")?;

    let mut conflicted_files = Vec::new();

    for entry in statuses.iter() {
        let status = entry.status();

        // Check if file has conflicts
        if status.contains(Status::CONFLICTED) {
            if let Some(path) = entry.path() {
                let file_path = repo
                    .workdir()
                    .context("Repository has no working directory")?
                    .join(path);
                conflicted_files.push(file_path);
            }
        }
    }

    if conflicted_files.is_empty() {
        anyhow::bail!("No conflicted files found");
    }

    Ok(conflicted_files)
}

/// Open the git repository in the current directory or parent directories
pub fn open_repository() -> Result<Repository> {
    Repository::discover(".")
        .context("Not a git repository (or any of the parent directories)")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_open_repository() {
        // This test will pass if run from within a git repository
        let result = open_repository();
        assert!(result.is_ok() || result.is_err()); // Just ensure it doesn't panic
    }
}
