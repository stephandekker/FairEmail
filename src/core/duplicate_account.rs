use crate::core::{Account, AccountValidationError};

/// Duplicate an existing account's configuration into a new, independent account.
///
/// The duplicate:
/// - Gets a fresh unique identifier (new UUID).
/// - Does NOT inherit the source's primary designation.
/// - Does NOT share any mutable state (messages, folders, sync state, error state, quota).
/// - Copies all configuration settings (connection, sync preferences, notifications, etc.).
pub fn duplicate(source: &Account) -> Result<Account, AccountValidationError> {
    let params = source.to_new_account_params();
    Account::new(params)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{
        AccountColor, AuthMethod, EncryptionMode, NewAccountParams, Protocol, SecuritySettings,
        SmtpConfig, SwipeAction, SwipeDefaults, SystemFolders,
    };

    fn source_account() -> Account {
        let params = NewAccountParams {
            display_name: "Work Email".into(),
            protocol: Protocol::Imap,
            host: "imap.example.com".into(),
            port: 993,
            encryption: EncryptionMode::SslTls,
            auth_method: AuthMethod::Plain,
            username: "user@example.com".into(),
            credential: "secret".into(),
            smtp: Some(SmtpConfig {
                host: "smtp.example.com".into(),
                port: 587,
                encryption: EncryptionMode::StartTls,
                auth_method: AuthMethod::Plain,
                username: "user@example.com".into(),
                credential: "secret".into(),
            }),
            pop3_settings: None,
            color: Some(AccountColor::new(1.0, 0.0, 0.0)),
            avatar_path: Some("/tmp/avatar.png".into()),
            category: Some("Work".into()),
            sync_enabled: true,
            on_demand: false,
            polling_interval_minutes: Some(15),
            unmetered_only: true,
            vpn_only: true,
            schedule_exempt: true,
            system_folders: Some(SystemFolders {
                drafts: Some("Drafts".into()),
                sent: Some("Sent".into()),
                archive: Some("Archive".into()),
                trash: Some("Trash".into()),
                junk: Some("Junk".into()),
            }),
            swipe_defaults: Some(SwipeDefaults {
                swipe_left: SwipeAction::Delete,
                swipe_right: SwipeAction::Archive,
                default_move_to: Some("Archive".into()),
            }),
            notifications_enabled: true,
            security_settings: Some(SecuritySettings {
                dnssec: true,
                dane: true,
                insecure: false,
                certificate_fingerprint: Some("ab:cd:ef:01".into()),
                client_certificate: Some("/etc/pki/client.pem".into()),
                auth_realm: Some("example.com".into()),
            }),
            fetch_settings: None,
            keep_alive_settings: None,
        };
        let mut acct = Account::new(params).unwrap();
        // Simulate the source being primary and having state.
        acct.set_primary(true);
        acct.set_error_state(Some("some error".into()));
        acct
    }

    #[test]
    fn duplicate_gets_new_id() {
        let source = source_account();
        let dup = duplicate(&source).unwrap();
        assert_ne!(dup.id(), source.id());
    }

    #[test]
    fn duplicate_does_not_inherit_primary() {
        let source = source_account();
        assert!(source.is_primary());
        let dup = duplicate(&source).unwrap();
        assert!(!dup.is_primary());
    }

    #[test]
    fn duplicate_does_not_inherit_error_state() {
        let source = source_account();
        assert!(source.error_state().is_some());
        let dup = duplicate(&source).unwrap();
        assert!(dup.error_state().is_none());
    }

    #[test]
    fn duplicate_does_not_inherit_quota() {
        let mut source = source_account();
        source.set_quota(Some(
            crate::core::account::QuotaInfo::new(500, 1000).unwrap(),
        ));
        let dup = duplicate(&source).unwrap();
        assert!(dup.quota().is_none());
    }

    #[test]
    fn duplicate_copies_all_config_fields() {
        let source = source_account();
        let dup = duplicate(&source).unwrap();

        assert_eq!(dup.display_name(), source.display_name());
        assert_eq!(dup.protocol(), source.protocol());
        assert_eq!(dup.host(), source.host());
        assert_eq!(dup.port(), source.port());
        assert_eq!(dup.encryption(), source.encryption());
        assert_eq!(dup.auth_method(), source.auth_method());
        assert_eq!(dup.username(), source.username());
        assert_eq!(dup.credential(), source.credential());
        assert_eq!(dup.color(), source.color());
        assert_eq!(dup.avatar_path(), source.avatar_path());
        assert_eq!(dup.category(), source.category());
        assert_eq!(dup.sync_enabled(), source.sync_enabled());
        assert_eq!(dup.on_demand(), source.on_demand());
        assert_eq!(
            dup.polling_interval_minutes(),
            source.polling_interval_minutes()
        );
        assert_eq!(dup.unmetered_only(), source.unmetered_only());
        assert_eq!(dup.vpn_only(), source.vpn_only());
        assert_eq!(dup.schedule_exempt(), source.schedule_exempt());
        assert_eq!(dup.notifications_enabled(), source.notifications_enabled());

        // SMTP config
        let dup_smtp = dup.smtp().unwrap();
        let src_smtp = source.smtp().unwrap();
        assert_eq!(dup_smtp.host, src_smtp.host);
        assert_eq!(dup_smtp.port, src_smtp.port);

        // System folders
        assert_eq!(
            dup.system_folders().unwrap().drafts,
            source.system_folders().unwrap().drafts
        );

        // Swipe defaults
        assert_eq!(
            dup.swipe_defaults().unwrap().swipe_left,
            source.swipe_defaults().unwrap().swipe_left
        );
    }

    #[test]
    fn duplicate_is_independent_of_source() {
        let mut source = source_account();
        let dup = duplicate(&source).unwrap();

        // Mutating the source does not affect the duplicate.
        source.set_sync_enabled(false);
        assert!(dup.sync_enabled());
    }
}
