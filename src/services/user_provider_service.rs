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

/// Import a provider configuration file from an arbitrary source path.
///
/// Reads the file, parses and validates its contents, then merges the new
/// providers with any existing user-supplied providers and writes the result
/// to the standard user provider file location.
///
/// Returns the number of providers in the imported file.
pub fn import_provider_file(source_path: &std::path::Path) -> Result<usize, UserProviderFileError> {
    use crate::core::user_provider_file::{merge_user_providers, parse_and_validate_provider_file};

    // Read and validate the source file.
    let content = std::fs::read_to_string(source_path)?;
    let new_providers = parse_and_validate_provider_file(&content)?;
    let import_count = new_providers.len();

    // Load existing user providers (if any) and merge.
    let existing_content = load_user_provider_file()?;
    let existing_providers = match existing_content {
        Some(ref c) => {
            crate::core::user_provider_file::parse_user_provider_file(c).unwrap_or_default()
        }
        None => vec![],
    };

    let merged = merge_user_providers(existing_providers, new_providers);

    // Write the merged result to the user provider file.
    let dest = user_provider_file_path().ok_or_else(|| {
        UserProviderFileError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "could not determine config directory",
        ))
    })?;

    // Ensure parent directory exists.
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let json = serde_json::to_string_pretty(&merged).map_err(UserProviderFileError::Parse)?;
    std::fs::write(&dest, json)?;

    Ok(import_count)
}

/// Load the provider database merged with any user-supplied custom providers.
///
/// Falls back to the bundled-only database if the user provider file is
/// absent or cannot be read/parsed.
pub fn load_merged_provider_database() -> crate::core::provider::ProviderDatabase {
    let user_content = load_user_provider_file().ok().flatten();
    crate::core::user_provider_file::build_merged_database(user_content.as_deref())
        .unwrap_or_else(|_| crate::core::provider::ProviderDatabase::bundled())
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
