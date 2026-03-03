/// Represents the type of git operation in progress
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GitOperation {
    /// No git operation in progress
    None,
    /// A merge operation
    Merge,
    /// A rebase operation
    Rebase,
    /// An interactive rebase operation
    RebaseInteractive,
}

impl GitOperation {
    /// Check if this is any kind of rebase
    pub fn is_rebase(&self) -> bool {
        matches!(self, GitOperation::Rebase | GitOperation::RebaseInteractive)
    }

    /// Check if this is an interactive rebase
    pub fn is_interactive_rebase(&self) -> bool {
        matches!(self, GitOperation::RebaseInteractive)
    }

    /// Get a display string for the operation
    pub fn as_str(&self) -> &'static str {
        match self {
            GitOperation::None => "None",
            GitOperation::Merge => "Merge",
            GitOperation::Rebase => "Rebase",
            GitOperation::RebaseInteractive => "Interactive Rebase",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_rebase_for_rebase() {
        assert!(GitOperation::Rebase.is_rebase());
    }

    #[test]
    fn test_is_rebase_for_interactive_rebase() {
        assert!(GitOperation::RebaseInteractive.is_rebase());
    }

    #[test]
    fn test_is_rebase_for_merge() {
        assert!(!GitOperation::Merge.is_rebase());
    }

    #[test]
    fn test_is_rebase_for_none() {
        assert!(!GitOperation::None.is_rebase());
    }

    #[test]
    fn test_is_interactive_rebase() {
        assert!(GitOperation::RebaseInteractive.is_interactive_rebase());
        assert!(!GitOperation::Rebase.is_interactive_rebase());
        assert!(!GitOperation::Merge.is_interactive_rebase());
        assert!(!GitOperation::None.is_interactive_rebase());
    }

    #[test]
    fn test_as_str() {
        assert_eq!(GitOperation::None.as_str(), "None");
        assert_eq!(GitOperation::Merge.as_str(), "Merge");
        assert_eq!(GitOperation::Rebase.as_str(), "Rebase");
        assert_eq!(
            GitOperation::RebaseInteractive.as_str(),
            "Interactive Rebase"
        );
    }

    #[test]
    fn test_equality() {
        assert_eq!(GitOperation::None, GitOperation::None);
        assert_eq!(GitOperation::Merge, GitOperation::Merge);
        assert_ne!(GitOperation::None, GitOperation::Merge);
        assert_ne!(GitOperation::Rebase, GitOperation::RebaseInteractive);
    }

    #[test]
    fn test_clone() {
        let op = GitOperation::Merge;
        let cloned = op;
        assert_eq!(op, cloned);
    }
}
