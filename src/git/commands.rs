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
