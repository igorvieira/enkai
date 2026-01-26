use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

use crate::domain::{ConflictHunk, ConflictedFile};

const CONFLICT_START: &str = "<<<<<<<";
const CONFLICT_SEPARATOR: &str = "=======";
const CONFLICT_END: &str = ">>>>>>>";

/// Parse conflicts from a file
pub fn parse_conflicts(file_path: &Path) -> Result<ConflictedFile> {
    let content = fs::read_to_string(file_path)
        .with_context(|| format!("Failed to read file: {}", file_path.display()))?;

    let lines: Vec<&str> = content.lines().collect();
    let mut conflicts = Vec::new();
    let mut i = 0;

    while i < lines.len() {
        // Safe indexing with bounds check
        let line = match lines.get(i) {
            Some(l) => l,
            None => break, // Should never happen due to while condition, but safe
        };

        if line.starts_with(CONFLICT_START) {
            let conflict_start_line = i;

            // Find separator with safe indexing
            let mut separator_line = None;
            for j in (i + 1)..lines.len() {
                if let Some(line) = lines.get(j) {
                    if line.starts_with(CONFLICT_SEPARATOR) {
                        separator_line = Some(j);
                        break;
                    }
                }
            }

            let separator_line = match separator_line {
                Some(line) => line,
                None => {
                    anyhow::bail!(
                        "Malformed conflict in {}: missing separator at line {}",
                        file_path.display(),
                        i + 1
                    );
                }
            };

            // Find end marker with safe indexing
            let mut end_line = None;
            for j in (separator_line + 1)..lines.len() {
                if let Some(line) = lines.get(j) {
                    if line.starts_with(CONFLICT_END) {
                        end_line = Some(j);
                        break;
                    }
                }
            }

            let end_line = match end_line {
                Some(line) => line,
                None => {
                    anyhow::bail!(
                        "Malformed conflict in {}: missing end marker at line {}",
                        file_path.display(),
                        separator_line + 1
                    );
                }
            };

            // Extract current and incoming content with safe slicing
            let current_lines = lines
                .get((conflict_start_line + 1)..separator_line)
                .ok_or_else(|| {
                    anyhow::anyhow!(
                        "Invalid conflict range in {}: lines {}-{}",
                        file_path.display(),
                        conflict_start_line + 1,
                        separator_line
                    )
                })?;

            let incoming_lines = lines.get((separator_line + 1)..end_line).ok_or_else(|| {
                anyhow::anyhow!(
                    "Invalid conflict range in {}: lines {}-{}",
                    file_path.display(),
                    separator_line + 1,
                    end_line
                )
            })?;

            let current = current_lines.join("\n");
            let incoming = incoming_lines.join("\n");

            let hunk = ConflictHunk::new(current, incoming, conflict_start_line, end_line);
            conflicts.push(hunk);

            // Move past this conflict
            i = end_line + 1;
        } else {
            i += 1;
        }
    }

    if conflicts.is_empty() {
        anyhow::bail!("No conflicts found in file: {}", file_path.display());
    }

    Ok(ConflictedFile::new(
        file_path.to_path_buf(),
        conflicts,
        content,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_parse_simple_conflict() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(
            temp_file,
            "line 1\n<<<<<<< HEAD\ncurrent content\n=======\nincoming content\n>>>>>>> branch\nline 2"
        )
        .unwrap();

        let result = parse_conflicts(temp_file.path());
        assert!(result.is_ok());

        let conflicted_file = result.unwrap();
        assert_eq!(conflicted_file.conflicts.len(), 1);
        assert_eq!(conflicted_file.conflicts[0].current, "current content");
        assert_eq!(conflicted_file.conflicts[0].incoming, "incoming content");
    }

    #[test]
    fn test_parse_multiple_conflicts() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(
            temp_file,
            "<<<<<<< HEAD\nfirst current\n=======\nfirst incoming\n>>>>>>> branch\nmiddle\n<<<<<<< HEAD\nsecond current\n=======\nsecond incoming\n>>>>>>> branch"
        )
        .unwrap();

        let result = parse_conflicts(temp_file.path());
        assert!(result.is_ok());

        let conflicted_file = result.unwrap();
        assert_eq!(conflicted_file.conflicts.len(), 2);
    }

    #[test]
    fn test_parse_no_conflicts() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "just normal content\nno conflicts here").unwrap();

        let result = parse_conflicts(temp_file.path());
        assert!(result.is_err());
    }
}
