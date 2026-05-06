use uuid::Uuid;

use super::Account;

/// Clear the primary designation if the deleted account was primary (FR-29, AC-9).
/// Returns true if the primary was cleared.
pub fn clear_primary_if_deleted(accounts: &mut [Account], deleted_id: Uuid) -> bool {
    // The deleted account is already removed from the list by the time we persist,
    // but in the in-memory list it may still be present. We clear primary on any
    // account with the matching ID.
    for acct in accounts.iter_mut() {
        if acct.id() == deleted_id && acct.is_primary() {
            acct.set_primary(false);
            return true;
        }
    }
    false
}

/// Remove an account from the custom order list, if present.
pub fn remove_from_order(order: &mut Vec<Uuid>, deleted_id: Uuid) {
    order.retain(|id| *id != deleted_id);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{AuthMethod, EncryptionMode, NewAccountParams, Protocol};

    fn make_account(name: &str, primary: bool) -> Account {
        let mut acct = Account::new(NewAccountParams {
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
        })
        .unwrap();
        if primary {
            acct.set_primary(true);
        }
        acct
    }

    #[test]
    fn clear_primary_when_deleted_account_is_primary() {
        let mut accounts = vec![make_account("A", true), make_account("B", false)];
        let id = accounts[0].id();
        let cleared = clear_primary_if_deleted(&mut accounts, id);
        assert!(cleared);
        assert!(!accounts[0].is_primary());
    }

    #[test]
    fn no_clear_when_deleted_account_is_not_primary() {
        let mut accounts = vec![make_account("A", true), make_account("B", false)];
        let id = accounts[1].id();
        let cleared = clear_primary_if_deleted(&mut accounts, id);
        assert!(!cleared);
        assert!(accounts[0].is_primary());
    }

    #[test]
    fn no_clear_when_id_not_found() {
        let mut accounts = vec![make_account("A", true)];
        let cleared = clear_primary_if_deleted(&mut accounts, Uuid::new_v4());
        assert!(!cleared);
        assert!(accounts[0].is_primary());
    }

    #[test]
    fn remove_from_order_removes_matching_id() {
        let id1 = Uuid::new_v4();
        let id2 = Uuid::new_v4();
        let id3 = Uuid::new_v4();
        let mut order = vec![id1, id2, id3];
        remove_from_order(&mut order, id2);
        assert_eq!(order, vec![id1, id3]);
    }

    #[test]
    fn remove_from_order_noop_when_id_not_present() {
        let id1 = Uuid::new_v4();
        let mut order = vec![id1];
        remove_from_order(&mut order, Uuid::new_v4());
        assert_eq!(order, vec![id1]);
    }
}
