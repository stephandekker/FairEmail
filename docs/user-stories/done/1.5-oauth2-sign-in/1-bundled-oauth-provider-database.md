# Bundled OAuth Provider Database

## Parent Feature
#1.5 OAuth2 Sign-In

## User Story
As a developer building the OAuth sign-in feature, I need a bundled provider database that maps email domains to their OAuth2 configurations (authorization endpoint, token endpoint, redirect URI, scopes, client credentials, and provider-specific parameters), so that the application knows how to initiate OAuth for each supported provider.

## Blocked by
_(none — this is the foundational slice)_

## Acceptance Criteria
- The application ships with OAuth2 configurations for at least: Gmail, Outlook/Office 365, Yahoo, AOL, Yandex, Mail.ru, Fastmail, and NetEase 163.
- Each provider configuration specifies: authorization endpoint, token endpoint, redirect URI, required scopes, and any provider-specific parameters (e.g. tenant placeholder, consent prompt, offline-access flag).
- Provider configurations can be looked up by email domain (e.g. `@gmail.com` resolves to the Gmail OAuth config).
- The provider database is loadable at application startup and available to other components.
- Provider entries clearly distinguish OAuth-capable providers from password-only providers.

## Mapping to Epic
- FR-1 (bundled provider database with OAuth configs for listed providers)
- FR-2 (each config specifies endpoints, redirect URI, scopes, provider-specific params)
- US-11 (OAuth providers distinguished from password-only)

## HITL / AFK
AFK — no human interaction required; this is data definition and loading logic.

## Notes
- The existing Android codebase uses `providers.xml` with an `OAuth` inner class on `EmailProvider`. The desktop implementation may choose a different format but must carry the same information.
- Design Note N-6 from the epic notes that provider-specific quirks (e.g. Gmail's `prompt=consent`, `access_type=offline`, Yandex's `force_confirm=true`) are baked into provider records, not user-configurable. These parameters should be included in the initial database even though the *behavior* of sending them is wired up in a later story (story 10).
