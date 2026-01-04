use anyhow::{Context, Result};
use std::fs;

use crate::domain::ConflictedFile;

/// Apply resolutions to a conflicted file and save it
pub fn apply_resolutions(conflicted_file: &ConflictedFile) -> Result<()> {
    if !conflicted_file.is_fully_resolved() {
        anyhow::bail!(
            "Cannot apply resolutions: not all conflicts are resolved ({}/{} resolved)",
            conflicted_file.resolved_count(),
            conflicted_file.total_conflicts()
        );
    }

    let content = &conflicted_file.original_content;
    let lines: Vec<&str> = content.lines().collect();
    let mut result_lines = Vec::new();
    let mut current_line = 0;

    for (i, conflict) in conflicted_file.conflicts.iter().enumerate() {
        // Add lines before this conflict with safe indexing
        while current_line < conflict.start_line {
            let line = lines.get(current_line).ok_or_else(|| {
                anyhow::anyhow!(
                    "Internal error: line index {} out of bounds (total lines: {})",
                    current_line,
                    lines.len()
                )
            })?;
            result_lines.push(line.to_string());
            current_line += 1;
        }

        // Add resolved content with safe indexing
        let resolution = conflicted_file.resolutions.get(i)
            .and_then(|r| *r)
            .ok_or_else(|| anyhow::anyhow!(
                "Internal error: conflict {} should be resolved but wasn't",
                i
            ))?;

        let resolved_content = conflict.resolve(resolution);
        for line in resolved_content.lines() {
            result_lines.push(line.to_string());
        }

        // Skip past the conflict markers
        current_line = conflict.end_line + 1;
    }

    // Add remaining lines after the last conflict with safe indexing
    while current_line < lines.len() {
        let line = lines.get(current_line).ok_or_else(|| {
            anyhow::anyhow!(
                "Internal error: line index {} out of bounds (total lines: {})",
                current_line,
                lines.len()
            )
        })?;
        result_lines.push(line.to_string());
        current_line += 1;
    }

    // Preserve original line endings and trailing newline behavior
    let original_had_trailing_newline = content.ends_with('\n');
    let final_content = if original_had_trailing_newline {
        format!("{}\n", result_lines.join("\n"))
    } else {
        result_lines.join("\n")
    };

    // Atomic write using temp file + rename
    let parent_dir = conflicted_file.path
        .parent()
        .unwrap_or_else(|| std::path::Path::new("."));

    let temp_path = parent_dir.join(format!(
        ".{}.enkai.tmp",
        conflicted_file.path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("file")
    ));

    // Write to temp file first
    fs::write(&temp_path, &final_content)
        .with_context(|| format!("Failed to write to temporary file: {}", temp_path.display()))?;

    // Atomic rename (on Unix systems, this is guaranteed atomic)
    fs::rename(&temp_path, &conflicted_file.path)
        .with_context(|| {
            format!(
                "Failed to rename {} to {}",
                temp_path.display(),
                conflicted_file.path.display()
            )
        })?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{ConflictHunk, Resolution};
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_apply_resolutions() {
        let mut temp_file = NamedTempFile::new().unwrap();
        let content = "line 1\n<<<<<<< HEAD\ncurrent\n=======\nincoming\n>>>>>>> branch\nline 2\n";
        write!(temp_file, "{}", content).unwrap();

        let path = temp_file.path().to_path_buf();
        let hunk = ConflictHunk::new("current".to_string(), "incoming".to_string(), 1, 5);
        let mut file = ConflictedFile::new(path.clone(), vec![hunk], content.to_string());

        file.set_resolution(0, Resolution::Current);

        let result = apply_resolutions(&file);
        assert!(result.is_ok());

        let new_content = fs::read_to_string(&path).unwrap();
        assert!(new_content.contains("current"));
        assert!(!new_content.contains("incoming"));
        assert!(!new_content.contains("<<<<<<<"));
    }
}
