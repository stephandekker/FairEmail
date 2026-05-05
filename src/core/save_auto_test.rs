use crate::core::account::{Account, UpdateAccountParams};

/// Determines whether an automatic connection test should be performed
/// before saving a new account (FR-42, US-22).
///
/// For new accounts, sync is always enabled and there is no prior state,
/// so the only question is whether the user already ran a successful test.
pub fn should_auto_test_new_account(test_passed_in_session: bool) -> bool {
    !test_passed_in_session
}

/// Determines whether an automatic connection test should be performed
/// before saving an existing account (FR-42, NFR-7, US-22).
///
/// Returns `false` when:
/// - The user already ran a successful test in this editing session, OR
/// - Sync is not enabled on the updated params, OR
/// - The connection-relevant parameters have not changed (idempotent save).
pub fn should_auto_test_existing_account(
    account: &Account,
    params: &UpdateAccountParams,
    test_passed_in_session: bool,
) -> bool {
    if test_passed_in_session {
        return false;
    }
    if !params.sync_enabled {
        return false;
    }
    connection_params_changed(account, params)
}

/// Checks whether any connection-relevant parameters differ between the
/// current account state and the proposed update. Non-connection fields
/// (display name, colour, category, swipe defaults, etc.) are ignored.
fn connection_params_changed(account: &Account, params: &UpdateAccountParams) -> bool {
    if account.protocol() != params.protocol {
        return true;
    }
    if account.host() != params.host {
        return true;
    }
    if account.port() != params.port {
        return true;
    }
    if account.encryption() != params.encryption {
        return true;
    }
    if account.auth_method() != params.auth_method {
        return true;
    }
    if account.username() != params.username {
        return true;
    }
    if account.credential() != params.credential {
        return true;
    }
    // Check SMTP config changes.
    match (account.smtp(), &params.smtp) {
        (None, None) => {}
        (Some(_), None) | (None, Some(_)) => return true,
        (Some(existing), Some(new)) => {
            if existing.host != new.host
                || existing.port != new.port
                || existing.encryption != new.encryption
                || existing.auth_method != new.auth_method
                || existing.username != new.username
                || existing.credential != new.credential
            {
                return true;
            }
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::account::{
        AuthMethod, EncryptionMode, NewAccountParams, Protocol, SmtpConfig,
    };

    fn make_account() -> Account {
        Account::new(NewAccountParams {
            display_name: "Test".into(),
            protocol: Protocol::Imap,
            host: "imap.example.com".into(),
            port: 993,
            encryption: EncryptionMode::SslTls,
            auth_method: AuthMethod::Plain,
            username: "user@example.com".into(),
            credential: "password".into(),
            smtp: None,
            pop3_settings: None,
            color: None,
            avatar_path: None,
            category: None,
            sync_enabled: true,
            on_demand: false,
            polling_interval_minutes: None,
            unmetered_only: false,
            vpn_only: false,
            schedule_exempt: false,
            system_folders: None,
            swipe_defaults: None,
            notifications_enabled: true,
            security_settings: None,
            fetch_settings: None,
            keep_alive_settings: None,
        })
        .unwrap()
    }

    fn matching_update_params() -> UpdateAccountParams {
        UpdateAccountParams {
            display_name: "Test".into(),
            protocol: Protocol::Imap,
            host: "imap.example.com".into(),
            port: 993,
            encryption: EncryptionMode::SslTls,
            auth_method: AuthMethod::Plain,
            username: "user@example.com".into(),
            credential: "password".into(),
            smtp: None,
            pop3_settings: None,
            color: None,
            avatar_path: None,
            category: None,
            sync_enabled: true,
            on_demand: false,
            polling_interval_minutes: None,
            unmetered_only: false,
            vpn_only: false,
            schedule_exempt: false,
            system_folders: None,
            swipe_defaults: None,
            notifications_enabled: true,
            security_settings: None,
            fetch_settings: None,
            keep_alive_settings: None,
        }
    }

    // -- should_auto_test_new_account --

    #[test]
    fn new_account_needs_test_when_not_passed() {
        assert!(should_auto_test_new_account(false));
    }

    #[test]
    fn new_account_skips_test_when_passed() {
        assert!(!should_auto_test_new_account(true));
    }

    // -- should_auto_test_existing_account --

    #[test]
    fn existing_account_skips_when_test_passed() {
        let acct = make_account();
        let mut params = matching_update_params();
        params.host = "different.example.com".into();
        assert!(!should_auto_test_existing_account(&acct, &params, true));
    }

    #[test]
    fn existing_account_skips_when_sync_disabled() {
        let acct = make_account();
        let mut params = matching_update_params();
        params.sync_enabled = false;
        params.host = "different.example.com".into();
        assert!(!should_auto_test_existing_account(&acct, &params, false));
    }

    #[test]
    fn existing_account_skips_when_params_unchanged() {
        let acct = make_account();
        let params = matching_update_params();
        // Unchanged params = idempotent save (NFR-7).
        assert!(!should_auto_test_existing_account(&acct, &params, false));
    }

    #[test]
    fn existing_account_needs_test_when_host_changed() {
        let acct = make_account();
        let mut params = matching_update_params();
        params.host = "imap2.example.com".into();
        assert!(should_auto_test_existing_account(&acct, &params, false));
    }

    #[test]
    fn existing_account_needs_test_when_port_changed() {
        let acct = make_account();
        let mut params = matching_update_params();
        params.port = 143;
        assert!(should_auto_test_existing_account(&acct, &params, false));
    }

    #[test]
    fn existing_account_needs_test_when_encryption_changed() {
        let acct = make_account();
        let mut params = matching_update_params();
        params.encryption = EncryptionMode::StartTls;
        assert!(should_auto_test_existing_account(&acct, &params, false));
    }

    #[test]
    fn existing_account_needs_test_when_credential_changed() {
        let acct = make_account();
        let mut params = matching_update_params();
        params.credential = "new-password".into();
        assert!(should_auto_test_existing_account(&acct, &params, false));
    }

    #[test]
    fn existing_account_needs_test_when_smtp_added() {
        let acct = make_account();
        let mut params = matching_update_params();
        params.smtp = Some(SmtpConfig {
            host: "smtp.example.com".into(),
            port: 587,
            encryption: EncryptionMode::StartTls,
            auth_method: AuthMethod::Plain,
            username: "user@example.com".into(),
            credential: "password".into(),
        });
        assert!(should_auto_test_existing_account(&acct, &params, false));
    }

    #[test]
    fn existing_account_no_test_when_only_display_name_changed() {
        let acct = make_account();
        let mut params = matching_update_params();
        params.display_name = "New Name".into();
        // Non-connection field change should not trigger a test.
        assert!(!should_auto_test_existing_account(&acct, &params, false));
    }

    #[test]
    fn existing_account_needs_test_when_protocol_changed() {
        let acct = make_account();
        let mut params = matching_update_params();
        params.protocol = Protocol::Pop3;
        assert!(should_auto_test_existing_account(&acct, &params, false));
    }

    #[test]
    fn existing_account_needs_test_when_auth_method_changed() {
        let acct = make_account();
        let mut params = matching_update_params();
        params.auth_method = AuthMethod::OAuth2;
        assert!(should_auto_test_existing_account(&acct, &params, false));
    }

    // -- connection_params_changed with SMTP --

    #[test]
    fn smtp_host_change_detected() {
        let acct = Account::new(NewAccountParams {
            display_name: "Test".into(),
            protocol: Protocol::Imap,
            host: "imap.example.com".into(),
            port: 993,
            encryption: EncryptionMode::SslTls,
            auth_method: AuthMethod::Plain,
            username: "user@example.com".into(),
            credential: "password".into(),
            smtp: Some(SmtpConfig {
                host: "smtp.example.com".into(),
                port: 587,
                encryption: EncryptionMode::StartTls,
                auth_method: AuthMethod::Plain,
                username: "user@example.com".into(),
                credential: "password".into(),
            }),
            pop3_settings: None,
            color: None,
            avatar_path: None,
            category: None,
            sync_enabled: true,
            on_demand: false,
            polling_interval_minutes: None,
            unmetered_only: false,
            vpn_only: false,
            schedule_exempt: false,
            system_folders: None,
            swipe_defaults: None,
            notifications_enabled: true,
            security_settings: None,
            fetch_settings: None,
            keep_alive_settings: None,
        })
        .unwrap();

        let mut params = UpdateAccountParams {
            display_name: "Test".into(),
            protocol: Protocol::Imap,
            host: "imap.example.com".into(),
            port: 993,
            encryption: EncryptionMode::SslTls,
            auth_method: AuthMethod::Plain,
            username: "user@example.com".into(),
            credential: "password".into(),
            smtp: Some(SmtpConfig {
                host: "smtp2.example.com".into(),
                port: 587,
                encryption: EncryptionMode::StartTls,
                auth_method: AuthMethod::Plain,
                username: "user@example.com".into(),
                credential: "password".into(),
            }),
            pop3_settings: None,
            color: None,
            avatar_path: None,
            category: None,
            sync_enabled: true,
            on_demand: false,
            polling_interval_minutes: None,
            unmetered_only: false,
            vpn_only: false,
            schedule_exempt: false,
            system_folders: None,
            swipe_defaults: None,
            notifications_enabled: true,
            security_settings: None,
            fetch_settings: None,
            keep_alive_settings: None,
        };

        assert!(should_auto_test_existing_account(&acct, &params, false));

        // Fix the SMTP host to match — should no longer trigger.
        params.smtp.as_mut().unwrap().host = "smtp.example.com".into();
        assert!(!should_auto_test_existing_account(&acct, &params, false));
    }
}
