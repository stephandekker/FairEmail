use super::provider::Provider;

/// The bundled provider catalogue, embedded at compile time from the JSON data file.
///
/// To add, modify, or remove a provider, edit `data/providers.json` — no Rust
/// code changes are needed for routine catalogue maintenance (FR-43).
///
/// The catalogue is versioned with the application binary; there is no
/// over-the-air update mechanism (FR-42).
const CATALOGUE_JSON: &str = include_str!("../../data/providers.json");

/// Returns all bundled provider entries by deserializing the embedded JSON catalogue.
pub(crate) fn bundled_providers() -> Vec<Provider> {
    serde_json::from_str(CATALOGUE_JSON).expect("bundled provider catalogue is valid JSON")
}

#[cfg(test)]
mod tests {
    use super::super::provider::ProviderDatabase;
    use super::*;

    /// The embedded catalogue parses successfully and contains providers.
    #[test]
    fn bundled_catalogue_loads() {
        let providers = bundled_providers();
        assert!(
            !providers.is_empty(),
            "catalogue must contain at least one provider"
        );
    }

    /// AC-1: A new provider can be added by appending to the JSON array — no
    /// source code changes required. We simulate this by appending a minimal
    /// provider entry to the catalogue JSON and verifying it parses and is
    /// findable via the database.
    #[test]
    fn adding_provider_requires_only_data_change() {
        let mut providers: Vec<Provider> = serde_json::from_str(CATALOGUE_JSON).unwrap();
        let original_count = providers.len();

        // Append a new provider via JSON (simulates editing the data file).
        let new_provider_json = r#"{
            "id": "extensibility_test_add",
            "display_name": "Extensibility Test Provider",
            "domain_patterns": ["extensibility-test.example.com"],
            "mx_patterns": [],
            "incoming": { "hostname": "imap.extensibility-test.example.com", "port": 993, "encryption": "SslTls" },
            "outgoing": { "hostname": "smtp.extensibility-test.example.com", "port": 465, "encryption": "SslTls" },
            "username_type": "EmailAddress",
            "keep_alive_interval": 15,
            "noop_keep_alive": false,
            "partial_fetch": true,
            "max_tls_version": "Tls1_3",
            "app_password_required": false,
            "documentation_url": null,
            "localized_docs": [],
            "oauth": null,
            "display_order": 10000,
            "enabled": true
        }"#;
        let new_provider: Provider = serde_json::from_str(new_provider_json).unwrap();
        providers.push(new_provider);

        assert_eq!(providers.len(), original_count + 1);

        let db = ProviderDatabase::new(providers);
        let candidates = db.lookup_by_domain("extensibility-test.example.com");
        let candidate = candidates.expect("new provider must be found");
        assert_eq!(candidate.provider.id, "extensibility_test_add");
    }

    /// AC-2: An existing provider's settings can be modified by editing the JSON.
    #[test]
    fn modifying_provider_requires_only_data_change() {
        let mut providers: Vec<Provider> = serde_json::from_str(CATALOGUE_JSON).unwrap();

        // Find Gmail and change its SMTP port (simulates editing the data file).
        let gmail = providers.iter_mut().find(|p| p.id == "gmail").unwrap();
        assert_eq!(gmail.outgoing.port, 465); // original
        gmail.outgoing.port = 587;

        // Re-serialize and re-parse to prove the round-trip works.
        let json = serde_json::to_string(&providers).unwrap();
        let reloaded: Vec<Provider> = serde_json::from_str(&json).unwrap();
        let gmail = reloaded.iter().find(|p| p.id == "gmail").unwrap();
        assert_eq!(gmail.outgoing.port, 587);
    }

    /// AC-3: A provider can be removed by deleting it from the JSON.
    #[test]
    fn removing_provider_requires_only_data_change() {
        let mut providers: Vec<Provider> = serde_json::from_str(CATALOGUE_JSON).unwrap();
        let original_count = providers.len();

        // Remove debug_test_provider (simulates editing the data file).
        providers.retain(|p| p.id != "debug_test_provider");
        assert_eq!(providers.len(), original_count - 1);

        let db = ProviderDatabase::new(providers);
        let candidates = db.lookup_by_domain("debug-test.example.com");
        assert!(candidates.is_none(), "removed provider must not be found");
    }

    /// AC-4: Adding a new optional field to the data model does not break
    /// parsing of existing entries that lack the field. We verify this by
    /// deserializing a minimal provider entry (without optional fields) and
    /// confirming defaults are applied.
    #[test]
    fn new_optional_fields_do_not_break_existing_entries() {
        // A minimal provider entry missing all `#[serde(default)]` optional fields.
        let minimal_json = r#"{
            "id": "minimal_test",
            "display_name": "Minimal Provider",
            "domain_patterns": ["minimal.example.com"],
            "mx_patterns": [],
            "incoming": { "hostname": "imap.minimal.example.com", "port": 993, "encryption": "SslTls" },
            "outgoing": { "hostname": "smtp.minimal.example.com", "port": 465, "encryption": "SslTls" },
            "username_type": "EmailAddress",
            "keep_alive_interval": 15,
            "noop_keep_alive": false,
            "partial_fetch": true,
            "max_tls_version": "Tls1_3",
            "app_password_required": false,
            "documentation_url": null,
            "localized_docs": [],
            "oauth": null,
            "display_order": 1,
            "enabled": true
        }"#;

        // This must parse without error — optional fields default gracefully.
        let provider: Provider = serde_json::from_str(minimal_json).unwrap();
        assert_eq!(provider.id, "minimal_test");
        // Fields with #[serde(default)] should have their defaults.
        assert!(!provider.debug_only);
        assert!(!provider.disable_ip_connections);
        assert!(!provider.requires_manual_enablement);
        assert!(!provider.supports_shared_mailbox);
        assert!(provider.subtitle.is_none());
        assert!(provider.registration_url.is_none());
        assert!(provider.app_password_url.is_none());
        assert!(provider.graph.is_none());
        assert!(provider.variant_of.is_none());
    }

    /// AC-5: The catalogue is versioned and released with the application.
    /// The JSON is embedded via `include_str!` at compile time, so there is
    /// no separate update mechanism — this test simply confirms the catalogue
    /// is a compile-time constant string that deserializes correctly.
    #[test]
    fn catalogue_is_compile_time_embedded() {
        // CATALOGUE_JSON is a &'static str from include_str! — if it were
        // loaded at runtime this const would not exist.
        assert!(
            !CATALOGUE_JSON.is_empty(),
            "catalogue must be embedded at compile time"
        );
        let providers: Vec<Provider> = serde_json::from_str(CATALOGUE_JSON).unwrap();
        assert!(!providers.is_empty());
    }
}
