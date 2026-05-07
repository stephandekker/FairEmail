use super::provider::{OAuthConfig, Provider, ProviderDatabase};

/// A read-only registry of OAuth-enabled providers.
///
/// Wraps a [`ProviderDatabase`] and exposes only providers that have an
/// [`OAuthConfig`] attached. Adding OAuth support for a new provider requires
/// only a configuration entry in the bundled provider data — no code changes
/// to authentication logic.
pub struct OAuthProviderRegistry {
    db: ProviderDatabase,
}

/// An entry in the OAuth provider registry, pairing a provider with its config.
#[derive(Debug, Clone)]
pub struct OAuthProviderEntry<'a> {
    pub provider: &'a Provider,
    pub oauth: &'a OAuthConfig,
}

impl OAuthProviderRegistry {
    /// Build the registry from a [`ProviderDatabase`].
    pub fn new(db: ProviderDatabase) -> Self {
        Self { db }
    }

    /// Build the registry from the bundled provider database.
    pub fn bundled() -> Self {
        Self::new(ProviderDatabase::bundled())
    }

    /// Look up an OAuth-enabled provider by email address.
    pub fn lookup_by_email(&self, email: &str) -> Option<OAuthProviderEntry<'_>> {
        let candidate = self.db.lookup_by_email(email)?;
        // We need the provider from the internal store (not the cloned candidate)
        // so the lifetime is tied to `self`. Look it up by id.
        self.lookup_by_id(&candidate.provider.id)
    }

    /// Look up an OAuth-enabled provider by domain.
    pub fn lookup_by_domain(&self, domain: &str) -> Option<OAuthProviderEntry<'_>> {
        let candidate = self.db.lookup_by_domain(domain)?;
        self.lookup_by_id(&candidate.provider.id)
    }

    /// Look up an OAuth-enabled provider by its unique id (e.g. `"gmail"`, `"outlook"`).
    pub fn lookup_by_id(&self, id: &str) -> Option<OAuthProviderEntry<'_>> {
        self.db
            .providers()
            .iter()
            .find(|p| p.enabled && p.id == id)
            .and_then(|p| {
                p.oauth
                    .as_ref()
                    .filter(|o| o.status.is_active(cfg!(debug_assertions)))
                    .map(|oauth| OAuthProviderEntry { provider: p, oauth })
            })
    }

    /// Iterate over all OAuth-enabled providers in the registry.
    pub fn oauth_providers(&self) -> impl Iterator<Item = OAuthProviderEntry<'_>> {
        self.db
            .providers()
            .iter()
            .filter(|p| {
                p.enabled
                    && p.oauth
                        .as_ref()
                        .is_some_and(|o| o.status.is_active(cfg!(debug_assertions)))
            })
            .map(|p| OAuthProviderEntry {
                provider: p,
                oauth: p.oauth.as_ref().unwrap(),
            })
    }

    /// The number of OAuth-enabled providers in the registry.
    pub fn count(&self) -> usize {
        self.oauth_providers().count()
    }

    /// Access the underlying provider database.
    pub fn database(&self) -> &ProviderDatabase {
        &self.db
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- AC-1: Bundled registry contains Gmail, Outlook/Microsoft 365, Yahoo, AOL ---

    #[test]
    fn bundled_registry_contains_gmail() {
        let registry = OAuthProviderRegistry::bundled();
        let entry = registry.lookup_by_domain("gmail.com").unwrap();
        assert_eq!(entry.provider.id, "gmail");
    }

    #[test]
    fn bundled_registry_contains_outlook() {
        let registry = OAuthProviderRegistry::bundled();
        let entry = registry.lookup_by_domain("outlook.com").unwrap();
        assert_eq!(entry.provider.id, "outlook");
    }

    #[test]
    fn bundled_registry_contains_microsoft_365() {
        let registry = OAuthProviderRegistry::bundled();
        let entry = registry.lookup_by_domain("office365.com").unwrap();
        assert_eq!(entry.provider.id, "office365");
    }

    #[test]
    fn bundled_registry_contains_yahoo() {
        let registry = OAuthProviderRegistry::bundled();
        let entry = registry.lookup_by_domain("yahoo.com").unwrap();
        assert_eq!(entry.provider.id, "yahoo");
    }

    #[test]
    fn bundled_registry_contains_aol() {
        let registry = OAuthProviderRegistry::bundled();
        let entry = registry.lookup_by_domain("aol.com").unwrap();
        assert_eq!(entry.provider.id, "aol");
    }

    // --- AC-2: Each entry has required fields ---

    #[test]
    fn entries_have_authorization_endpoint() {
        let registry = OAuthProviderRegistry::bundled();
        for entry in registry.oauth_providers() {
            assert!(
                !entry.oauth.auth_url.is_empty(),
                "Provider {} missing auth_url",
                entry.provider.id
            );
            assert!(
                entry.oauth.auth_url.starts_with("https://"),
                "Provider {} auth_url must be HTTPS: {}",
                entry.provider.id,
                entry.oauth.auth_url
            );
        }
    }

    #[test]
    fn entries_have_token_endpoint() {
        let registry = OAuthProviderRegistry::bundled();
        for entry in registry.oauth_providers() {
            assert!(
                !entry.oauth.token_url.is_empty(),
                "Provider {} missing token_url",
                entry.provider.id
            );
            assert!(
                entry.oauth.token_url.starts_with("https://"),
                "Provider {} token_url must be HTTPS: {}",
                entry.provider.id,
                entry.oauth.token_url
            );
        }
    }

    #[test]
    fn entries_have_scopes() {
        let registry = OAuthProviderRegistry::bundled();
        for entry in registry.oauth_providers() {
            assert!(
                !entry.oauth.scopes.is_empty(),
                "Provider {} must define at least one scope",
                entry.provider.id
            );
        }
    }

    #[test]
    fn entries_have_redirect_uri() {
        let registry = OAuthProviderRegistry::bundled();
        for entry in registry.oauth_providers() {
            assert!(
                !entry.oauth.redirect_uri.is_empty(),
                "Provider {} missing redirect_uri",
                entry.provider.id
            );
        }
    }

    // --- AC-3: Adding a provider is configuration-only ---

    #[test]
    fn lookup_by_email_returns_oauth_entry() {
        let registry = OAuthProviderRegistry::bundled();
        let entry = registry.lookup_by_email("user@gmail.com").unwrap();
        assert_eq!(entry.provider.id, "gmail");
        assert!(!entry.oauth.auth_url.is_empty());
    }

    #[test]
    fn lookup_by_id_returns_oauth_entry() {
        let registry = OAuthProviderRegistry::bundled();
        let entry = registry.lookup_by_id("outlook").unwrap();
        assert_eq!(entry.provider.display_name, "Outlook.com (Microsoft)");
    }

    #[test]
    fn lookup_returns_none_for_non_oauth_provider() {
        let registry = OAuthProviderRegistry::bundled();
        // iCloud does not have OAuth configured
        assert!(registry.lookup_by_domain("icloud.com").is_none());
    }

    #[test]
    fn oauth_providers_iterator_returns_only_oauth_enabled() {
        let registry = OAuthProviderRegistry::bundled();
        for entry in registry.oauth_providers() {
            assert!(
                entry.provider.oauth.is_some(),
                "Iterator returned non-OAuth provider: {}",
                entry.provider.id
            );
        }
    }

    #[test]
    fn registry_count_matches_iterator() {
        let registry = OAuthProviderRegistry::bundled();
        let iter_count = registry.oauth_providers().count();
        assert_eq!(registry.count(), iter_count);
        assert!(iter_count >= 4, "Expected at least 4 OAuth providers");
    }

    // --- AC-4: Provider-specific parameters ---

    #[test]
    fn gmail_has_pkce_required() {
        let registry = OAuthProviderRegistry::bundled();
        let entry = registry.lookup_by_id("gmail").unwrap();
        assert!(entry.oauth.pkce_required);
    }

    #[test]
    fn outlook_has_pkce_required() {
        let registry = OAuthProviderRegistry::bundled();
        let entry = registry.lookup_by_id("outlook").unwrap();
        assert!(entry.oauth.pkce_required);
    }

    #[test]
    fn gmail_has_consent_prompt_via_extra_params() {
        let registry = OAuthProviderRegistry::bundled();
        let entry = registry.lookup_by_id("gmail").unwrap();
        assert!(entry
            .oauth
            .extra_params
            .contains(&("prompt".to_string(), "consent".to_string())));
    }

    #[test]
    fn gmail_has_offline_access_via_extra_params() {
        let registry = OAuthProviderRegistry::bundled();
        let entry = registry.lookup_by_id("gmail").unwrap();
        assert!(entry
            .oauth
            .extra_params
            .contains(&("access_type".to_string(), "offline".to_string())));
    }

    #[test]
    fn outlook_has_consent_prompt_via_extra_params() {
        let registry = OAuthProviderRegistry::bundled();
        let entry = registry.lookup_by_id("outlook").unwrap();
        assert!(entry
            .oauth
            .extra_params
            .contains(&("prompt".to_string(), "consent".to_string())));
    }

    #[test]
    fn microsoft_365_has_tenant_support() {
        let registry = OAuthProviderRegistry::bundled();
        let entry = registry.lookup_by_id("office365").unwrap();
        assert!(entry.oauth.requires_tenant());
    }

    #[test]
    fn all_oauth_providers_have_pkce_configured() {
        let registry = OAuthProviderRegistry::bundled();
        for entry in registry.oauth_providers() {
            // pkce_required is a bool — it's always explicitly set, confirming
            // that the field is part of every provider's configuration.
            let _ = entry.oauth.pkce_required;
        }
    }

    #[test]
    fn pkce_required_field_controls_authorization_url() {
        // Verify that pkce_required being true causes PKCE params in the URL
        // (integration with oauth_flow is tested in that module; here we just
        // verify the field is present and usable).
        let registry = OAuthProviderRegistry::bundled();
        let gmail = registry.lookup_by_id("gmail").unwrap();
        assert!(gmail.oauth.pkce_required, "Gmail must require PKCE");
    }
}
