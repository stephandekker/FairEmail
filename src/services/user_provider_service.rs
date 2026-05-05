//! Service for loading user-supplied provider files from the filesystem (FR-16).

use crate::core::user_provider_file::{
    UserProviderFileError, APP_CONFIG_DIR, USER_PROVIDER_FILENAME,
};
use std::path::PathBuf;

/// Resolve the path to the user-supplied provider file.
///
/// Uses `$XDG_CONFIG_HOME/fairmail/providers.json` if set,
/// otherwise `~/.config/fairmail/providers.json`.
pub fn user_provider_file_path() -> Option<PathBuf> {
    let config_dir = std::env::var("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            let home = std::env::var("HOME").unwrap_or_else(|_| String::from("~"));
            PathBuf::from(home).join(".config")
        });

    Some(config_dir.join(APP_CONFIG_DIR).join(USER_PROVIDER_FILENAME))
}

/// Load user-supplied provider file content, if it exists.
///
/// Returns `Ok(None)` if the file does not exist (normal default behavior).
/// Returns `Ok(Some(content))` if the file exists and was read.
/// Returns `Err` on I/O errors other than not-found.
pub fn load_user_provider_file() -> Result<Option<String>, UserProviderFileError> {
    let path = match user_provider_file_path() {
        Some(p) => p,
        None => return Ok(None),
    };

    match std::fs::read_to_string(&path) {
        Ok(content) => Ok(Some(content)),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(UserProviderFileError::Io(e)),
    }
}

/// Load user-supplied provider file from a specific path.
///
/// Returns `Ok(None)` if the file does not exist.
pub fn load_user_provider_file_from(
    path: &std::path::Path,
) -> Result<Option<String>, UserProviderFileError> {
    match std::fs::read_to_string(path) {
        Ok(content) => Ok(Some(content)),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(UserProviderFileError::Io(e)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_load_nonexistent_file_returns_none() {
        let path = PathBuf::from("/tmp/nonexistent-fairmail-test-providers.json");
        let result = load_user_provider_file_from(&path).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_load_existing_file_returns_content() {
        let mut tmp = NamedTempFile::new().unwrap();
        writeln!(tmp, "[]").unwrap();

        let result = load_user_provider_file_from(tmp.path()).unwrap();
        assert!(result.is_some());
    }

    #[test]
    fn test_user_provider_file_path_uses_xdg() {
        std::env::set_var("XDG_CONFIG_HOME", "/tmp/test-xdg");
        let path = user_provider_file_path().unwrap();
        assert_eq!(path, PathBuf::from("/tmp/test-xdg/fairmail/providers.json"));
        std::env::remove_var("XDG_CONFIG_HOME");
    }
}
