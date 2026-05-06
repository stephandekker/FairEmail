use std::io::{Read, Write};
use std::net::TcpListener;
use uuid::Uuid;

use crate::core::credential_store::{CredentialRole, CredentialStore, SecretValue};
use crate::core::oauth_flow::{
    compute_expiry_epoch, parse_callback_query, parse_token_response_json, CallbackParams,
    OAuthFlowError, TokenResponse, ValidatedTokenResponse,
};

const CALLBACK_SUCCESS_HTML: &str = "<html><body>\
    <h2>Authorization complete</h2>\
    <p>You can close this window and return to FairEmail.</p>\
    </body></html>";

const CALLBACK_ERROR_HTML: &str = "<html><body>\
    <h2>Authorization failed</h2>\
    <p>Please return to FairEmail and try again.</p>\
    </body></html>";

/// Bind a local TCP listener on 127.0.0.1 with an OS-assigned port
/// for receiving the OAuth redirect callback.
pub fn bind_redirect_listener() -> Result<(TcpListener, u16), OAuthFlowError> {
    let listener = TcpListener::bind("127.0.0.1:0")
        .map_err(|e| OAuthFlowError::ServerBindFailed(e.to_string()))?;
    let port = listener
        .local_addr()
        .map_err(|e| OAuthFlowError::ServerBindFailed(e.to_string()))?
        .port();
    Ok((listener, port))
}

/// Wait for an OAuth redirect callback on the listener.
///
/// Blocks the calling thread until a connection arrives. The session timeout
/// is enforced by the caller via `OAuthSession::validate_state`.
pub fn wait_for_callback(listener: TcpListener) -> Result<CallbackParams, OAuthFlowError> {
    let (mut stream, _) = listener
        .accept()
        .map_err(|e| OAuthFlowError::CallbackError(format!("Accept failed: {e}")))?;

    let mut buf = [0u8; 4096];
    let n = stream
        .read(&mut buf)
        .map_err(|e| OAuthFlowError::CallbackError(format!("Read failed: {e}")))?;

    let request = String::from_utf8_lossy(&buf[..n]);
    let first_line = request
        .lines()
        .next()
        .ok_or_else(|| OAuthFlowError::CallbackError("Empty request".to_string()))?;

    let path = first_line
        .split_whitespace()
        .nth(1)
        .ok_or_else(|| OAuthFlowError::CallbackError("Malformed request".to_string()))?;

    let query = path
        .split('?')
        .nth(1)
        .ok_or_else(|| OAuthFlowError::CallbackError("No query string".to_string()))?;

    let result = parse_callback_query(query);

    let http_response = match &result {
        Ok(_) => format!(
            "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nConnection: close\r\n\r\n{CALLBACK_SUCCESS_HTML}"
        ),
        Err(_) => format!(
            "HTTP/1.1 400 Bad Request\r\nContent-Type: text/html; charset=utf-8\r\nConnection: close\r\n\r\n{CALLBACK_ERROR_HTML}"
        ),
    };
    let _ = stream.write_all(http_response.as_bytes());

    result
}

/// Load the user's configured OAuth browser preference from the settings database.
///
/// Returns `None` if no preference is set, or if the database cannot be read
/// (e.g. first launch before the DB is created).
pub fn load_browser_preference() -> Option<String> {
    let db_path = if let Ok(custom) = std::env::var("FAIRMAIL_DATA_DIR") {
        std::path::PathBuf::from(custom).join("fairmail.db")
    } else {
        glib::user_data_dir().join("fairmail").join("fairmail.db")
    };
    let store = crate::services::sqlite_settings_store::SqliteSettingsStore::new(db_path).ok()?;
    store.load().ok()?.oauth_browser
}

/// Result of selecting and opening a browser for OAuth.
pub struct BrowserOpenResult {
    /// Human-readable name of the browser that was selected.
    pub browser_name: String,
    /// Optional warning about browser compatibility issues (FR-32).
    pub warning: Option<String>,
}

/// Open the given URL in the best available browser for OAuth (FR-5, FR-31, FR-32).
///
/// Uses the browser selection logic to prefer privacy-focused browsers and
/// respect user configuration. Returns information about the selected browser
/// so the caller can display warnings if needed.
pub fn open_browser_with_selection(
    url: &str,
    user_preference: Option<&str>,
) -> Result<BrowserOpenResult, OAuthFlowError> {
    let installed = crate::core::browser_selection::detect_installed_browsers();
    let selection = crate::core::browser_selection::select_browser(user_preference, &installed);

    crate::core::browser_selection::launch_browser(&selection.command, url)
        .map_err(OAuthFlowError::BrowserOpenFailed)?;

    Ok(BrowserOpenResult {
        browser_name: selection.browser_name,
        warning: selection.warning,
    })
}

/// Open the given URL in the user's system browser (FR-5).
///
/// Legacy entry point that delegates to `open_browser_with_selection` without
/// a user preference. Callers that need browser warnings or user-configured
/// browser support should use `open_browser_with_selection` directly.
pub fn open_browser(url: &str) -> Result<(), OAuthFlowError> {
    open_browser_with_selection(url, None)?;
    Ok(())
}

/// Exchange an authorization code for tokens via HTTPS POST to the token endpoint (FR-8).
pub fn exchange_code_for_tokens(
    params: crate::core::oauth_flow::TokenExchangeParams,
) -> Result<TokenResponse, OAuthFlowError> {
    let stripped = params.token_url.strip_prefix("https://").ok_or_else(|| {
        OAuthFlowError::TokenExchangeFailed("Token URL must use HTTPS".to_string())
    })?;

    let (host_and_port, path) = match stripped.find('/') {
        Some(i) => (&stripped[..i], &stripped[i..]),
        None => (stripped, "/"),
    };

    let (host, port) = match host_and_port.rsplit_once(':') {
        Some((h, p)) => (h, p.parse::<u16>().unwrap_or(443)),
        None => (host_and_port, 443u16),
    };

    let body = params.form_body();

    let tcp = std::net::TcpStream::connect((host, port))
        .map_err(|e| OAuthFlowError::TokenExchangeFailed(format!("Connect failed: {e}")))?;

    tcp.set_read_timeout(Some(std::time::Duration::from_secs(30)))
        .ok();

    let connector = native_tls::TlsConnector::new()
        .map_err(|e| OAuthFlowError::TokenExchangeFailed(format!("TLS init failed: {e}")))?;

    let mut tls = connector
        .connect(host, tcp)
        .map_err(|e| OAuthFlowError::TokenExchangeFailed(format!("TLS handshake failed: {e}")))?;

    let request = format!(
        "POST {path} HTTP/1.1\r\n\
         Host: {host_and_port}\r\n\
         Content-Type: application/x-www-form-urlencoded\r\n\
         Content-Length: {}\r\n\
         Accept: application/json\r\n\
         Connection: close\r\n\
         \r\n\
         {body}",
        body.len(),
    );

    tls.write_all(request.as_bytes())
        .map_err(|e| OAuthFlowError::TokenExchangeFailed(format!("Write failed: {e}")))?;

    let mut response = String::new();
    tls.read_to_string(&mut response)
        .map_err(|e| OAuthFlowError::TokenExchangeFailed(format!("Read failed: {e}")))?;

    let (headers, resp_body) = response
        .split_once("\r\n\r\n")
        .ok_or_else(|| OAuthFlowError::TokenExchangeFailed("Invalid HTTP response".to_string()))?;

    let status_line = headers.lines().next().unwrap_or("");
    let status_code = status_line.split_whitespace().nth(1).unwrap_or("0");
    if !status_code.starts_with('2') {
        return Err(OAuthFlowError::TokenExchangeFailed(format!(
            "HTTP {status_code}: {resp_body}"
        )));
    }

    parse_token_response_json(resp_body)
}

/// Store validated OAuth tokens in the credential store (FR-12, FR-13, FR-14).
///
/// Stores the access token (as IMAP credential), refresh token, and expiry
/// timestamp independently per account. Tokens are wrapped in `SecretValue`
/// and never displayed in plain text (NFR-3).
pub fn store_oauth_tokens(
    credential_store: &dyn CredentialStore,
    account_id: Uuid,
    validated: &ValidatedTokenResponse,
) -> Result<(), OAuthFlowError> {
    // Store access token as the IMAP credential
    credential_store
        .write(
            account_id,
            CredentialRole::ImapPassword,
            &SecretValue::new(validated.access_token.clone()),
        )
        .map_err(|e| OAuthFlowError::CredentialStoreError(e.to_string()))?;

    // Store refresh token
    credential_store
        .write(
            account_id,
            CredentialRole::OAuthRefreshToken,
            &SecretValue::new(validated.refresh_token.clone()),
        )
        .map_err(|e| OAuthFlowError::CredentialStoreError(e.to_string()))?;

    // Store expiry timestamp if available
    if let Some(expiry_epoch) = compute_expiry_epoch(validated.expires_in) {
        credential_store
            .write(
                account_id,
                CredentialRole::OAuthTokenExpiry,
                &SecretValue::new(expiry_epoch.to_string()),
            )
            .map_err(|e| OAuthFlowError::CredentialStoreError(e.to_string()))?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::oauth_flow::ValidatedTokenResponse;
    use crate::services::memory_credential_store::MemoryCredentialStore;

    #[test]
    fn bind_redirect_listener_gets_valid_port() {
        let (listener, port) = bind_redirect_listener().unwrap();
        assert!(port > 0);
        // Verify the listener is actually bound
        let addr = listener.local_addr().unwrap();
        assert_eq!(addr.ip(), std::net::Ipv4Addr::LOCALHOST);
        assert_eq!(addr.port(), port);
    }

    #[test]
    fn bind_redirect_listener_different_ports() {
        let (_l1, p1) = bind_redirect_listener().unwrap();
        let (_l2, p2) = bind_redirect_listener().unwrap();
        assert_ne!(p1, p2);
    }

    #[test]
    fn store_oauth_tokens_writes_all_credentials() {
        let store = MemoryCredentialStore::new();
        let account_id = Uuid::new_v4();
        let validated = ValidatedTokenResponse {
            access_token: "access-token-123".to_string(),
            refresh_token: "refresh-token-456".to_string(),
            expires_in: Some(3600),
        };

        store_oauth_tokens(&store, account_id, &validated).unwrap();

        // Access token stored as IMAP credential
        let access = store
            .read(account_id, CredentialRole::ImapPassword)
            .unwrap();
        assert_eq!(access.expose(), "access-token-123");

        // Refresh token stored
        let refresh = store
            .read(account_id, CredentialRole::OAuthRefreshToken)
            .unwrap();
        assert_eq!(refresh.expose(), "refresh-token-456");

        // Expiry stored as epoch timestamp
        let expiry = store
            .read(account_id, CredentialRole::OAuthTokenExpiry)
            .unwrap();
        let expiry_val: u64 = expiry.expose().parse().unwrap();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        assert!(expiry_val >= now + 3599);
        assert!(expiry_val <= now + 3601);
    }

    #[test]
    fn store_oauth_tokens_skips_expiry_when_none() {
        let store = MemoryCredentialStore::new();
        let account_id = Uuid::new_v4();
        let validated = ValidatedTokenResponse {
            access_token: "access".to_string(),
            refresh_token: "refresh".to_string(),
            expires_in: None,
        };

        store_oauth_tokens(&store, account_id, &validated).unwrap();

        assert!(store.read(account_id, CredentialRole::ImapPassword).is_ok());
        assert!(store
            .read(account_id, CredentialRole::OAuthRefreshToken)
            .is_ok());
        assert!(store
            .read(account_id, CredentialRole::OAuthTokenExpiry)
            .is_err());
    }

    #[test]
    fn store_tokens_per_account_independence() {
        let store = MemoryCredentialStore::new();
        let id1 = Uuid::new_v4();
        let id2 = Uuid::new_v4();

        let tokens1 = ValidatedTokenResponse {
            access_token: "access-1".to_string(),
            refresh_token: "refresh-1".to_string(),
            expires_in: Some(3600),
        };
        let tokens2 = ValidatedTokenResponse {
            access_token: "access-2".to_string(),
            refresh_token: "refresh-2".to_string(),
            expires_in: Some(7200),
        };

        store_oauth_tokens(&store, id1, &tokens1).unwrap();
        store_oauth_tokens(&store, id2, &tokens2).unwrap();

        // Each account has its own tokens
        assert_eq!(
            store
                .read(id1, CredentialRole::ImapPassword)
                .unwrap()
                .expose(),
            "access-1"
        );
        assert_eq!(
            store
                .read(id2, CredentialRole::ImapPassword)
                .unwrap()
                .expose(),
            "access-2"
        );
        assert_eq!(
            store
                .read(id1, CredentialRole::OAuthRefreshToken)
                .unwrap()
                .expose(),
            "refresh-1"
        );
        assert_eq!(
            store
                .read(id2, CredentialRole::OAuthRefreshToken)
                .unwrap()
                .expose(),
            "refresh-2"
        );
    }

    #[test]
    fn store_tokens_overwrites_existing() {
        let store = MemoryCredentialStore::new();
        let account_id = Uuid::new_v4();

        let first = ValidatedTokenResponse {
            access_token: "old-access".to_string(),
            refresh_token: "old-refresh".to_string(),
            expires_in: Some(3600),
        };
        store_oauth_tokens(&store, account_id, &first).unwrap();

        let second = ValidatedTokenResponse {
            access_token: "new-access".to_string(),
            refresh_token: "new-refresh".to_string(),
            expires_in: Some(7200),
        };
        store_oauth_tokens(&store, account_id, &second).unwrap();

        assert_eq!(
            store
                .read(account_id, CredentialRole::ImapPassword)
                .unwrap()
                .expose(),
            "new-access"
        );
        assert_eq!(
            store
                .read(account_id, CredentialRole::OAuthRefreshToken)
                .unwrap()
                .expose(),
            "new-refresh"
        );
    }

    #[test]
    fn tokens_never_leak_via_debug() {
        let validated = ValidatedTokenResponse {
            access_token: "super-secret-access".to_string(),
            refresh_token: "super-secret-refresh".to_string(),
            expires_in: Some(3600),
        };

        let store = MemoryCredentialStore::new();
        let account_id = Uuid::new_v4();
        store_oauth_tokens(&store, account_id, &validated).unwrap();

        // SecretValue redacts in Debug
        let secret = store
            .read(account_id, CredentialRole::ImapPassword)
            .unwrap();
        let debug = format!("{:?}", secret);
        assert!(!debug.contains("super-secret-access"));

        let secret = store
            .read(account_id, CredentialRole::OAuthRefreshToken)
            .unwrap();
        let debug = format!("{:?}", secret);
        assert!(!debug.contains("super-secret-refresh"));
    }

    #[test]
    fn wait_for_callback_with_simulated_request() {
        let (listener, port) = bind_redirect_listener().unwrap();

        // Simulate a browser redirect in a background thread
        let handle = std::thread::spawn(move || {
            let mut stream = std::net::TcpStream::connect(format!("127.0.0.1:{port}")).unwrap();
            let request =
                "GET /callback?code=test-auth-code&state=test-state HTTP/1.1\r\nHost: 127.0.0.1\r\n\r\n";
            stream.write_all(request.as_bytes()).unwrap();

            let mut response = String::new();
            stream.read_to_string(&mut response).unwrap();
            response
        });

        let params = wait_for_callback(listener).unwrap();
        assert_eq!(params.code, "test-auth-code");
        assert_eq!(params.state, "test-state");

        let response = handle.join().unwrap();
        assert!(response.contains("200 OK"));
        assert!(response.contains("Authorization complete"));
    }

    #[test]
    fn wait_for_callback_with_error_response() {
        let (listener, port) = bind_redirect_listener().unwrap();

        let handle = std::thread::spawn(move || {
            let mut stream = std::net::TcpStream::connect(format!("127.0.0.1:{port}")).unwrap();
            let request = "GET /callback?error=access_denied HTTP/1.1\r\nHost: 127.0.0.1\r\n\r\n";
            stream.write_all(request.as_bytes()).unwrap();

            let mut response = String::new();
            stream.read_to_string(&mut response).unwrap();
            response
        });

        let result = wait_for_callback(listener);
        assert!(matches!(result, Err(OAuthFlowError::CallbackError(_))));

        let response = handle.join().unwrap();
        assert!(response.contains("400 Bad Request"));
    }
}
