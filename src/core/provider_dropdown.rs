use super::provider::{Provider, ProviderDatabase, ProviderEncryption};

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
            documentation_url: None,
            localized_docs: vec![],
            oauth: None,
            display_order: order,
            enabled: true,
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
}
