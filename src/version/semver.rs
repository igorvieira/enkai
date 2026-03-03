/// Simple semver version representation
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Version {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

impl Version {
    /// Parse a version string like "0.1.0" or "v0.1.0"
    pub fn parse(s: &str) -> Option<Self> {
        let s = s.trim().strip_prefix('v').unwrap_or(s);
        let parts: Vec<&str> = s.split('.').collect();

        if parts.len() != 3 {
            return None;
        }

        let major = parts[0].parse().ok()?;
        let minor = parts[1].parse().ok()?;
        // Handle versions like "0.1.0-beta" by taking only the numeric part
        let patch_str = parts[2].split('-').next()?;
        let patch = patch_str.parse().ok()?;

        Some(Version {
            major,
            minor,
            patch,
        })
    }

    /// Check if this version is newer than another
    pub fn is_newer_than(&self, other: &Version) -> bool {
        if self.major != other.major {
            return self.major > other.major;
        }
        if self.minor != other.minor {
            return self.minor > other.minor;
        }
        self.patch > other.patch
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_version() {
        assert_eq!(
            Version::parse("0.1.0"),
            Some(Version {
                major: 0,
                minor: 1,
                patch: 0
            })
        );
        assert_eq!(
            Version::parse("v1.2.3"),
            Some(Version {
                major: 1,
                minor: 2,
                patch: 3
            })
        );
        assert_eq!(
            Version::parse("0.1.0-beta"),
            Some(Version {
                major: 0,
                minor: 1,
                patch: 0
            })
        );
        assert_eq!(Version::parse("invalid"), None);
        assert_eq!(Version::parse("1.2"), None);
    }

    #[test]
    fn test_is_newer_than() {
        let v010 = Version::parse("0.1.0").unwrap();
        let v020 = Version::parse("0.2.0").unwrap();
        let v100 = Version::parse("1.0.0").unwrap();
        let v011 = Version::parse("0.1.1").unwrap();

        assert!(v020.is_newer_than(&v010));
        assert!(v100.is_newer_than(&v020));
        assert!(v011.is_newer_than(&v010));
        assert!(!v010.is_newer_than(&v020));
        assert!(!v010.is_newer_than(&v010));
    }
}
