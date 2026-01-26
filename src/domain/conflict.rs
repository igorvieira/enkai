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

    /// Clear resolution for a specific conflict (undo)
    pub fn clear_resolution(&mut self, conflict_index: usize) {
        if conflict_index < self.resolutions.len() {
            self.resolutions[conflict_index] = None;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_conflict_hunk_creation() {
        let hunk = ConflictHunk::new(
            "current content".to_string(),
            "incoming content".to_string(),
            10,
            20,
        );
        assert_eq!(hunk.current, "current content");
        assert_eq!(hunk.incoming, "incoming content");
        assert_eq!(hunk.start_line, 10);
        assert_eq!(hunk.end_line, 20);
    }

    #[test]
    fn test_conflict_hunk_resolve_current() {
        let hunk = ConflictHunk::new("current".to_string(), "incoming".to_string(), 0, 5);
        assert_eq!(hunk.resolve(Resolution::Current), "current");
    }

    #[test]
    fn test_conflict_hunk_resolve_incoming() {
        let hunk = ConflictHunk::new("current".to_string(), "incoming".to_string(), 0, 5);
        assert_eq!(hunk.resolve(Resolution::Incoming), "incoming");
    }

    #[test]
    fn test_conflict_hunk_resolve_both() {
        let hunk = ConflictHunk::new("current".to_string(), "incoming".to_string(), 0, 5);
        assert_eq!(hunk.resolve(Resolution::Both), "current\nincoming");
    }

    #[test]
    fn test_conflict_hunk_resolve_both_with_whitespace() {
        let hunk = ConflictHunk::new("  current  ".to_string(), "  incoming  ".to_string(), 0, 5);
        assert_eq!(hunk.resolve(Resolution::Both), "current\nincoming");
    }

    #[test]
    fn test_conflicted_file_creation() {
        let path = PathBuf::from("test.txt");
        let conflicts = vec![
            ConflictHunk::new("a".to_string(), "b".to_string(), 0, 5),
            ConflictHunk::new("c".to_string(), "d".to_string(), 6, 10),
        ];
        let file = ConflictedFile::new(path.clone(), conflicts.clone(), "content".to_string());

        assert_eq!(file.path, path);
        assert_eq!(file.conflicts.len(), 2);
        assert_eq!(file.resolutions.len(), 2);
        assert_eq!(file.original_content, "content");
    }

    #[test]
    fn test_conflicted_file_fully_resolved() {
        let path = PathBuf::from("test.txt");
        let conflicts = vec![ConflictHunk::new("a".to_string(), "b".to_string(), 0, 5)];
        let mut file = ConflictedFile::new(path, conflicts, "content".to_string());

        assert!(!file.is_fully_resolved());
        assert_eq!(file.resolved_count(), 0);

        file.set_resolution(0, Resolution::Current);
        assert!(file.is_fully_resolved());
        assert_eq!(file.resolved_count(), 1);
    }

    #[test]
    fn test_conflicted_file_partial_resolution() {
        let path = PathBuf::from("test.txt");
        let conflicts = vec![
            ConflictHunk::new("a".to_string(), "b".to_string(), 0, 5),
            ConflictHunk::new("c".to_string(), "d".to_string(), 6, 10),
            ConflictHunk::new("e".to_string(), "f".to_string(), 11, 15),
        ];
        let mut file = ConflictedFile::new(path, conflicts, "content".to_string());

        file.set_resolution(0, Resolution::Current);
        file.set_resolution(2, Resolution::Incoming);

        assert!(!file.is_fully_resolved());
        assert_eq!(file.resolved_count(), 2);
        assert_eq!(file.total_conflicts(), 3);
    }

    #[test]
    fn test_conflicted_file_clear_resolution() {
        let path = PathBuf::from("test.txt");
        let conflicts = vec![ConflictHunk::new("a".to_string(), "b".to_string(), 0, 5)];
        let mut file = ConflictedFile::new(path, conflicts, "content".to_string());

        file.set_resolution(0, Resolution::Current);
        assert!(file.is_fully_resolved());

        file.clear_resolution(0);
        assert!(!file.is_fully_resolved());
        assert_eq!(file.resolved_count(), 0);
    }

    #[test]
    fn test_conflicted_file_set_resolution_out_of_bounds() {
        let path = PathBuf::from("test.txt");
        let conflicts = vec![ConflictHunk::new("a".to_string(), "b".to_string(), 0, 5)];
        let mut file = ConflictedFile::new(path, conflicts, "content".to_string());

        // Should not panic
        file.set_resolution(10, Resolution::Current);
        assert_eq!(file.resolved_count(), 0);
    }

    #[test]
    fn test_conflicted_file_clear_resolution_out_of_bounds() {
        let path = PathBuf::from("test.txt");
        let conflicts = vec![ConflictHunk::new("a".to_string(), "b".to_string(), 0, 5)];
        let mut file = ConflictedFile::new(path, conflicts, "content".to_string());

        // Should not panic
        file.clear_resolution(10);
    }

    #[test]
    fn test_conflicted_file_name() {
        let path = PathBuf::from("/path/to/test.txt");
        let conflicts = vec![];
        let file = ConflictedFile::new(path, conflicts, "".to_string());

        assert_eq!(file.file_name(), "test.txt");
    }

    #[test]
    fn test_conflicted_file_path_string() {
        let path = PathBuf::from("test.txt");
        let conflicts = vec![];
        let file = ConflictedFile::new(path, conflicts, "".to_string());

        assert!(file.path_string().contains("test.txt"));
    }
}
