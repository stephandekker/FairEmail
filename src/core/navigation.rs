use crate::core::Account;

/// A group of accounts under a shared category header (FR-18, AC-7).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CategoryGroup {
    /// The category name. `None` represents the uncategorized group.
    pub category: Option<String>,
    /// Accounts in this group, already sorted by the within-group order.
    pub accounts: Vec<usize>,
}

/// Group accounts by category for the navigation pane (FR-18, AC-7).
///
/// - Categories are sorted alphabetically.
/// - Within each category, accounts are sorted by primary-first then alphabetically
///   (standing in for "custom order" until a drag-and-drop order field exists).
/// - Accounts without a category appear in an uncategorized group at the end.
///
/// Returns a list of `CategoryGroup`s where each group contains indices into the
/// original `accounts` slice.
pub fn group_by_category(accounts: &[Account]) -> Vec<CategoryGroup> {
    use std::collections::BTreeMap;

    let mut categorized: BTreeMap<String, Vec<usize>> = BTreeMap::new();
    let mut uncategorized: Vec<usize> = Vec::new();

    for (i, acct) in accounts.iter().enumerate() {
        match acct.category() {
            Some(cat) => {
                let trimmed = cat.trim();
                if trimmed.is_empty() {
                    uncategorized.push(i);
                } else {
                    categorized.entry(trimmed.to_string()).or_default().push(i);
                }
            }
            None => {
                uncategorized.push(i);
            }
        }
    }

    let mut groups: Vec<CategoryGroup> = Vec::new();

    // Categorized groups in alphabetical order.
    for (cat_name, mut indices) in categorized {
        sort_indices_by_account(accounts, &mut indices);
        groups.push(CategoryGroup {
            category: Some(cat_name),
            accounts: indices,
        });
    }

    // Uncategorized group at the end.
    if !uncategorized.is_empty() {
        sort_indices_by_account(accounts, &mut uncategorized);
        groups.push(CategoryGroup {
            category: None,
            accounts: uncategorized,
        });
    }

    groups
}

/// Sort accounts for flat (non-grouped) display (FR-19).
///
/// Order: primary first, then alphabetically by display name (case-insensitive).
/// This stands in for "custom order, then primary first, then alphabetically"
/// until a user-defined sort order field exists.
///
/// Returns indices into the original slice in sorted order.
pub fn sort_accounts_flat(accounts: &[Account]) -> Vec<usize> {
    let mut indices: Vec<usize> = (0..accounts.len()).collect();
    indices.sort_by(|&a, &b| {
        let acct_a = &accounts[a];
        let acct_b = &accounts[b];
        // Primary first.
        acct_b.is_primary().cmp(&acct_a.is_primary()).then_with(|| {
            acct_a
                .display_name()
                .to_lowercase()
                .cmp(&acct_b.display_name().to_lowercase())
        })
    });
    indices
}

/// Sort a set of indices by the within-group account order:
/// primary first, then alphabetically by display name.
fn sort_indices_by_account(accounts: &[Account], indices: &mut [usize]) {
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{AuthMethod, EncryptionMode, NewAccountParams, Protocol};

    fn make_account(name: &str, category: Option<&str>, primary: bool) -> Account {
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
            category: category.map(|s| s.to_string()),
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
        .unwrap();
        if primary {
            acct.set_primary(true);
        }
        acct
    }

    // -- group_by_category tests --

    #[test]
    fn empty_accounts_returns_no_groups() {
        let groups = group_by_category(&[]);
        assert!(groups.is_empty());
    }

    #[test]
    fn all_uncategorized_single_group() {
        let accounts = vec![
            make_account("Bravo", None, false),
            make_account("Alpha", None, false),
        ];
        let groups = group_by_category(&accounts);
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].category, None);
        // Sorted alphabetically.
        assert_eq!(groups[0].accounts, vec![1, 0]);
    }

    #[test]
    fn categories_sorted_alphabetically() {
        let accounts = vec![
            make_account("A1", Some("Work"), false),
            make_account("B1", Some("Personal"), false),
            make_account("C1", Some("Finance"), false),
        ];
        let groups = group_by_category(&accounts);
        assert_eq!(groups.len(), 3);
        assert_eq!(groups[0].category, Some("Finance".into()));
        assert_eq!(groups[1].category, Some("Personal".into()));
        assert_eq!(groups[2].category, Some("Work".into()));
    }

    #[test]
    fn accounts_within_category_sorted_primary_first_then_alpha() {
        let accounts = vec![
            make_account("Zeta", Some("Work"), false),
            make_account("Alpha", Some("Work"), false),
            make_account("Mid", Some("Work"), true), // primary
        ];
        let groups = group_by_category(&accounts);
        assert_eq!(groups.len(), 1);
        // Mid (primary) first, then Alpha, then Zeta.
        assert_eq!(groups[0].accounts, vec![2, 1, 0]);
    }

    #[test]
    fn uncategorized_group_at_end() {
        let accounts = vec![
            make_account("Uncategorized1", None, false),
            make_account("Categorized1", Some("Work"), false),
        ];
        let groups = group_by_category(&accounts);
        assert_eq!(groups.len(), 2);
        assert_eq!(groups[0].category, Some("Work".into()));
        assert_eq!(groups[0].accounts, vec![1]);
        assert_eq!(groups[1].category, None);
        assert_eq!(groups[1].accounts, vec![0]);
    }

    #[test]
    fn whitespace_only_category_treated_as_uncategorized() {
        let accounts = vec![
            make_account("A", Some("  "), false),
            make_account("B", Some("Work"), false),
        ];
        let groups = group_by_category(&accounts);
        assert_eq!(groups.len(), 2);
        assert_eq!(groups[0].category, Some("Work".into()));
        assert_eq!(groups[1].category, None);
        assert_eq!(groups[1].accounts, vec![0]);
    }

    #[test]
    fn mixed_categorized_and_uncategorized() {
        let accounts = vec![
            make_account("W1", Some("Work"), false),
            make_account("P1", Some("Personal"), false),
            make_account("U1", None, false),
            make_account("W2", Some("Work"), true),
            make_account("U2", None, true),
        ];
        let groups = group_by_category(&accounts);
        assert_eq!(groups.len(), 3);
        // Personal group.
        assert_eq!(groups[0].category, Some("Personal".into()));
        assert_eq!(groups[0].accounts, vec![1]);
        // Work group: W2 (primary) first, then W1.
        assert_eq!(groups[1].category, Some("Work".into()));
        assert_eq!(groups[1].accounts, vec![3, 0]);
        // Uncategorized: U2 (primary) first, then U1.
        assert_eq!(groups[2].category, None);
        assert_eq!(groups[2].accounts, vec![4, 2]);
    }

    // -- sort_accounts_flat tests --

    #[test]
    fn flat_sort_empty() {
        let indices = sort_accounts_flat(&[]);
        assert!(indices.is_empty());
    }

    #[test]
    fn flat_sort_primary_first() {
        let accounts = vec![
            make_account("Beta", None, false),
            make_account("Alpha", None, true), // primary
            make_account("Gamma", None, false),
        ];
        let indices = sort_accounts_flat(&accounts);
        // Alpha (primary) first, then Beta, then Gamma.
        assert_eq!(indices, vec![1, 0, 2]);
    }

    #[test]
    fn flat_sort_alphabetical_when_no_primary() {
        let accounts = vec![
            make_account("Charlie", None, false),
            make_account("Alpha", None, false),
            make_account("Bravo", None, false),
        ];
        let indices = sort_accounts_flat(&accounts);
        assert_eq!(indices, vec![1, 2, 0]);
    }

    #[test]
    fn flat_sort_case_insensitive() {
        let accounts = vec![
            make_account("bravo", None, false),
            make_account("Alpha", None, false),
        ];
        let indices = sort_accounts_flat(&accounts);
        assert_eq!(indices, vec![1, 0]);
    }

    #[test]
    fn flat_sort_ignores_category() {
        let accounts = vec![
            make_account("Zeta", Some("Work"), false),
            make_account("Alpha", Some("Personal"), false),
        ];
        let indices = sort_accounts_flat(&accounts);
        // Categories are ignored in flat sort.
        assert_eq!(indices, vec![1, 0]);
    }
}
