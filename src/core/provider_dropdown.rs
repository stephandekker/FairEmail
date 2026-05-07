use super::provider::{Provider, ProviderDatabase, ProviderEncryption, ServerConfig, UsernameType};

/// A single entry in the provider dropdown.
#[derive(Debug, Clone)]
pub struct ProviderDropdownEntry {
    /// Provider ID, or empty string for "Custom".
    pub id: String,
    /// Display label shown in the dropdown.
    pub label: String,
}

/// Pre-fill values returned when a provider is selected.
#[derive(Debug, Clone)]
pub struct ProviderPrefill {
    pub hostname: String,
    pub port: u16,
    pub encryption: ProviderEncryption,
}

/// Build the list of dropdown entries from the provider database.
/// The first entry is always "Custom", followed by enabled providers
/// sorted by display_order then display_name.
pub fn build_dropdown_entries(db: &ProviderDatabase) -> Vec<ProviderDropdownEntry> {
    let mut entries = vec![ProviderDropdownEntry {
        id: String::new(),
        label: "Custom".to_string(),
    }];

    let mut providers: Vec<&Provider> = db.providers().iter().filter(|p| p.enabled).collect();
    providers.sort_by(|a, b| {
        a.display_order
            .cmp(&b.display_order)
            .then_with(|| a.display_name.cmp(&b.display_name))
    });

    // Deduplicate by provider ID.
    let mut seen_ids = std::collections::HashSet::new();
    for p in providers {
        if seen_ids.insert(&p.id) {
            entries.push(ProviderDropdownEntry {
                id: p.id.clone(),
                label: p.display_name.clone(),
            });
        }
    }

    entries
}

/// Get the inbound pre-fill data for a provider by ID.
/// Returns `None` for the "Custom" entry (empty ID) or if the provider is not found.
pub fn prefill_for_provider(db: &ProviderDatabase, provider_id: &str) -> Option<ProviderPrefill> {
    if provider_id.is_empty() {
        return None;
    }
    db.providers()
        .iter()
        .find(|p| p.id == provider_id && p.enabled)
        .map(|p| ProviderPrefill {
            hostname: p.incoming.hostname.clone(),
            port: p.incoming.port,
            encryption: p.incoming.encryption,
        })
}

/// Get provider-specific guidance text (e.g. "Use an app-specific password").
/// Returns `None` if no special guidance applies.
pub fn provider_guidance(db: &ProviderDatabase, provider_id: &str) -> Option<String> {
    if provider_id.is_empty() {
        return None;
    }
    let provider = db
        .providers()
        .iter()
        .find(|p| p.id == provider_id && p.enabled)?;

    let mut parts: Vec<String> = Vec::new();

    if provider.app_password_required {
        parts.push("This provider requires an app-specific password.".to_string());
    }

    // Include localized doc snippets (use first available).
    if let Some(doc) = provider.localized_docs.first() {
        parts.push(doc.text.clone());
    }

    if let Some(ref url) = provider.documentation_url {
        parts.push(format!("Setup guide: {url}"));
    }

    if parts.is_empty() {
        None
    } else {
        Some(parts.join(" "))
    }
}

/// Find the dropdown index for a given provider ID.
/// Returns 0 (Custom) if the provider is not found.
pub fn index_for_provider_id(entries: &[ProviderDropdownEntry], provider_id: &str) -> u32 {
    if provider_id.is_empty() {
        return 0;
    }
    entries
        .iter()
        .position(|e| e.id == provider_id)
        .unwrap_or(0) as u32
}

/// A single entry in the browsable provider list (FR-10, FR-13, FR-14).
#[derive(Debug, Clone)]
pub struct ProviderListEntry {
    /// Provider ID. Empty string for the "Other provider" fallback.
    pub id: String,
    /// Display name.
    pub label: String,
    /// Optional subtitle (e.g. "Corporate" for Office365).
    pub subtitle: Option<String>,
    /// If `Some`, this entry is a variant of the named primary provider.
    pub variant_of: Option<String>,
    /// Whether this is the "Other provider" fallback entry.
    pub is_other: bool,
}

/// Full pre-fill data returned when a provider is selected from the list (AC-6).
/// Includes both server configs, username format, and behavioural overrides.
#[derive(Debug, Clone)]
pub struct FullProviderPrefill {
    pub incoming: ServerConfig,
    pub outgoing: ServerConfig,
    pub username_type: UsernameType,
    pub keep_alive_interval: u32,
    pub noop_keep_alive: bool,
    pub partial_fetch: bool,
    pub app_password_required: bool,
}

/// Build the browsable provider list for the account setup flow (FR-10, FR-11, FR-12, FR-13, FR-14).
///
/// - Excludes disabled providers (FR-11).
/// - Excludes debug-only providers unless `debug_mode` is true (FR-12).
/// - Sorts by `display_order` (lower = higher priority), then locale-aware alphabetical (FR-10, NFR-4).
/// - Groups variants with their primary provider (FR-13).
/// - Appends an "Other provider" fallback entry at the end (FR-14).
pub fn build_provider_list(db: &ProviderDatabase, debug_mode: bool) -> Vec<ProviderListEntry> {
    let mut providers: Vec<&Provider> = db
        .providers()
        .iter()
        .filter(|p| p.enabled)
        .filter(|p| !p.debug_only || debug_mode)
        .filter(|p| p.variant_of.is_none()) // Primary providers only; variants inserted below.
        .collect();

    providers.sort_by(|a, b| {
        a.display_order
            .cmp(&b.display_order)
            .then_with(|| locale_aware_cmp(&a.display_name, &b.display_name))
    });

    // Deduplicate by provider ID.
    let mut seen_ids = std::collections::HashSet::new();
    let mut entries = Vec::new();

    for p in providers {
        if !seen_ids.insert(&p.id) {
            continue;
        }
        entries.push(ProviderListEntry {
            id: p.id.clone(),
            label: p.display_name.clone(),
            subtitle: p.subtitle.clone(),
            variant_of: None,
            is_other: false,
        });

        // Insert any variants of this provider immediately after (FR-13).
        let mut variants: Vec<&Provider> = db
            .providers()
            .iter()
            .filter(|v| v.enabled && (!v.debug_only || debug_mode))
            .filter(|v| v.variant_of.as_deref() == Some(&p.id))
            .collect();
        variants.sort_by(|a, b| {
            a.display_order
                .cmp(&b.display_order)
                .then_with(|| locale_aware_cmp(&a.display_name, &b.display_name))
        });
        for v in variants {
            if seen_ids.insert(&v.id) {
                entries.push(ProviderListEntry {
                    id: v.id.clone(),
                    label: v.display_name.clone(),
                    subtitle: v.subtitle.clone(),
                    variant_of: v.variant_of.clone(),
                    is_other: false,
                });
            }
        }
    }

    // Append "Other provider" fallback (FR-14).
    entries.push(ProviderListEntry {
        id: String::new(),
        label: "Other provider".to_string(),
        subtitle: None,
        variant_of: None,
        is_other: true,
    });

    entries
}

/// Get the full pre-fill data for a provider by ID (AC-6).
/// Returns `None` for the "Other provider" entry (empty ID) or unknown providers.
pub fn full_prefill_for_provider(
    db: &ProviderDatabase,
    provider_id: &str,
) -> Option<FullProviderPrefill> {
    if provider_id.is_empty() {
        return None;
    }
    db.providers()
        .iter()
        .find(|p| p.id == provider_id && p.enabled)
        .map(|p| FullProviderPrefill {
            incoming: p.incoming.clone(),
            outgoing: p.outgoing.clone(),
            username_type: p.username_type.clone(),
            keep_alive_interval: p.keep_alive_interval,
            noop_keep_alive: p.noop_keep_alive,
            partial_fetch: p.partial_fetch,
            app_password_required: p.app_password_required,
        })
}

/// Locale-aware case-insensitive string comparison.
///
/// Uses `str::to_lowercase()` for Unicode-aware case folding.
/// For true ICU-level collation a crate like `icu_collator` would be needed,
/// but lowercased comparison satisfies the story's "locale-aware" requirement
/// for Latin-script provider names.
fn locale_aware_cmp(a: &str, b: &str) -> std::cmp::Ordering {
    let a_lower = a.to_lowercase();
    let b_lower = b.to_lowercase();
    a_lower.cmp(&b_lower)
}

/// Check whether the application is running in debug/development mode (FR-12).
///
/// Returns `true` when the binary was compiled with `debug_assertions`
/// (i.e. a `cargo build` / `cargo run` without `--release`).
pub fn is_debug_mode() -> bool {
    cfg!(debug_assertions)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::provider::{
        LocalizedDoc, MaxTlsVersion, Provider, ProviderDatabase, ProviderEncryption, ServerConfig,
        UsernameType,
    };

    fn make_provider(id: &str, name: &str, order: u32) -> Provider {
        Provider {
            id: id.to_string(),
            display_name: name.to_string(),
            domain_patterns: vec![format!("{id}.com")],
            mx_patterns: vec![],
            incoming: ServerConfig {
                hostname: format!("imap.{id}.com"),
                port: 993,
                encryption: ProviderEncryption::SslTls,
            },
            outgoing: ServerConfig {
                hostname: format!("smtp.{id}.com"),
                port: 465,
                encryption: ProviderEncryption::SslTls,
            },
            username_type: UsernameType::EmailAddress,
            keep_alive_interval: 15,
            noop_keep_alive: false,
            partial_fetch: true,
            max_tls_version: MaxTlsVersion::Tls1_3,
            app_password_required: false,
            disable_ip_connections: false,
            requires_manual_enablement: false,
            documentation_url: None,
            localized_docs: vec![],
            oauth: None,
            display_order: order,
            enabled: true,
            supports_shared_mailbox: false,
            subtitle: None,
            registration_url: None,
            graph: None,
            debug_only: false,
            variant_of: None,
        }
    }

    #[test]
    fn test_build_dropdown_entries_starts_with_custom() {
        let db = ProviderDatabase::new(vec![make_provider("gmail", "Gmail", 1)]);
        let entries = build_dropdown_entries(&db);
        assert_eq!(entries[0].label, "Custom");
        assert!(entries[0].id.is_empty());
    }

    #[test]
    fn test_build_dropdown_entries_sorted_by_order() {
        let db = ProviderDatabase::new(vec![
            make_provider("yahoo", "Yahoo", 2),
            make_provider("gmail", "Gmail", 1),
            make_provider("outlook", "Outlook", 3),
        ]);
        let entries = build_dropdown_entries(&db);
        assert_eq!(entries.len(), 4); // Custom + 3
        assert_eq!(entries[1].label, "Gmail");
        assert_eq!(entries[2].label, "Yahoo");
        assert_eq!(entries[3].label, "Outlook");
    }

    #[test]
    fn test_build_dropdown_entries_excludes_disabled() {
        let mut disabled = make_provider("disabled", "Disabled", 1);
        disabled.enabled = false;
        let db = ProviderDatabase::new(vec![make_provider("gmail", "Gmail", 1), disabled]);
        let entries = build_dropdown_entries(&db);
        assert_eq!(entries.len(), 2); // Custom + gmail
    }

    #[test]
    fn test_prefill_for_custom_returns_none() {
        let db = ProviderDatabase::new(vec![make_provider("gmail", "Gmail", 1)]);
        assert!(prefill_for_provider(&db, "").is_none());
    }

    #[test]
    fn test_prefill_for_known_provider() {
        let db = ProviderDatabase::new(vec![make_provider("gmail", "Gmail", 1)]);
        let prefill = prefill_for_provider(&db, "gmail").unwrap();
        assert_eq!(prefill.hostname, "imap.gmail.com");
        assert_eq!(prefill.port, 993);
        assert_eq!(prefill.encryption, ProviderEncryption::SslTls);
    }

    #[test]
    fn test_prefill_for_unknown_returns_none() {
        let db = ProviderDatabase::new(vec![make_provider("gmail", "Gmail", 1)]);
        assert!(prefill_for_provider(&db, "nonexistent").is_none());
    }

    #[test]
    fn test_guidance_for_custom_returns_none() {
        let db = ProviderDatabase::new(vec![make_provider("gmail", "Gmail", 1)]);
        assert!(provider_guidance(&db, "").is_none());
    }

    #[test]
    fn test_guidance_with_app_password() {
        let mut p = make_provider("gmail", "Gmail", 1);
        p.app_password_required = true;
        let db = ProviderDatabase::new(vec![p]);
        let guidance = provider_guidance(&db, "gmail").unwrap();
        assert!(guidance.contains("app-specific password"));
    }

    #[test]
    fn test_guidance_with_doc_url() {
        let mut p = make_provider("gmail", "Gmail", 1);
        p.documentation_url = Some("https://example.com/setup".to_string());
        let db = ProviderDatabase::new(vec![p]);
        let guidance = provider_guidance(&db, "gmail").unwrap();
        assert!(guidance.contains("https://example.com/setup"));
    }

    #[test]
    fn test_guidance_with_localized_doc() {
        let mut p = make_provider("gmail", "Gmail", 1);
        p.localized_docs = vec![LocalizedDoc {
            locale: "en".to_string(),
            text: "Enable IMAP in settings first.".to_string(),
        }];
        let db = ProviderDatabase::new(vec![p]);
        let guidance = provider_guidance(&db, "gmail").unwrap();
        assert!(guidance.contains("Enable IMAP in settings first."));
    }

    #[test]
    fn test_guidance_none_when_no_special_info() {
        let db = ProviderDatabase::new(vec![make_provider("gmail", "Gmail", 1)]);
        assert!(provider_guidance(&db, "gmail").is_none());
    }

    #[test]
    fn test_index_for_provider_id() {
        let db = ProviderDatabase::new(vec![
            make_provider("gmail", "Gmail", 1),
            make_provider("yahoo", "Yahoo", 2),
        ]);
        let entries = build_dropdown_entries(&db);
        assert_eq!(index_for_provider_id(&entries, ""), 0);
        assert_eq!(index_for_provider_id(&entries, "gmail"), 1);
        assert_eq!(index_for_provider_id(&entries, "yahoo"), 2);
        assert_eq!(index_for_provider_id(&entries, "unknown"), 0);
    }

    #[test]
    fn test_bundled_dropdown_has_many_entries() {
        let db = ProviderDatabase::bundled();
        let entries = build_dropdown_entries(&db);
        // Custom + many providers
        assert!(entries.len() > 50);
    }

    // --- Provider list tests (story 4) ---

    #[test]
    fn test_provider_list_sorted_by_order_then_name() {
        let db = ProviderDatabase::new(vec![
            make_provider("yahoo", "Yahoo", 2),
            make_provider("gmail", "Gmail", 1),
            make_provider("beta", "Beta Mail", 2),
            make_provider("outlook", "Outlook", 3),
        ]);
        let entries = build_provider_list(&db, false);
        // Gmail (1), Beta Mail (2), Yahoo (2), Outlook (3), Other provider
        assert_eq!(entries.len(), 5);
        assert_eq!(entries[0].label, "Gmail");
        assert_eq!(entries[1].label, "Beta Mail");
        assert_eq!(entries[2].label, "Yahoo");
        assert_eq!(entries[3].label, "Outlook");
        assert!(entries[4].is_other);
        assert_eq!(entries[4].label, "Other provider");
    }

    #[test]
    fn test_provider_list_excludes_disabled() {
        let mut disabled = make_provider("disabled", "Disabled", 1);
        disabled.enabled = false;
        let db = ProviderDatabase::new(vec![make_provider("gmail", "Gmail", 1), disabled]);
        let entries = build_provider_list(&db, false);
        assert_eq!(entries.len(), 2); // Gmail + Other
        assert_eq!(entries[0].label, "Gmail");
        assert!(entries[1].is_other);
    }

    #[test]
    fn test_provider_list_excludes_debug_only_in_normal_mode() {
        let mut debug_prov = make_provider("debug", "Debug Provider", 1);
        debug_prov.debug_only = true;
        let db = ProviderDatabase::new(vec![make_provider("gmail", "Gmail", 1), debug_prov]);
        let entries = build_provider_list(&db, false);
        assert_eq!(entries.len(), 2); // Gmail + Other
        assert!(!entries.iter().any(|e| e.id == "debug"));
    }

    #[test]
    fn test_provider_list_includes_debug_only_in_debug_mode() {
        let mut debug_prov = make_provider("debug", "Debug Provider", 1);
        debug_prov.debug_only = true;
        let db = ProviderDatabase::new(vec![make_provider("gmail", "Gmail", 2), debug_prov]);
        let entries = build_provider_list(&db, true);
        assert_eq!(entries.len(), 3); // Debug + Gmail + Other
        assert!(entries.iter().any(|e| e.id == "debug"));
    }

    #[test]
    fn test_provider_list_groups_variants() {
        let primary = make_provider("outlook", "Outlook", 2);
        let mut variant = make_provider("office365", "Microsoft 365", 3);
        variant.variant_of = Some("outlook".to_string());
        let db = ProviderDatabase::new(vec![
            make_provider("gmail", "Gmail", 1),
            primary,
            variant,
            make_provider("yahoo", "Yahoo", 4),
        ]);
        let entries = build_provider_list(&db, false);
        // Gmail (1), Outlook (2), Microsoft 365 (variant of Outlook), Yahoo (4), Other
        assert_eq!(entries.len(), 5);
        assert_eq!(entries[0].label, "Gmail");
        assert_eq!(entries[1].label, "Outlook");
        assert_eq!(entries[2].label, "Microsoft 365");
        assert_eq!(entries[2].variant_of.as_deref(), Some("outlook"));
        assert_eq!(entries[3].label, "Yahoo");
        assert!(entries[4].is_other);
    }

    #[test]
    fn test_provider_list_has_other_at_end() {
        let db = ProviderDatabase::new(vec![make_provider("gmail", "Gmail", 1)]);
        let entries = build_provider_list(&db, false);
        let last = entries.last().unwrap();
        assert!(last.is_other);
        assert!(last.id.is_empty());
        assert_eq!(last.label, "Other provider");
    }

    #[test]
    fn test_provider_list_other_only_when_empty_db() {
        let db = ProviderDatabase::new(vec![]);
        let entries = build_provider_list(&db, false);
        assert_eq!(entries.len(), 1);
        assert!(entries[0].is_other);
    }

    #[test]
    fn test_full_prefill_returns_all_settings() {
        let mut p = make_provider("gmail", "Gmail", 1);
        p.keep_alive_interval = 30;
        p.noop_keep_alive = true;
        p.partial_fetch = false;
        p.app_password_required = true;
        let db = ProviderDatabase::new(vec![p]);
        let prefill = full_prefill_for_provider(&db, "gmail").unwrap();
        assert_eq!(prefill.incoming.hostname, "imap.gmail.com");
        assert_eq!(prefill.incoming.port, 993);
        assert_eq!(prefill.outgoing.hostname, "smtp.gmail.com");
        assert_eq!(prefill.outgoing.port, 465);
        assert_eq!(prefill.username_type, UsernameType::EmailAddress);
        assert_eq!(prefill.keep_alive_interval, 30);
        assert!(prefill.noop_keep_alive);
        assert!(!prefill.partial_fetch);
        assert!(prefill.app_password_required);
    }

    #[test]
    fn test_full_prefill_other_returns_none() {
        let db = ProviderDatabase::new(vec![make_provider("gmail", "Gmail", 1)]);
        assert!(full_prefill_for_provider(&db, "").is_none());
    }

    #[test]
    fn test_bundled_list_popular_first() {
        let db = ProviderDatabase::bundled();
        let entries = build_provider_list(&db, false);
        // Gmail, Outlook, Yahoo should be among the first entries
        let gmail_pos = entries.iter().position(|e| e.id == "gmail").unwrap();
        let outlook_pos = entries.iter().position(|e| e.id == "outlook").unwrap();
        let yahoo_pos = entries.iter().position(|e| e.id == "yahoo").unwrap();
        assert!(gmail_pos < 5, "Gmail should be near the top");
        assert!(outlook_pos < 5, "Outlook should be near the top");
        assert!(yahoo_pos < 10, "Yahoo should be near the top");
    }

    #[test]
    fn test_bundled_list_office365_after_outlook() {
        let db = ProviderDatabase::bundled();
        let entries = build_provider_list(&db, false);
        let outlook_pos = entries.iter().position(|e| e.id == "outlook").unwrap();
        let office365_pos = entries.iter().position(|e| e.id == "office365").unwrap();
        assert_eq!(
            office365_pos,
            outlook_pos + 1,
            "Office365 should be grouped right after Outlook"
        );
        assert_eq!(
            entries[office365_pos].variant_of.as_deref(),
            Some("outlook")
        );
    }

    #[test]
    fn test_bundled_list_debug_provider_hidden_in_normal_mode() {
        let db = ProviderDatabase::bundled();
        let entries = build_provider_list(&db, false);
        assert!(!entries.iter().any(|e| e.id == "debug_test_provider"));
    }

    #[test]
    fn test_bundled_list_debug_provider_visible_in_debug_mode() {
        let db = ProviderDatabase::bundled();
        let entries = build_provider_list(&db, true);
        assert!(entries.iter().any(|e| e.id == "debug_test_provider"));
    }

    #[test]
    fn test_locale_aware_sort_case_insensitive() {
        let db = ProviderDatabase::new(vec![
            make_provider("charlie", "charlie mail", 1),
            make_provider("alpha", "Alpha Mail", 1),
            make_provider("bravo", "BRAVO Mail", 1),
        ]);
        let entries = build_provider_list(&db, false);
        assert_eq!(entries[0].label, "Alpha Mail");
        assert_eq!(entries[1].label, "BRAVO Mail");
        assert_eq!(entries[2].label, "charlie mail");
    }
}
