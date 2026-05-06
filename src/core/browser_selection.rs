use std::path::PathBuf;

/// Information about an installed browser.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BrowserInfo {
    /// Human-readable name (e.g. "Firefox", "Chromium").
    pub name: String,
    /// Absolute path to the executable.
    pub executable: PathBuf,
    /// Whether this browser is considered privacy-focused.
    pub privacy_focused: bool,
    /// Known compatibility issues with OAuth redirect handling (empty = none).
    pub known_issues: Vec<String>,
}

/// The result of selecting a browser for the OAuth flow.
#[derive(Debug, Clone)]
pub struct BrowserSelectionResult {
    /// The command to run to open a URL. Either a specific executable path or "xdg-open".
    pub command: String,
    /// Human-readable name of the selected browser.
    pub browser_name: String,
    /// Optional warning to display to the user before proceeding.
    pub warning: Option<String>,
}

/// Well-known browsers in preference order, with privacy metadata.
///
/// Privacy-focused browsers are listed first so that `select_browser` can
/// prefer them when the user has not overridden the choice (FR-31).
const KNOWN_BROWSERS: &[(&str, &str, bool)] = &[
    // Privacy-focused browsers
    ("Tor Browser", "torbrowser", true),
    ("LibreWolf", "librewolf", true),
    ("Mullvad Browser", "mullvad-browser", true),
    ("Firefox", "firefox", true),
    ("GNOME Web", "epiphany", true),
    // Standard browsers
    ("Chromium", "chromium", false),
    ("Chromium", "chromium-browser", false),
    ("Google Chrome", "google-chrome-stable", false),
    ("Google Chrome", "google-chrome", false),
    ("Brave", "brave-browser", false),
    ("Vivaldi", "vivaldi", false),
    ("Microsoft Edge", "microsoft-edge", false),
    ("Opera", "opera", false),
];

/// Browsers with known OAuth redirect compatibility issues (FR-32).
///
/// Each entry is (executable-name, list of issue descriptions).
const COMPATIBILITY_ISSUES: &[(&str, &[&str])] = &[
    (
        "torbrowser",
        &[
            "Tor Browser may block localhost redirects required for OAuth sign-in. \
           Consider using a different browser for this flow.",
        ],
    ),
    (
        "mullvad-browser",
        &["Mullvad Browser may block localhost redirects required for OAuth sign-in."],
    ),
];

/// Detect browsers installed on the system by checking `$PATH`.
pub fn detect_installed_browsers() -> Vec<BrowserInfo> {
    detect_installed_browsers_with(resolve_from_path)
}

/// Testable version: accepts a resolver that maps executable name → optional path.
fn detect_installed_browsers_with(resolve: impl Fn(&str) -> Option<PathBuf>) -> Vec<BrowserInfo> {
    let mut browsers = Vec::new();
    // Track which display-names we've already added to avoid duplicates
    // (e.g. chromium vs chromium-browser both resolve to Chromium).
    let mut seen_names = std::collections::HashSet::new();

    for &(name, exec, privacy) in KNOWN_BROWSERS {
        if let Some(path) = resolve(exec) {
            if seen_names.insert(name.to_string()) {
                let issues = compatibility_issues_for(exec);
                browsers.push(BrowserInfo {
                    name: name.to_string(),
                    executable: path,
                    privacy_focused: privacy,
                    known_issues: issues,
                });
            }
        }
    }

    browsers
}

/// Look up compatibility issues for a given executable name.
fn compatibility_issues_for(exec: &str) -> Vec<String> {
    for &(name, issues) in COMPATIBILITY_ISSUES {
        if name == exec {
            return issues.iter().map(|s| (*s).to_string()).collect();
        }
    }
    Vec::new()
}

/// Resolve an executable name to an absolute path via `$PATH`.
fn resolve_from_path(name: &str) -> Option<PathBuf> {
    let path_var = std::env::var("PATH").ok()?;
    for dir in path_var.split(':') {
        let candidate = PathBuf::from(dir).join(name);
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    None
}

/// Select the best browser for the OAuth flow (FR-31, FR-32, FR-33).
///
/// Selection priority:
/// 1. User-configured browser (if set and found on system)
/// 2. Privacy-focused browser (first available from the preference list)
/// 3. Any available browser
/// 4. Fall back to `xdg-open` (system default)
///
/// If no browser can be found at all (xdg-open is missing too), returns an error
/// rather than falling back to an embedded surface, because Linux cannot reliably
/// embed a browser engine without bundling one (see story notes on FR-33).
pub fn select_browser(
    user_preference: Option<&str>,
    installed: &[BrowserInfo],
) -> BrowserSelectionResult {
    // 1. User override
    if let Some(pref) = user_preference {
        if !pref.is_empty() {
            if let Some(browser) = installed.iter().find(|b| {
                b.executable.to_string_lossy() == pref || b.name.eq_ignore_ascii_case(pref)
            }) {
                let warning = build_warning(browser);
                return BrowserSelectionResult {
                    command: browser.executable.to_string_lossy().to_string(),
                    browser_name: browser.name.clone(),
                    warning,
                };
            }
            // User specified a browser that isn't in our known list — try it directly
            // (it may be a custom executable path).
            return BrowserSelectionResult {
                command: pref.to_string(),
                browser_name: pref.to_string(),
                warning: None,
            };
        }
    }

    // 2. Privacy-focused browser
    if let Some(browser) = installed
        .iter()
        .find(|b| b.privacy_focused && b.known_issues.is_empty())
    {
        return BrowserSelectionResult {
            command: browser.executable.to_string_lossy().to_string(),
            browser_name: browser.name.clone(),
            warning: None,
        };
    }

    // 3. Privacy-focused with issues (still preferred, but warn)
    if let Some(browser) = installed.iter().find(|b| b.privacy_focused) {
        let warning = build_warning(browser);
        return BrowserSelectionResult {
            command: browser.executable.to_string_lossy().to_string(),
            browser_name: browser.name.clone(),
            warning,
        };
    }

    // 4. Any available browser
    if let Some(browser) = installed.first() {
        let warning = build_warning(browser);
        return BrowserSelectionResult {
            command: browser.executable.to_string_lossy().to_string(),
            browser_name: browser.name.clone(),
            warning,
        };
    }

    // 5. Fall back to xdg-open (FR-33: hard error is only if this also fails at launch time)
    BrowserSelectionResult {
        command: "xdg-open".to_string(),
        browser_name: "System default browser".to_string(),
        warning: None,
    }
}

/// Build a user-facing warning string if the browser has known issues.
fn build_warning(browser: &BrowserInfo) -> Option<String> {
    if browser.known_issues.is_empty() {
        return None;
    }
    Some(format!(
        "Warning for {}: {}",
        browser.name,
        browser.known_issues.join(" ")
    ))
}

/// Open a URL using the given browser command.
///
/// Returns an error string if the process fails to spawn.
pub fn launch_browser(command: &str, url: &str) -> Result<(), String> {
    std::process::Command::new(command)
        .arg(url)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .map_err(|e| format!("Failed to open browser '{}': {}", command, e))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_browser(name: &str, exec: &str, privacy: bool, issues: Vec<String>) -> BrowserInfo {
        BrowserInfo {
            name: name.to_string(),
            executable: PathBuf::from(format!("/usr/bin/{exec}")),
            privacy_focused: privacy,
            known_issues: issues,
        }
    }

    // --- detect_installed_browsers_with ---

    #[test]
    fn detect_finds_installed_browsers() {
        let browsers = detect_installed_browsers_with(|exec| match exec {
            "firefox" => Some(PathBuf::from("/usr/bin/firefox")),
            "chromium" => Some(PathBuf::from("/usr/bin/chromium")),
            _ => None,
        });
        assert_eq!(browsers.len(), 2);
        assert_eq!(browsers[0].name, "Firefox");
        assert!(browsers[0].privacy_focused);
        assert_eq!(browsers[1].name, "Chromium");
        assert!(!browsers[1].privacy_focused);
    }

    #[test]
    fn detect_deduplicates_by_name() {
        // Both "chromium" and "chromium-browser" resolve, but only one Chromium entry
        let browsers = detect_installed_browsers_with(|exec| match exec {
            "chromium" => Some(PathBuf::from("/usr/bin/chromium")),
            "chromium-browser" => Some(PathBuf::from("/usr/bin/chromium-browser")),
            _ => None,
        });
        assert_eq!(browsers.len(), 1);
        assert_eq!(browsers[0].name, "Chromium");
    }

    #[test]
    fn detect_returns_empty_when_nothing_installed() {
        let browsers = detect_installed_browsers_with(|_| None);
        assert!(browsers.is_empty());
    }

    #[test]
    fn detect_attaches_compatibility_issues() {
        let browsers = detect_installed_browsers_with(|exec| {
            if exec == "torbrowser" {
                Some(PathBuf::from("/usr/bin/torbrowser"))
            } else {
                None
            }
        });
        assert_eq!(browsers.len(), 1);
        assert!(!browsers[0].known_issues.is_empty());
        assert!(browsers[0].known_issues[0].contains("localhost"));
    }

    // --- select_browser ---

    #[test]
    fn select_prefers_user_override() {
        let installed = vec![
            make_browser("Firefox", "firefox", true, vec![]),
            make_browser("Chromium", "chromium", false, vec![]),
        ];
        let result = select_browser(Some("Chromium"), &installed);
        assert_eq!(result.browser_name, "Chromium");
        assert!(result.warning.is_none());
    }

    #[test]
    fn select_prefers_user_override_by_path() {
        let installed = vec![
            make_browser("Firefox", "firefox", true, vec![]),
            make_browser("Chromium", "chromium", false, vec![]),
        ];
        let result = select_browser(Some("/usr/bin/chromium"), &installed);
        assert_eq!(result.browser_name, "Chromium");
    }

    #[test]
    fn select_falls_through_to_privacy_browser() {
        let installed = vec![
            make_browser("Chromium", "chromium", false, vec![]),
            make_browser("Firefox", "firefox", true, vec![]),
        ];
        let result = select_browser(None, &installed);
        assert_eq!(result.browser_name, "Firefox");
    }

    #[test]
    fn select_warns_on_privacy_browser_with_issues() {
        let installed = vec![make_browser(
            "Tor Browser",
            "torbrowser",
            true,
            vec!["blocks redirects".into()],
        )];
        let result = select_browser(None, &installed);
        assert_eq!(result.browser_name, "Tor Browser");
        assert!(result.warning.is_some());
        assert!(result.warning.unwrap().contains("blocks redirects"));
    }

    #[test]
    fn select_prefers_issue_free_privacy_browser() {
        let installed = vec![
            make_browser(
                "Tor Browser",
                "torbrowser",
                true,
                vec!["blocks redirects".into()],
            ),
            make_browser("Firefox", "firefox", true, vec![]),
        ];
        let result = select_browser(None, &installed);
        assert_eq!(result.browser_name, "Firefox");
        assert!(result.warning.is_none());
    }

    #[test]
    fn select_falls_back_to_any_browser() {
        let installed = vec![make_browser("Chromium", "chromium", false, vec![])];
        let result = select_browser(None, &installed);
        assert_eq!(result.browser_name, "Chromium");
    }

    #[test]
    fn select_falls_back_to_xdg_open_when_no_browsers() {
        let result = select_browser(None, &[]);
        assert_eq!(result.command, "xdg-open");
        assert_eq!(result.browser_name, "System default browser");
    }

    #[test]
    fn select_user_override_unknown_browser_passes_through() {
        let result = select_browser(Some("/opt/custom-browser/run"), &[]);
        assert_eq!(result.command, "/opt/custom-browser/run");
        assert_eq!(result.browser_name, "/opt/custom-browser/run");
    }

    #[test]
    fn select_empty_user_preference_ignored() {
        let installed = vec![make_browser("Firefox", "firefox", true, vec![])];
        let result = select_browser(Some(""), &installed);
        assert_eq!(result.browser_name, "Firefox");
    }

    #[test]
    fn select_warns_for_user_override_with_issues() {
        let installed = vec![
            make_browser(
                "Tor Browser",
                "torbrowser",
                true,
                vec!["blocks redirects".into()],
            ),
            make_browser("Firefox", "firefox", true, vec![]),
        ];
        let result = select_browser(Some("Tor Browser"), &installed);
        assert_eq!(result.browser_name, "Tor Browser");
        assert!(result.warning.is_some());
    }

    // --- build_warning ---

    #[test]
    fn build_warning_returns_none_for_no_issues() {
        let browser = make_browser("Firefox", "firefox", true, vec![]);
        assert!(build_warning(&browser).is_none());
    }

    #[test]
    fn build_warning_includes_browser_name() {
        let browser = make_browser("Tor Browser", "torbrowser", true, vec!["issue1".into()]);
        let warning = build_warning(&browser).unwrap();
        assert!(warning.contains("Tor Browser"));
        assert!(warning.contains("issue1"));
    }

    // --- compatibility_issues_for ---

    #[test]
    fn compatibility_issues_for_known_browser() {
        let issues = compatibility_issues_for("torbrowser");
        assert!(!issues.is_empty());
    }

    #[test]
    fn compatibility_issues_for_unknown_browser() {
        let issues = compatibility_issues_for("firefox");
        assert!(issues.is_empty());
    }

    // --- BrowserInfo ---

    #[test]
    fn browser_info_equality() {
        let a = make_browser("Firefox", "firefox", true, vec![]);
        let b = make_browser("Firefox", "firefox", true, vec![]);
        assert_eq!(a, b);
    }

    // --- launch_browser ---

    #[test]
    fn launch_browser_fails_for_nonexistent_command() {
        let result = launch_browser("/nonexistent/browser/12345", "https://example.com");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Failed to open browser"));
    }
}
