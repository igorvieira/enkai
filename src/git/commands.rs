use anyhow::{Context, Result};
use std::process::Command;

/// Continue the rebase after resolving conflicts
pub fn continue_rebase() -> Result<()> {
    let output = Command::new("git")
        .args(["rebase", "--continue"])
        .output()
        .context("Failed to execute git rebase --continue")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("git rebase --continue failed: {}", stderr);
    }

    Ok(())
}

/// Abort the rebase
pub fn abort_rebase() -> Result<()> {
    let output = Command::new("git")
        .args(["rebase", "--abort"])
        .output()
        .context("Failed to execute git rebase --abort")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("git rebase --abort failed: {}", stderr);
    }

    Ok(())
}

/// Skip the current commit in the rebase
pub fn skip_rebase() -> Result<()> {
    let output = Command::new("git")
        .args(["rebase", "--skip"])
        .output()
        .context("Failed to execute git rebase --skip")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("git rebase --skip failed: {}", stderr);
    }

    Ok(())
}

/// Stage a file (git add)
pub fn stage_file(path: &str) -> Result<()> {
    let output = Command::new("git")
        .args(["add", path])
        .output()
        .context("Failed to execute git add")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("git add failed: {}", stderr);
    }

    Ok(())
}

/// Unstage a file (git restore --staged)
pub fn unstage_file(path: &str) -> Result<()> {
    let output = Command::new("git")
        .args(["restore", "--staged", path])
        .output()
        .context("Failed to execute git restore --staged")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("git restore --staged failed: {}", stderr);
    }

    Ok(())
}

/// Restore a file to last committed state (git restore)
pub fn restore_file(path: &str) -> Result<()> {
    let output = Command::new("git")
        .args(["restore", path])
        .output()
        .context("Failed to execute git restore")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("git restore failed: {}", stderr);
    }

    Ok(())
}

/// Stage all files (git add --all)
pub fn stage_all() -> Result<()> {
    let output = Command::new("git")
        .args(["add", "--all"])
        .output()
        .context("Failed to execute git add --all")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("git add --all failed: {}", stderr);
    }

    Ok(())
}

/// Unstage all files (git restore --staged .)
pub fn unstage_all() -> Result<()> {
    let output = Command::new("git")
        .args(["restore", "--staged", "."])
        .output()
        .context("Failed to execute git restore --staged .")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("git restore --staged . failed: {}", stderr);
    }

    Ok(())
}

/// Restore all files to last committed state (git restore .)
pub fn restore_all() -> Result<()> {
    let output = Command::new("git")
        .args(["restore", "."])
        .output()
        .context("Failed to execute git restore .")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("git restore . failed: {}", stderr);
    }

    Ok(())
}

/// Create a git commit with the given message
pub fn commit_changes(message: &str) -> Result<()> {
    let output = Command::new("git")
        .args(["commit", "-m", message])
        .output()
        .context("Failed to execute git commit")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("git commit failed: {}", stderr);
    }

    Ok(())
}

/// Get the diff for a file (staged or unstaged)
pub fn get_file_diff(path: &str, staged: bool) -> Result<String> {
    let mut args = vec!["diff"];
    if staged {
        args.push("--cached");
    }
    args.push("--");
    args.push(path);

    let output = Command::new("git")
        .args(&args)
        .output()
        .context("Failed to execute git diff")?;

    let diff = String::from_utf8_lossy(&output.stdout).to_string();
    Ok(diff)
}
