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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolution_display_strings() {
        assert_eq!(Resolution::Current.as_str(), "Current (HEAD)");
        assert_eq!(Resolution::Incoming.as_str(), "Incoming");
        assert_eq!(Resolution::Both.as_str(), "Both");
    }

    #[test]
    fn test_resolution_equality() {
        assert_eq!(Resolution::Current, Resolution::Current);
        assert_eq!(Resolution::Incoming, Resolution::Incoming);
        assert_eq!(Resolution::Both, Resolution::Both);
        assert_ne!(Resolution::Current, Resolution::Incoming);
        assert_ne!(Resolution::Current, Resolution::Both);
        assert_ne!(Resolution::Incoming, Resolution::Both);
    }

    #[test]
    fn test_resolution_copy_clone() {
        let r1 = Resolution::Current;
        let r2 = r1; // Copy trait
        assert_eq!(r1, r2);

        let r3 = r1; // Copy trait (clone not needed)
        assert_eq!(r1, r3);
    }
}
