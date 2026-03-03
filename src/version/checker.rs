use super::semver::Version;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

const GITHUB_API_URL: &str = "https://api.github.com/repos/igorvieira/murasaki_rs/releases/latest";
const CHECK_TIMEOUT: Duration = Duration::from_secs(3);

/// Information about an available update
#[derive(Debug, Clone)]
pub struct UpdateInfo {
    pub current_version: String,
    pub latest_version: String,
}

/// Check for updates in a non-blocking way
/// Returns Some(UpdateInfo) if a newer version is available, None otherwise
pub fn check_for_updates() -> Option<UpdateInfo> {
    // Check if update check is disabled
    if std::env::var("SAKI_NO_UPDATE_CHECK").is_ok() {
        return None;
    }

    let current_version = env!("CARGO_PKG_VERSION").to_string();

    // Spawn a thread to check for updates with a timeout
    let (tx, rx) = mpsc::channel();
    let current_for_thread = current_version.clone();

    thread::spawn(move || {
        let result = fetch_and_compare_version(&current_for_thread);
        let _ = tx.send(result);
    });

    // Wait for the result with timeout
    match rx.recv_timeout(CHECK_TIMEOUT) {
        Ok(Some(latest_version)) => Some(UpdateInfo {
            current_version,
            latest_version,
        }),
        _ => None,
    }
}

/// Fetch the latest version from GitHub and compare with current
fn fetch_and_compare_version(current_version: &str) -> Option<String> {
    let response = ureq::get(GITHUB_API_URL)
        .set("User-Agent", "murasaki_rs")
        .set("Accept", "application/vnd.github.v3+json")
        .call()
        .ok()?;

    let body = response.into_string().ok()?;

    // Simple JSON parsing for tag_name field
    let latest_version = extract_tag_name(&body)?;

    // Compare versions
    let current = Version::parse(current_version)?;
    let latest = Version::parse(&latest_version)?;

    if latest.is_newer_than(&current) {
        Some(latest_version)
    } else {
        None
    }
}

/// Extract the tag_name from GitHub API JSON response
/// This is a simple extraction without a full JSON parser
fn extract_tag_name(json: &str) -> Option<String> {
    // Look for "tag_name": "vX.Y.Z" pattern
    let tag_key = "\"tag_name\"";
    let start = json.find(tag_key)?;
    let rest = &json[start + tag_key.len()..];

    // Skip whitespace and colon
    let rest = rest.trim_start();
    let rest = rest.strip_prefix(':')?;
    let rest = rest.trim_start();

    // Find the quoted value
    let rest = rest.strip_prefix('"')?;
    let end = rest.find('"')?;

    Some(rest[..end].to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_tag_name() {
        let json = r#"{"tag_name": "v0.2.0", "name": "Release 0.2.0"}"#;
        assert_eq!(extract_tag_name(json), Some("v0.2.0".to_string()));

        let json = r#"{"tag_name":"0.1.0"}"#;
        assert_eq!(extract_tag_name(json), Some("0.1.0".to_string()));

        let json = r#"{"name": "test"}"#;
        assert_eq!(extract_tag_name(json), None);
    }

    #[test]
    fn test_check_disabled_by_env() {
        std::env::set_var("SAKI_NO_UPDATE_CHECK", "1");
        assert!(check_for_updates().is_none());
        std::env::remove_var("SAKI_NO_UPDATE_CHECK");
    }
}
