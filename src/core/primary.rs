use uuid::Uuid;

use super::Account;

/// Errors from primary-account designation operations.
#[derive(Debug, Clone, thiserror::Error)]
pub enum PrimaryDesignationError {
    #[error("account not found: {0}")]
    NotFound(Uuid),
    #[error("account {0} is not eligible for primary: synchronization is disabled")]
    SyncDisabled(Uuid),
}

/// Designate the account with the given `id` as primary (FR-24, FR-25, FR-26).
/// Only sync-enabled accounts are eligible. The previous primary (if any)
/// is automatically demoted. Returns the IDs of accounts whose primary status changed.
pub fn set_primary(
    accounts: &mut [Account],
    id: Uuid,
) -> Result<Vec<Uuid>, PrimaryDesignationError> {
    // Find the target account and validate eligibility.
    let target_idx = accounts
        .iter()
        .position(|a| a.id() == id)
        .ok_or(PrimaryDesignationError::NotFound(id))?;

    if !accounts[target_idx].sync_enabled() {
        return Err(PrimaryDesignationError::SyncDisabled(id));
    }

    let mut changed = Vec::new();

    // Demote current primary (FR-26).
    for acct in accounts.iter_mut() {
        if acct.is_primary() && acct.id() != id {
            acct.set_primary(false);
            changed.push(acct.id());
        }
    }

    // Promote the target.
    if !accounts[target_idx].is_primary() {
        accounts[target_idx].set_primary(true);
        changed.push(id);
    }

    Ok(changed)
}

/// Auto-designate primary when a new account is added (FR-28).
/// If no account is currently primary and the new account has sync enabled,
/// it becomes primary. Returns true if the account was made primary.
pub fn auto_designate_on_add(accounts: &mut [Account], new_id: Uuid) -> bool {
    let has_primary = accounts.iter().any(|a| a.is_primary());
    if has_primary {
        return false;
    }
    if let Some(acct) = accounts.iter_mut().find(|a| a.id() == new_id) {
        if acct.sync_enabled() {
            acct.set_primary(true);
            return true;
        }
    }
    false
}

/// Revoke primary designation if sync is disabled on the primary account (FR-32).
/// Call this after updating an account's sync_enabled status.
/// Returns the ID of the account that was demoted, if any.
pub fn revoke_if_sync_disabled(accounts: &mut [Account]) -> Option<Uuid> {
    for acct in accounts.iter_mut() {
        if acct.is_primary() && !acct.sync_enabled() {
            acct.set_primary(false);
            return Some(acct.id());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{
        AuthMethod, EncryptionMode, NewAccountParams, Protocol, UpdateAccountParams,
    };

    fn make_synced_account(name: &str) -> Account {
        Account::new(NewAccountParams {
            display_name: name.into(),
            protocol: Protocol::Imap,
            host: "imap.example.com".into(),
            port: 993,
            encryption: EncryptionMode::SslTls,
            auth_method: AuthMethod::Plain,
            username: "user@example.com".into(),
            credential: "secret".into(),
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
            oauth_tenant: None,
            shared_mailbox: None,
        })
        .unwrap()
    }

    fn make_unsynced_account(name: &str) -> Account {
        Account::new(NewAccountParams {
            display_name: name.into(),
            protocol: Protocol::Imap,
            host: "imap.example.com".into(),
            port: 993,
            encryption: EncryptionMode::SslTls,
            auth_method: AuthMethod::Plain,
            username: "user@example.com".into(),
            credential: "secret".into(),
            smtp: None,
            pop3_settings: None,
            color: None,
            avatar_path: None,
            category: None,
            sync_enabled: false,
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
            oauth_tenant: None,
            shared_mailbox: None,
        })
        .unwrap()
    }

    #[test]
    fn set_primary_promotes_account() {
        let mut accounts = vec![make_synced_account("A"), make_synced_account("B")];
        let id = accounts[1].id();
        let changed = set_primary(&mut accounts, id).unwrap();
        assert!(accounts[1].is_primary());
        assert!(!accounts[0].is_primary());
        assert_eq!(changed, vec![id]);
    }

    #[test]
    fn set_primary_demotes_previous() {
        let mut accounts = vec![make_synced_account("A"), make_synced_account("B")];
        let id_a = accounts[0].id();
        let id_b = accounts[1].id();
        set_primary(&mut accounts, id_a).unwrap();
        assert!(accounts[0].is_primary());

        let changed = set_primary(&mut accounts, id_b).unwrap();
        assert!(!accounts[0].is_primary());
        assert!(accounts[1].is_primary());
        assert!(changed.contains(&id_a));
        assert!(changed.contains(&id_b));
    }

    #[test]
    fn set_primary_rejects_sync_disabled() {
        let mut accounts = vec![make_unsynced_account("A")];
        let id = accounts[0].id();
        let result = set_primary(&mut accounts, id);
        assert!(matches!(
            result,
            Err(PrimaryDesignationError::SyncDisabled(_))
        ));
        assert!(!accounts[0].is_primary());
    }

    #[test]
    fn set_primary_rejects_unknown_id() {
        let mut accounts = vec![make_synced_account("A")];
        let fake_id = Uuid::new_v4();
        let result = set_primary(&mut accounts, fake_id);
        assert!(matches!(result, Err(PrimaryDesignationError::NotFound(_))));
    }

    #[test]
    fn set_primary_idempotent() {
        let mut accounts = vec![make_synced_account("A")];
        let id = accounts[0].id();
        set_primary(&mut accounts, id).unwrap();
        let changed = set_primary(&mut accounts, id).unwrap();
        assert!(accounts[0].is_primary());
        // No change reported when already primary.
        assert!(changed.is_empty());
    }

    #[test]
    fn auto_designate_on_add_first_synced_account() {
        let mut accounts = vec![make_synced_account("A")];
        let id = accounts[0].id();
        let designated = auto_designate_on_add(&mut accounts, id);
        assert!(designated);
        assert!(accounts[0].is_primary());
    }

    #[test]
    fn auto_designate_on_add_skips_when_primary_exists() {
        let mut accounts = vec![make_synced_account("A"), make_synced_account("B")];
        let id_a = accounts[0].id();
        let id_b = accounts[1].id();
        accounts[0].set_primary(true);
        let designated = auto_designate_on_add(&mut accounts, id_b);
        assert!(!designated);
        assert!(accounts[0].is_primary());
        assert!(!accounts[1].is_primary());
        // original primary unchanged
        assert_eq!(accounts[0].id(), id_a);
    }

    #[test]
    fn auto_designate_on_add_skips_unsynced() {
        let mut accounts = vec![make_unsynced_account("A")];
        let id = accounts[0].id();
        let designated = auto_designate_on_add(&mut accounts, id);
        assert!(!designated);
        assert!(!accounts[0].is_primary());
    }

    #[test]
    fn revoke_if_sync_disabled_revokes_primary() {
        let mut accounts = vec![make_synced_account("A")];
        let id = accounts[0].id();
        accounts[0].set_primary(true);
        // Simulate disabling sync via update.
        accounts[0]
            .update(UpdateAccountParams {
                display_name: "A".into(),
                protocol: Protocol::Imap,
                host: "imap.example.com".into(),
                port: 993,
                encryption: EncryptionMode::SslTls,
                auth_method: AuthMethod::Plain,
                username: "user@example.com".into(),
                credential: "secret".into(),
                smtp: None,
                pop3_settings: None,
                color: None,
                avatar_path: None,
                category: None,
                sync_enabled: false,
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
                oauth_tenant: None,
                shared_mailbox: None,
            })
            .unwrap();

        let revoked = revoke_if_sync_disabled(&mut accounts);
        assert_eq!(revoked, Some(id));
        assert!(!accounts[0].is_primary());
    }

    #[test]
    fn revoke_if_sync_disabled_noop_when_primary_is_synced() {
        let mut accounts = vec![make_synced_account("A")];
        accounts[0].set_primary(true);
        let revoked = revoke_if_sync_disabled(&mut accounts);
        assert_eq!(revoked, None);
        assert!(accounts[0].is_primary());
    }

    #[test]
    fn revoke_if_sync_disabled_noop_when_no_primary() {
        let mut accounts = vec![make_unsynced_account("A")];
        let revoked = revoke_if_sync_disabled(&mut accounts);
        assert_eq!(revoked, None);
    }

    #[test]
    fn only_one_primary_at_a_time() {
        let mut accounts = vec![
            make_synced_account("A"),
            make_synced_account("B"),
            make_synced_account("C"),
        ];
        let id_a = accounts[0].id();
        let id_b = accounts[1].id();
        let id_c = accounts[2].id();
        set_primary(&mut accounts, id_a).unwrap();
        set_primary(&mut accounts, id_b).unwrap();
        set_primary(&mut accounts, id_c).unwrap();
        let primary_count = accounts.iter().filter(|a| a.is_primary()).count();
        assert_eq!(primary_count, 1);
        assert!(accounts[2].is_primary());
    }
}
