use uuid::Uuid;

use crate::core::Account;

/// Apply a custom order to accounts (FR-20, US-19).
///
/// Given a list of account UUIDs representing the desired order, returns
/// indices into the `accounts` slice in that order. Any accounts not present
/// in `order` are appended at the end using the default sort (primary first,
/// then alphabetical).
pub fn apply_custom_order(accounts: &[Account], order: &[Uuid]) -> Vec<usize> {
    let mut result: Vec<usize> = Vec::with_capacity(accounts.len());
    let mut used = vec![false; accounts.len()];

    // Add accounts in custom order.
    for id in order {
        if let Some(idx) = accounts.iter().position(|a| a.id() == *id) {
            if !used[idx] {
                result.push(idx);
                used[idx] = true;
            }
        }
    }

    // Append any remaining accounts using default sort (primary first, then alpha).
    let mut remaining: Vec<usize> = (0..accounts.len()).filter(|&i| !used[i]).collect();
    remaining.sort_by(|&a, &b| {
        let acct_a = &accounts[a];
        let acct_b = &accounts[b];
        acct_b.is_primary().cmp(&acct_a.is_primary()).then_with(|| {
            acct_a
                .display_name()
                .to_lowercase()
                .cmp(&acct_b.display_name().to_lowercase())
        })
    });
    result.extend(remaining);

    result
}

/// Move an account from one position to another within the order (FR-20, US-19).
///
/// Returns the updated order. If `from` or `to` are out of bounds, returns
/// the order unchanged.
pub fn move_account(order: &[Uuid], from: usize, to: usize) -> Vec<Uuid> {
    let mut new_order = order.to_vec();
    if from >= new_order.len() || to >= new_order.len() || from == to {
        return new_order;
    }
    let item = new_order.remove(from);
    new_order.insert(to, item);
    new_order
}

/// Compute the default order: primary first, then alphabetical by display name (FR-21, US-20).
///
/// Returns a `Vec<Uuid>` representing the default order.
pub fn default_order(accounts: &[Account]) -> Vec<Uuid> {
    let mut indices: Vec<usize> = (0..accounts.len()).collect();
    indices.sort_by(|&a, &b| {
        let acct_a = &accounts[a];
        let acct_b = &accounts[b];
        acct_b.is_primary().cmp(&acct_a.is_primary()).then_with(|| {
            acct_a
                .display_name()
                .to_lowercase()
                .cmp(&acct_b.display_name().to_lowercase())
        })
    });
    indices.iter().map(|&i| accounts[i].id()).collect()
}

/// Build order from current accounts list (for initial order when none persisted).
/// Preserves the current order of accounts as given.
pub fn order_from_accounts(accounts: &[Account]) -> Vec<Uuid> {
    accounts.iter().map(|a| a.id()).collect()
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
        })
        .unwrap();
        if primary {
            acct.set_primary(true);
        }
        acct
    }

    #[test]
    fn apply_custom_order_full_match() {
        let accounts = vec![
            make_account("Alpha", false),
            make_account("Beta", false),
            make_account("Gamma", false),
        ];
        let order = vec![accounts[2].id(), accounts[0].id(), accounts[1].id()];
        let result = apply_custom_order(&accounts, &order);
        assert_eq!(result, vec![2, 0, 1]);
    }

    #[test]
    fn apply_custom_order_partial_match_appends_remaining() {
        let accounts = vec![
            make_account("Alpha", false),
            make_account("Beta", false),
            make_account("Gamma", true),
        ];
        // Only Beta in custom order; Alpha and Gamma should be appended (Gamma primary first).
        let order = vec![accounts[1].id()];
        let result = apply_custom_order(&accounts, &order);
        assert_eq!(result, vec![1, 2, 0]);
    }

    #[test]
    fn apply_custom_order_empty_order_uses_default() {
        let accounts = vec![make_account("Beta", false), make_account("Alpha", true)];
        let result = apply_custom_order(&accounts, &[]);
        // Primary first (Alpha idx=1), then Beta (idx=0).
        assert_eq!(result, vec![1, 0]);
    }

    #[test]
    fn apply_custom_order_stale_ids_ignored() {
        let accounts = vec![make_account("Alpha", false), make_account("Beta", false)];
        let stale_id = Uuid::new_v4();
        let order = vec![stale_id, accounts[1].id(), accounts[0].id()];
        let result = apply_custom_order(&accounts, &order);
        assert_eq!(result, vec![1, 0]);
    }

    #[test]
    fn move_account_forward() {
        let ids: Vec<Uuid> = (0..4).map(|_| Uuid::new_v4()).collect();
        let result = move_account(&ids, 0, 2);
        assert_eq!(result, vec![ids[1], ids[2], ids[0], ids[3]]);
    }

    #[test]
    fn move_account_backward() {
        let ids: Vec<Uuid> = (0..4).map(|_| Uuid::new_v4()).collect();
        let result = move_account(&ids, 3, 1);
        assert_eq!(result, vec![ids[0], ids[3], ids[1], ids[2]]);
    }

    #[test]
    fn move_account_same_position_no_change() {
        let ids: Vec<Uuid> = (0..3).map(|_| Uuid::new_v4()).collect();
        let result = move_account(&ids, 1, 1);
        assert_eq!(result, ids);
    }

    #[test]
    fn move_account_out_of_bounds_no_change() {
        let ids: Vec<Uuid> = (0..3).map(|_| Uuid::new_v4()).collect();
        let result = move_account(&ids, 5, 1);
        assert_eq!(result, ids);
    }

    #[test]
    fn default_order_primary_first_then_alpha() {
        let accounts = vec![
            make_account("Charlie", false),
            make_account("Alpha", true),
            make_account("Beta", false),
        ];
        let order = default_order(&accounts);
        assert_eq!(
            order,
            vec![accounts[1].id(), accounts[2].id(), accounts[0].id()]
        );
    }

    #[test]
    fn default_order_all_non_primary_alphabetical() {
        let accounts = vec![
            make_account("Zeta", false),
            make_account("Alpha", false),
            make_account("Mid", false),
        ];
        let order = default_order(&accounts);
        assert_eq!(
            order,
            vec![accounts[1].id(), accounts[2].id(), accounts[0].id()]
        );
    }

    #[test]
    fn order_from_accounts_preserves_current_order() {
        let accounts = vec![
            make_account("B", false),
            make_account("A", false),
            make_account("C", false),
        ];
        let order = order_from_accounts(&accounts);
        assert_eq!(
            order,
            vec![accounts[0].id(), accounts[1].id(), accounts[2].id()]
        );
    }

    #[test]
    fn apply_custom_order_no_duplicates() {
        let accounts = vec![make_account("Alpha", false), make_account("Beta", false)];
        // Duplicate ID in order should not duplicate in result.
        let order = vec![accounts[0].id(), accounts[0].id(), accounts[1].id()];
        let result = apply_custom_order(&accounts, &order);
        assert_eq!(result, vec![0, 1]);
    }
}
