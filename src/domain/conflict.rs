use std::path::PathBuf;

use super::Resolution;

/// Represents a single conflict hunk within a file
#[derive(Debug, Clone)]
pub struct ConflictHunk {
    /// Content from the current branch (HEAD)
    pub current: String,
    /// Content from the incoming branch
    pub incoming: String,
    /// Starting line number of the conflict in the original file
    pub start_line: usize,
    /// Ending line number of the conflict in the original file
    pub end_line: usize,
}

impl ConflictHunk {
    /// Create a new conflict hunk
    pub fn new(current: String, incoming: String, start_line: usize, end_line: usize) -> Self {
        Self {
            current,
            incoming,
            start_line,
            end_line,
        }
    }

    /// Get the resolved content based on the resolution strategy
    pub fn resolve(&self, resolution: Resolution) -> String {
        match resolution {
            Resolution::Current => self.current.clone(),
            Resolution::Incoming => self.incoming.clone(),
            Resolution::Both => {
                format!("{}\n{}", self.current.trim(), self.incoming.trim())
            }
        }
    }
}

/// Represents a file with conflicts
#[derive(Debug, Clone)]
pub struct ConflictedFile {
    /// Path to the file
    pub path: PathBuf,
    /// All conflicts found in the file
    pub conflicts: Vec<ConflictHunk>,
    /// Resolution choices for each conflict (None if not yet resolved)
    pub resolutions: Vec<Option<Resolution>>,
    /// Original file content (before parsing conflicts)
    pub original_content: String,
}

impl ConflictedFile {
    /// Create a new conflicted file
    pub fn new(path: PathBuf, conflicts: Vec<ConflictHunk>, original_content: String) -> Self {
        let resolutions = vec![None; conflicts.len()];
        Self {
            path,
            conflicts,
            resolutions,
            original_content,
        }
    }

    /// Check if all conflicts have been resolved
    pub fn is_fully_resolved(&self) -> bool {
        self.resolutions.iter().all(|r| r.is_some())
    }

    /// Get the number of resolved conflicts
    pub fn resolved_count(&self) -> usize {
        self.resolutions.iter().filter(|r| r.is_some()).count()
    }

    /// Get the total number of conflicts
    pub fn total_conflicts(&self) -> usize {
        self.conflicts.len()
    }

    /// Set resolution for a specific conflict
    pub fn set_resolution(&mut self, conflict_index: usize, resolution: Resolution) {
        if conflict_index < self.resolutions.len() {
            self.resolutions[conflict_index] = Some(resolution);
        }
    }

    /// Get the file name as a string
    pub fn file_name(&self) -> String {
        self.path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Unknown")
            .to_string()
    }

    /// Get the file path as a string
    pub fn path_string(&self) -> String {
        self.path.display().to_string()
    }
}
