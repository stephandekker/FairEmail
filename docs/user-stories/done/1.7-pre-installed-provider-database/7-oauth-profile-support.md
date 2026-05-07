# User Story: OAuth Profile Support

## Parent Feature
#1.7 Pre-installed Provider Database

## Description
As a **user of a provider that supports OAuth** (e.g. Gmail, Outlook, Yahoo), I want the application to offer an OAuth sign-in flow using credentials and endpoints bundled in the provider catalogue, so that I can authenticate securely without creating or managing an app-specific password.

This slice adds OAuth profile support to the provider data model and the setup flow:
- A provider entry may include an OAuth profile with: client ID, optional client secret, required scopes, authorization endpoint, token endpoint, redirect URI, optional privacy policy URL, optional prompt behaviour, and optional custom parameters (FR-20).
- When a provider with an enabled OAuth profile is matched, the application offers OAuth-based sign-in as the preferred authentication method (FR-21).
- OAuth and Graph profiles may each be independently enabled, disabled, or restricted to debug mode (FR-24).
- OAuth credentials and endpoints are bundled, requiring no network fetch for configuration discovery (US-8, NFR-7).

## Acceptance Criteria
- [ ] A provider entry can include an OAuth profile with all specified attributes (client ID, scopes, endpoints, redirect URI, etc.).
- [ ] When a matched provider has an enabled OAuth profile, the setup flow offers OAuth sign-in as the preferred authentication method (AC-7).
- [ ] The OAuth flow uses the bundled client credentials and endpoint URLs — no network call is needed to discover OAuth configuration.
- [ ] OAuth profiles can be independently marked as enabled, disabled, or debug-only.
- [ ] A disabled OAuth profile does not trigger the OAuth sign-in offer; the user falls back to password authentication.
- [ ] The application uses OAuth flows appropriate for public clients (e.g. PKCE) where the provider supports them (NFR-7).

## Blocked by
`2-server-settings-prefill`

## HITL / AFK
**AFK** — OAuth flow integration is well-defined. The bundled credentials are data; the OAuth protocol handling is a standard library concern.

## Notes
- NFR-7 explicitly states that client secrets bundled in the catalogue are not truly secret. The implementation must not rely on client-secret confidentiality for security — use PKCE or equivalent public-client flows.
- OQ-3 in the epic asks whether there should be a fallback for revoked/rotated OAuth credentials. This story implements the current "update the application" approach. If a fallback mechanism is desired, it should be a separate story.
