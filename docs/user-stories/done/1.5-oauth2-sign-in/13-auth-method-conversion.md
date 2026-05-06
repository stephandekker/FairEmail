# Authentication Method Conversion

## Parent Feature
#1.5 OAuth2 Sign-In

## User Story
As a user, I want to convert an existing account from OAuth to password authentication (or from password to OAuth) without losing my account settings, folder structure, rules, identities, or local messages, so that I can change my authentication method as my needs evolve.

## Blocked by
- `2-core-oauth-authorization-flow`
- `3-setup-wizard-oauth-integration`
- `7-re-authorization-from-account-settings`

## Acceptance Criteria
- An OAuth-authenticated account can be converted to password authentication from the account settings.
- A password-authenticated account can be converted to OAuth from the account settings (triggers the OAuth flow for the account's provider).
- After conversion in either direction, all account settings, folder state, rules, identities, and local messages are preserved.
- The new authentication method is immediately used for subsequent connections.
- The old credential (token or password) is removed after successful conversion.

## Mapping to Epic
- FR-30 (convert OAuth ↔ password, preserving all state)
- US-17 (convert without losing settings/folders/messages)
- AC-11 (conversion preserves all account state)

## HITL / AFK
HITL — user initiates the conversion from account settings and (for password→OAuth) completes the browser flow.

## Notes
- Converting from password to OAuth reuses the core OAuth flow (story 2) and re-authorization mechanism (story 7). The key new behavior is changing the auth type on an existing account rather than replacing tokens of the same type.
- The epic does not specify whether a confirmation dialog should warn the user before conversion. Recommend confirming, since credential changes are significant.
