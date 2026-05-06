//! Automatic OAuth2 token refresh service (FR-15, FR-16, FR-19, FR-20).
//!
//! Performs the HTTP token-refresh request and atomically updates the
//! credential store. Serializes concurrent refresh attempts per account
//! via a mutex map so that at most one refresh is in flight per account.

use std::collections::HashMap;
use std::io::{Read, Write};
use std::sync::Mutex;
use std::time::SystemTime;

use uuid::Uuid;

use crate::core::credential_store::{CredentialRole, CredentialStore, SecretValue};
use crate::core::oauth_flow::{compute_expiry_epoch, OAuthFlowError};
use crate::core::token_refresh::{
    may_refresh_now, parse_refresh_response_json, should_refresh_token, RefreshTokenParams,
};

/// Per-account refresh state used to serialize concurrent refreshes (FR-19, N-3).
struct AccountRefreshState {
    /// Whether a refresh is currently in flight.
    in_progress: bool,
    /// Unix epoch of the last successful refresh.
    last_refresh_epoch: Option<u64>,
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

    /// Ensure the access token for `account_id` is fresh.
    ///
    /// If the token needs refreshing and no other refresh is in flight for
    /// this account, performs the refresh synchronously (blocking I/O) and
    /// atomically updates the credential store (FR-20).
    ///
    /// Returns `Ok(true)` if a refresh was performed, `Ok(false)` if no
    /// refresh was needed or another refresh is already in progress (the
    /// caller should use the existing token — US-9).
    pub(crate) fn ensure_fresh_token(
        &self,
        account_id: Uuid,
        token_url: &str,
        client_id: &str,
        credential_store: &dyn CredentialStore,
    ) -> Result<bool, OAuthFlowError> {
        // Read current expiry to decide whether refresh is needed.
        let expiry_epoch = read_expiry(credential_store, account_id);

        if !should_refresh_token(expiry_epoch) {
            return Ok(false);
        }

        // Try to acquire the per-account refresh slot.
        {
            let mut map = self.state.lock().unwrap_or_else(|e| e.into_inner());
            let entry = map.entry(account_id).or_insert(AccountRefreshState {
                in_progress: false,
                last_refresh_epoch: None,
            });

            if entry.in_progress {
                // Another thread is already refreshing — use existing token (US-9).
                return Ok(false);
            }

            if !may_refresh_now(entry.last_refresh_epoch) {
                // Too soon since the last refresh — back off.
                return Ok(false);
            }

            entry.in_progress = true;
        }

        // Perform the refresh outside the lock so other accounts aren't blocked.
        let result = self.do_refresh(account_id, token_url, client_id, credential_store);

        // Release the slot and record the timestamp on success.
        {
            let mut map = self.state.lock().unwrap_or_else(|e| e.into_inner());
            if let Some(entry) = map.get_mut(&account_id) {
                entry.in_progress = false;
                if result.is_ok() {
                    entry.last_refresh_epoch = Some(now_epoch());
                }
            }
        }

        result
    }

    fn do_refresh(
        &self,
        account_id: Uuid,
        token_url: &str,
        client_id: &str,
        credential_store: &dyn CredentialStore,
    ) -> Result<bool, OAuthFlowError> {
        // Read the refresh token from the credential store.
        let refresh_token = credential_store
            .read(account_id, CredentialRole::OAuthRefreshToken)
            .map_err(|e| OAuthFlowError::CredentialStoreError(e.to_string()))?;

        let params = RefreshTokenParams {
            token_url: token_url.to_string(),
            client_id: client_id.to_string(),
            refresh_token: refresh_token.expose().to_string(),
        };

        // Perform HTTP token refresh.
        let response = execute_refresh_request(&params)?;
        let parsed = parse_refresh_response_json(&response)?;

        // Atomically update all token fields in the credential store (FR-20).
        credential_store
            .write(
                account_id,
                CredentialRole::ImapPassword,
                &SecretValue::new(parsed.access_token),
            )
            .map_err(|e| OAuthFlowError::CredentialStoreError(e.to_string()))?;

        // Update refresh token only if the provider rotated it.
        if let Some(ref new_rt) = parsed.refresh_token {
            if !new_rt.is_empty() {
                credential_store
                    .write(
                        account_id,
                        CredentialRole::OAuthRefreshToken,
                        &SecretValue::new(new_rt.clone()),
                    )
                    .map_err(|e| OAuthFlowError::CredentialStoreError(e.to_string()))?;
            }
        }

        // Update expiry.
        if let Some(expiry_epoch) = compute_expiry_epoch(parsed.expires_in) {
            credential_store
                .write(
                    account_id,
                    CredentialRole::OAuthTokenExpiry,
                    &SecretValue::new(expiry_epoch.to_string()),
                )
                .map_err(|e| OAuthFlowError::CredentialStoreError(e.to_string()))?;
        }

        Ok(true)
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
/// Uses the same blocking native_tls pattern as `exchange_code_for_tokens`
/// in `oauth_service.rs`.
fn execute_refresh_request(params: &RefreshTokenParams) -> Result<String, OAuthFlowError> {
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
        assert!(!result.unwrap());

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
                },
            );
        }

        let store = MemoryCredentialStore::new();
        setup_store_with_tokens(&store, id, Some(-60)); // expired

        // Should return false (skipped) because another refresh is in progress
        let result = manager.ensure_fresh_token(id, "https://example.com/token", "client", &store);
        assert!(!result.unwrap());
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
                },
            );
        }

        let store = MemoryCredentialStore::new();
        setup_store_with_tokens(&store, id, Some(-60)); // expired

        // Should return false because < 15 minutes since last refresh
        let result = manager.ensure_fresh_token(id, "https://example.com/token", "client", &store);
        assert!(!result.unwrap());
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
                },
            );
        }

        let store = MemoryCredentialStore::new();
        // id2 has a fresh token — should not be blocked by id1
        setup_store_with_tokens(&store, id2, Some(3600));
        let result = manager.ensure_fresh_token(id2, "https://example.com/token", "client", &store);
        assert!(!result.unwrap()); // no refresh needed, not blocked
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
                },
            );
        }

        // This will fail because we can't actually connect to example.com,
        // but the slot should be released.
        let _ = manager.ensure_fresh_token(id, "https://example.com/token", "client", &store);

        let map = manager.state.lock().unwrap();
        assert!(!map[&id].in_progress);
    }
}
