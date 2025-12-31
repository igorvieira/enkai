pub mod conflict;
pub mod git_operation;
pub mod resolution;

pub use conflict::{ConflictHunk, ConflictedFile};
pub use git_operation::GitOperation;
pub use resolution::Resolution;
