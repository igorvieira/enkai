/// Represents the resolution strategy for a conflict
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Resolution {
    /// Accept current (HEAD) changes
    Current,
    /// Accept incoming (branch being merged/rebased) changes
    Incoming,
    /// Accept both changes
    Both,
}

impl Resolution {
    /// Get a display string for the resolution
    pub fn as_str(&self) -> &'static str {
        match self {
            Resolution::Current => "Current (HEAD)",
            Resolution::Incoming => "Incoming",
            Resolution::Both => "Both",
        }
    }
}
