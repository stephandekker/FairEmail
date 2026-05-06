//! Automatic OAuth2 token refresh service (FR-15, FR-16, FR-17, FR-18, FR-19, FR-20).
//!
//! Performs the HTTP token-refresh request and atomically updates the
//! credential store. Serializes concurrent refresh attempts per account
//! via a mutex map so that at most one refresh is in flight per account.
//!
//! Classifies errors as transient or permanent (FR-17, FR-18) and retries
//! transient failures with exponential backoff up to 90 seconds (NFR-2).

use std::collections::HashMap;
use std::io::{Read, Write};
use std::sync::Mutex;
use std::time::SystemTime;

use uuid::Uuid;

use crate::core::credential_store::{CredentialRole, CredentialStore, SecretValue};
use crate::core::oauth_flow::compute_expiry_epoch;
use crate::core::token_refresh::{
    classify_refresh_error, may_refresh_now, parse_refresh_response_json, retry_backoff,
    should_refresh_token, RefreshErrorKind, RefreshOutcome, RefreshTokenParams, RETRY_BACKOFF_SECS,
};

/// Per-account refresh state used to serialize concurrent refreshes (FR-19, N-3).
struct AccountRefreshState {
    /// Whether a refresh is currently in flight.
    in_progress: bool,
    /// Unix epoch of the last successful refresh.
    last_refresh_epoch: Option<u64>,
    /// Whether this account requires re-authorization (FR-18).
    needs_reauth: bool,
}

/// Error from the HTTP refresh request, carrying status code for classification.
struct HttpRefreshError {
    /// HTTP status code (0 if no HTTP response was received, e.g. network error).
    status_code: u16,
    /// The response body (for error classification).
    body: String,
    /// Human-readable error description.
    message: String,
}

/// Manages automatic token refresh for all OAuth2 accounts.
///
/// Thread-safe: the inner mutex map serializes refreshes per account (FR-19).
pub(crate) struct TokenRefreshManager {
    state: Mutex<HashMap<Uuid, AccountRefreshState>>,
}

impl TokenRefreshManager {
    pub(crate) fn new() -> Self {
        Self {
            state: Mutex::new(HashMap::new()),
        }
    }

    /// Check whether an account has been marked as needing re-authorization.
    pub(crate) fn needs_reauth(&self, account_id: Uuid) -> bool {
        self.state
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .get(&account_id)
            .map(|s| s.needs_reauth)
            .unwrap_or(false)
    }

    /// Clear the re-authorization flag for an account (after user re-authorizes).
    pub(crate) fn clear_reauth(&self, account_id: Uuid) {
        let mut map = self.state.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(entry) = map.get_mut(&account_id) {
            entry.needs_reauth = false;
        }
    }

    /// Ensure the access token for `account_id` is fresh.
    ///
    /// If the token needs refreshing and no other refresh is in flight for
    /// this account, performs the refresh synchronously (blocking I/O) and
    /// atomically updates the credential store (FR-20).
    ///
    /// On transient failures, retries with exponential backoff up to 90 seconds
    /// (NFR-2, FR-17). During retries, the last valid access token remains
    /// available in the credential store for cached/local operations (AC-8).
    ///
    /// On permanent failures (revoked token, invalid_grant), marks the account
    /// as requiring re-authorization (FR-18).
    pub(crate) fn ensure_fresh_token(
        &self,
        account_id: Uuid,
        token_url: &str,
        client_id: &str,
        credential_store: &dyn CredentialStore,
    ) -> RefreshOutcome {
        // Read current expiry to decide whether refresh is needed.
        let expiry_epoch = read_expiry(credential_store, account_id);

        if !should_refresh_token(expiry_epoch) {
            return RefreshOutcome::Skipped;
        }

        // Try to acquire the per-account refresh slot.
        {
            let mut map = self.state.lock().unwrap_or_else(|e| e.into_inner());
            let entry = map.entry(account_id).or_insert(AccountRefreshState {
                in_progress: false,
                last_refresh_epoch: None,
                needs_reauth: false,
            });

            if entry.in_progress {
                // Another thread is already refreshing — use existing token (US-9).
                return RefreshOutcome::Skipped;
            }

            if entry.needs_reauth {
                return RefreshOutcome::NeedsReauthorization {
                    reason: "This account requires re-authorization. Please sign in again."
                        .to_string(),
                };
            }

            if !may_refresh_now(entry.last_refresh_epoch) {
                // Too soon since the last refresh — back off.
                return RefreshOutcome::Skipped;
            }

            entry.in_progress = true;
        }

        // Perform the refresh with retry outside the lock so other accounts
        // aren't blocked.
        let result = self.do_refresh_with_retry(account_id, token_url, client_id, credential_store);

        // Release the slot, record timestamp on success, mark re-auth on permanent failure.
        {
            let mut map = self.state.lock().unwrap_or_else(|e| e.into_inner());
            if let Some(entry) = map.get_mut(&account_id) {
                entry.in_progress = false;
                match &result {
                    RefreshOutcome::Refreshed => {
                        entry.last_refresh_epoch = Some(now_epoch());
                        entry.needs_reauth = false;
                    }
                    RefreshOutcome::NeedsReauthorization { .. } => {
                        entry.needs_reauth = true;
                    }
                    _ => {}
                }
            }
        }

        result
    }

    /// Attempt refresh with retry for transient errors (FR-17, NFR-2).
    fn do_refresh_with_retry(
        &self,
        account_id: Uuid,
        token_url: &str,
        client_id: &str,
        credential_store: &dyn CredentialStore,
    ) -> RefreshOutcome {
        // Read the refresh token from the credential store.
        let refresh_token =
            match credential_store.read(account_id, CredentialRole::OAuthRefreshToken) {
                Ok(rt) => rt,
                Err(e) => {
                    return RefreshOutcome::NeedsReauthorization {
                        reason: format!("Could not read refresh token: {e}. Please sign in again."),
                    };
                }
            };

        let params = RefreshTokenParams {
            token_url: token_url.to_string(),
            client_id: client_id.to_string(),
            refresh_token: refresh_token.expose().to_string(),
        };

        for attempt in 0..RETRY_BACKOFF_SECS.len() {
            match execute_refresh_request_classified(&params) {
                Ok(response_body) => {
                    return self.apply_refresh_response(
                        account_id,
                        &response_body,
                        credential_store,
                    );
                }
                Err(err) => {
                    let kind = classify_refresh_error(err.status_code, &err.body);
                    match kind {
                        RefreshErrorKind::Permanent => {
                            return RefreshOutcome::NeedsReauthorization {
                                reason: format!(
                                    "Your email provider rejected the token refresh ({}). \
                                     Please sign in again to re-authorize your account.",
                                    err.message
                                ),
                            };
                        }
                        RefreshErrorKind::RateLimited => {
                            return RefreshOutcome::RateLimited {
                                reason: "Your email provider is temporarily limiting requests. \
                                     Please wait a few minutes and try again, or re-authorize \
                                     your account if this persists."
                                    .to_string(),
                            };
                        }
                        RefreshErrorKind::Transient => {
                            // Last attempt — transient timeout exhausted (NFR-2).
                            if attempt + 1 >= RETRY_BACKOFF_SECS.len() {
                                return RefreshOutcome::NeedsReauthorization {
                                    reason: format!(
                                        "Token refresh failed after retrying for 90 seconds ({}). \
                                         Please check your internet connection or re-authorize \
                                         your account.",
                                        err.message
                                    ),
                                };
                            }
                            // Sleep before next retry. The last valid access token
                            // remains in the credential store during this time (AC-8).
                            std::thread::sleep(retry_backoff(attempt));
                        }
                    }
                }
            }
        }

        // Should not be reached, but handle gracefully.
        RefreshOutcome::NeedsReauthorization {
            reason: "Token refresh failed unexpectedly. Please sign in again.".to_string(),
        }
    }

    /// Parse and apply a successful refresh response to the credential store.
    fn apply_refresh_response(
        &self,
        account_id: Uuid,
        response_body: &str,
        credential_store: &dyn CredentialStore,
    ) -> RefreshOutcome {
        let parsed = match parse_refresh_response_json(response_body) {
            Ok(p) => p,
            Err(e) => {
                return RefreshOutcome::NeedsReauthorization {
                    reason: format!(
                        "Could not parse token refresh response: {e}. Please sign in again."
                    ),
                };
            }
        };

        // Atomically update all token fields in the credential store (FR-20).
        if let Err(e) = credential_store.write(
            account_id,
            CredentialRole::ImapPassword,
            &SecretValue::new(parsed.access_token),
        ) {
            return RefreshOutcome::NeedsReauthorization {
                reason: format!("Could not store access token: {e}. Please sign in again."),
            };
        }

        // Update refresh token only if the provider rotated it.
        if let Some(ref new_rt) = parsed.refresh_token {
            if !new_rt.is_empty() {
                if let Err(e) = credential_store.write(
                    account_id,
                    CredentialRole::OAuthRefreshToken,
                    &SecretValue::new(new_rt.clone()),
                ) {
                    return RefreshOutcome::NeedsReauthorization {
                        reason: format!(
                            "Could not store refresh token: {e}. Please sign in again."
                        ),
                    };
                }
            }
        }

        // Update expiry.
        if let Some(expiry_epoch) = compute_expiry_epoch(parsed.expires_in) {
            if let Err(e) = credential_store.write(
                account_id,
                CredentialRole::OAuthTokenExpiry,
                &SecretValue::new(expiry_epoch.to_string()),
            ) {
                return RefreshOutcome::NeedsReauthorization {
                    reason: format!("Could not store token expiry: {e}. Please sign in again."),
                };
            }
        }

        RefreshOutcome::Refreshed
    }
}

/// Read the stored token expiry epoch for an account, if any.
fn read_expiry(store: &dyn CredentialStore, account_id: Uuid) -> Option<u64> {
    store
        .read(account_id, CredentialRole::OAuthTokenExpiry)
        .ok()
        .and_then(|v| v.expose().parse::<u64>().ok())
}

/// Perform the HTTP POST to the token endpoint for a refresh grant.
///
/// Returns the response body on success, or an `HttpRefreshError` with
/// the HTTP status code and body for error classification (FR-17).
fn execute_refresh_request_classified(
    params: &RefreshTokenParams,
) -> Result<String, HttpRefreshError> {
    let stripped = match params.token_url.strip_prefix("https://") {
        Some(s) => s,
        None => {
            return Err(HttpRefreshError {
                status_code: 0,
                body: String::new(),
                message: "Token URL must use HTTPS".to_string(),
            });
        }
    };

    let (host_and_port, path) = match stripped.find('/') {
        Some(i) => (&stripped[..i], &stripped[i..]),
        None => (stripped, "/"),
    };

    let (host, port) = match host_and_port.rsplit_once(':') {
        Some((h, p)) => (h, p.parse::<u16>().unwrap_or(443)),
        None => (host_and_port, 443u16),
    };

    let body = params.form_body();

    let tcp = std::net::TcpStream::connect((host, port)).map_err(|e| HttpRefreshError {
        status_code: 0,
        body: String::new(),
        message: format!("Connect failed: {e}"),
    })?;

    tcp.set_read_timeout(Some(std::time::Duration::from_secs(30)))
        .ok();

    let connector = native_tls::TlsConnector::new().map_err(|e| HttpRefreshError {
        status_code: 0,
        body: String::new(),
        message: format!("TLS init failed: {e}"),
    })?;

    let mut tls = connector.connect(host, tcp).map_err(|e| HttpRefreshError {
        status_code: 0,
        body: String::new(),
        message: format!("TLS handshake failed: {e}"),
    })?;

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
        .map_err(|e| HttpRefreshError {
            status_code: 0,
            body: String::new(),
            message: format!("Write failed: {e}"),
        })?;

    let mut response = String::new();
    tls.read_to_string(&mut response)
        .map_err(|e| HttpRefreshError {
            status_code: 0,
            body: String::new(),
            message: format!("Read failed: {e}"),
        })?;

    let (headers, resp_body) = response
        .split_once("\r\n\r\n")
        .ok_or_else(|| HttpRefreshError {
            status_code: 0,
            body: String::new(),
            message: "Invalid HTTP response".to_string(),
        })?;

    let status_line = headers.lines().next().unwrap_or("");
    let status_code: u16 = status_line
        .split_whitespace()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);

    if !(200..300).contains(&status_code) {
        return Err(HttpRefreshError {
            status_code,
            body: resp_body.to_string(),
            message: format!("HTTP {status_code}"),
        });
    }

    Ok(resp_body.to_string())
}

fn now_epoch() -> u64 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::token_refresh::MIN_REFRESH_INTERVAL_SECS;
    use crate::services::memory_credential_store::MemoryCredentialStore;

    fn setup_store_with_tokens(
        store: &MemoryCredentialStore,
        account_id: Uuid,
        expiry_from_now_secs: Option<i64>,
    ) {
        store
            .write(
                account_id,
                CredentialRole::ImapPassword,
                &SecretValue::new("old-access-token".to_string()),
            )
            .unwrap();
        store
            .write(
                account_id,
                CredentialRole::OAuthRefreshToken,
                &SecretValue::new("my-refresh-token".to_string()),
            )
            .unwrap();
        if let Some(offset) = expiry_from_now_secs {
            let expiry = (now_epoch() as i64 + offset) as u64;
            store
                .write(
                    account_id,
                    CredentialRole::OAuthTokenExpiry,
                    &SecretValue::new(expiry.to_string()),
                )
                .unwrap();
        }
    }

    #[test]
    fn no_refresh_when_token_is_fresh() {
        let store = MemoryCredentialStore::new();
        let id = Uuid::new_v4();
        setup_store_with_tokens(&store, id, Some(3600)); // expires in 1 hour

        let manager = TokenRefreshManager::new();
        let result = manager.ensure_fresh_token(id, "https://example.com/token", "client", &store);
        assert_eq!(result, RefreshOutcome::Skipped);

        // Token unchanged
        assert_eq!(
            store
                .read(id, CredentialRole::ImapPassword)
                .unwrap()
                .expose(),
            "old-access-token"
        );
    }

    #[test]
    fn serialization_prevents_concurrent_refresh() {
        let manager = TokenRefreshManager::new();
        let id = Uuid::new_v4();

        // Manually mark a refresh as in-progress
        {
            let mut map = manager.state.lock().unwrap();
            map.insert(
                id,
                AccountRefreshState {
                    in_progress: true,
                    last_refresh_epoch: None,
                    needs_reauth: false,
                },
            );
        }

        let store = MemoryCredentialStore::new();
        setup_store_with_tokens(&store, id, Some(-60)); // expired

        // Should return Skipped because another refresh is in progress
        let result = manager.ensure_fresh_token(id, "https://example.com/token", "client", &store);
        assert_eq!(result, RefreshOutcome::Skipped);
    }

    #[test]
    fn min_interval_prevents_rapid_refresh() {
        let manager = TokenRefreshManager::new();
        let id = Uuid::new_v4();

        // Record a recent successful refresh
        {
            let mut map = manager.state.lock().unwrap();
            map.insert(
                id,
                AccountRefreshState {
                    in_progress: false,
                    last_refresh_epoch: Some(now_epoch() - 60), // 1 min ago
                    needs_reauth: false,
                },
            );
        }

        let store = MemoryCredentialStore::new();
        setup_store_with_tokens(&store, id, Some(-60)); // expired

        // Should return Skipped because < 15 minutes since last refresh
        let result = manager.ensure_fresh_token(id, "https://example.com/token", "client", &store);
        assert_eq!(result, RefreshOutcome::Skipped);
    }

    #[test]
    fn read_expiry_returns_none_when_missing() {
        let store = MemoryCredentialStore::new();
        let id = Uuid::new_v4();
        assert!(read_expiry(&store, id).is_none());
    }

    #[test]
    fn read_expiry_returns_parsed_value() {
        let store = MemoryCredentialStore::new();
        let id = Uuid::new_v4();
        store
            .write(
                id,
                CredentialRole::OAuthTokenExpiry,
                &SecretValue::new("1700000000".to_string()),
            )
            .unwrap();
        assert_eq!(read_expiry(&store, id), Some(1700000000));
    }

    #[test]
    fn read_expiry_returns_none_for_invalid() {
        let store = MemoryCredentialStore::new();
        let id = Uuid::new_v4();
        store
            .write(
                id,
                CredentialRole::OAuthTokenExpiry,
                &SecretValue::new("not-a-number".to_string()),
            )
            .unwrap();
        assert!(read_expiry(&store, id).is_none());
    }

    #[test]
    fn manager_new_has_empty_state() {
        let manager = TokenRefreshManager::new();
        let map = manager.state.lock().unwrap();
        assert!(map.is_empty());
    }

    #[test]
    fn independent_accounts_dont_block_each_other() {
        let manager = TokenRefreshManager::new();
        let id1 = Uuid::new_v4();
        let id2 = Uuid::new_v4();

        // Mark id1 as in-progress
        {
            let mut map = manager.state.lock().unwrap();
            map.insert(
                id1,
                AccountRefreshState {
                    in_progress: true,
                    last_refresh_epoch: None,
                    needs_reauth: false,
                },
            );
        }

        let store = MemoryCredentialStore::new();
        // id2 has a fresh token — should not be blocked by id1
        setup_store_with_tokens(&store, id2, Some(3600));
        let result = manager.ensure_fresh_token(id2, "https://example.com/token", "client", &store);
        assert_eq!(result, RefreshOutcome::Skipped); // no refresh needed, not blocked
    }

    #[test]
    fn refresh_slot_released_on_failure() {
        let manager = TokenRefreshManager::new();
        let id = Uuid::new_v4();

        let store = MemoryCredentialStore::new();
        setup_store_with_tokens(&store, id, Some(-60)); // expired

        // Allow refresh interval
        {
            let mut map = manager.state.lock().unwrap();
            map.insert(
                id,
                AccountRefreshState {
                    in_progress: false,
                    last_refresh_epoch: Some(now_epoch() - MIN_REFRESH_INTERVAL_SECS - 1),
                    needs_reauth: false,
                },
            );
        }

        // This will fail because we can't actually connect to example.com,
        // but the slot should be released.
        let _ = manager.ensure_fresh_token(id, "https://example.com/token", "client", &store);

        let map = manager.state.lock().unwrap();
        assert!(!map[&id].in_progress);
    }

    #[test]
    fn needs_reauth_false_by_default() {
        let manager = TokenRefreshManager::new();
        let id = Uuid::new_v4();
        assert!(!manager.needs_reauth(id));
    }

    #[test]
    fn needs_reauth_returns_stored_state() {
        let manager = TokenRefreshManager::new();
        let id = Uuid::new_v4();

        {
            let mut map = manager.state.lock().unwrap();
            map.insert(
                id,
                AccountRefreshState {
                    in_progress: false,
                    last_refresh_epoch: None,
                    needs_reauth: true,
                },
            );
        }

        assert!(manager.needs_reauth(id));
    }

    #[test]
    fn clear_reauth_resets_flag() {
        let manager = TokenRefreshManager::new();
        let id = Uuid::new_v4();

        {
            let mut map = manager.state.lock().unwrap();
            map.insert(
                id,
                AccountRefreshState {
                    in_progress: false,
                    last_refresh_epoch: None,
                    needs_reauth: true,
                },
            );
        }

        manager.clear_reauth(id);
        assert!(!manager.needs_reauth(id));
    }

    #[test]
    fn ensure_fresh_token_returns_needs_reauth_when_flagged() {
        let manager = TokenRefreshManager::new();
        let id = Uuid::new_v4();

        let store = MemoryCredentialStore::new();
        setup_store_with_tokens(&store, id, Some(-60)); // expired

        // Mark as needing re-auth
        {
            let mut map = manager.state.lock().unwrap();
            map.insert(
                id,
                AccountRefreshState {
                    in_progress: false,
                    last_refresh_epoch: None,
                    needs_reauth: true,
                },
            );
        }

        let result = manager.ensure_fresh_token(id, "https://example.com/token", "client", &store);
        assert!(matches!(
            result,
            RefreshOutcome::NeedsReauthorization { .. }
        ));
    }

    #[test]
    fn token_unchanged_during_failed_refresh() {
        // AC-8: During transient failures, the last valid access token continues
        // to be used. Since we can't connect to example.com, the refresh will
        // fail but the original token should remain in the store.
        let manager = TokenRefreshManager::new();
        let id = Uuid::new_v4();

        let store = MemoryCredentialStore::new();
        setup_store_with_tokens(&store, id, Some(-60)); // expired

        {
            let mut map = manager.state.lock().unwrap();
            map.insert(
                id,
                AccountRefreshState {
                    in_progress: false,
                    last_refresh_epoch: Some(now_epoch() - MIN_REFRESH_INTERVAL_SECS - 1),
                    needs_reauth: false,
                },
            );
        }

        // This will fail (can't connect to example.com), but token stays.
        let _ = manager.ensure_fresh_token(id, "https://example.com/token", "client", &store);

        assert_eq!(
            store
                .read(id, CredentialRole::ImapPassword)
                .unwrap()
                .expose(),
            "old-access-token"
        );
    }

    #[test]
    fn apply_refresh_response_updates_all_fields() {
        let manager = TokenRefreshManager::new();
        let store = MemoryCredentialStore::new();
        let id = Uuid::new_v4();
        setup_store_with_tokens(&store, id, Some(3600));

        let json =
            r#"{"access_token":"new-access","refresh_token":"new-refresh","expires_in":7200}"#;
        let result = manager.apply_refresh_response(id, json, &store);
        assert_eq!(result, RefreshOutcome::Refreshed);

        assert_eq!(
            store
                .read(id, CredentialRole::ImapPassword)
                .unwrap()
                .expose(),
            "new-access"
        );
        assert_eq!(
            store
                .read(id, CredentialRole::OAuthRefreshToken)
                .unwrap()
                .expose(),
            "new-refresh"
        );
    }

    #[test]
    fn apply_refresh_response_keeps_old_refresh_token_when_none() {
        let manager = TokenRefreshManager::new();
        let store = MemoryCredentialStore::new();
        let id = Uuid::new_v4();
        setup_store_with_tokens(&store, id, Some(3600));

        let json = r#"{"access_token":"new-access","expires_in":3600}"#;
        let result = manager.apply_refresh_response(id, json, &store);
        assert_eq!(result, RefreshOutcome::Refreshed);

        // Refresh token unchanged
        assert_eq!(
            store
                .read(id, CredentialRole::OAuthRefreshToken)
                .unwrap()
                .expose(),
            "my-refresh-token"
        );
    }

    #[test]
    fn apply_refresh_response_returns_needs_reauth_on_invalid_json() {
        let manager = TokenRefreshManager::new();
        let store = MemoryCredentialStore::new();
        let id = Uuid::new_v4();

        let result = manager.apply_refresh_response(id, "not json", &store);
        assert!(matches!(
            result,
            RefreshOutcome::NeedsReauthorization { .. }
        ));
    }
}
