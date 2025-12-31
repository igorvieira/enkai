/// Represents the type of git operation in progress
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GitOperation {
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
            GitOperation::Merge => "Merge",
            GitOperation::Rebase => "Rebase",
            GitOperation::RebaseInteractive => "Interactive Rebase",
        }
    }
}
