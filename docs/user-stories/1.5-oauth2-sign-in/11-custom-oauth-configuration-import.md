# Custom OAuth Configuration Import

## Parent Feature
#1.5 OAuth2 Sign-In

## User Story
As a self-hoster running my own identity provider (e.g. Keycloak, Authelia), I want to import a configuration file that defines my custom OAuth2 endpoints, scopes, and client credentials, so that I can use OAuth sign-in with my private mail server.

## Blocked by
- `1-bundled-oauth-provider-database`
- `2-core-oauth-authorization-flow`
- `3-setup-wizard-oauth-integration`

## Acceptance Criteria
- The application allows importing a provider configuration file that defines custom OAuth2 endpoints, scopes, client credentials, and server settings.
- The import mechanism is accessible from the application's advanced or debug settings.
- Imported custom providers appear in the setup wizard alongside bundled providers.
- The sign-in experience for a custom provider is identical to a bundled provider (same OAuth flow, same connection test, same account creation).
- Completing the OAuth flow with an imported custom provider creates a working account.

## Mapping to Epic
- FR-3 (provider database extensible by user)
- FR-26 (import provider configuration file)
- FR-27 (custom providers in wizard alongside bundled)
- FR-28 (import from advanced/debug settings)
- US-13 (import custom OAuth config)
- US-14 (custom provider appears in wizard)
- AC-7 (import + setup wizard = working account)

## HITL / AFK
HITL — user selects and imports a configuration file.

## Notes
- The existing Android codebase uses XML format for provider configurations. The desktop implementation may choose a different format (e.g. JSON, TOML) but must carry the same information fields.
- The epic does not specify validation rules for imported configurations. Recommend validating that required fields (authorization endpoint, token endpoint, at least one scope) are present before accepting the import.
- Open Question OQ-7 is tangentially relevant: if a custom provider specifies non-HTTPS endpoints, should the application warn or refuse? The epic raises this for NetEase 163 specifically, but it applies to custom configs too.
