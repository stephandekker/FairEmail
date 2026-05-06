# Provider-Specific OAuth Parameters

## Parent Feature
#1.5 OAuth2 Sign-In

## User Story
As a user signing in with a provider that has specific OAuth requirements (e.g. Gmail requiring `prompt=consent` and `access_type=offline`, Yandex requiring `force_confirm=true`), I want the application to automatically include those parameters in the authorization request, so that the flow succeeds and a refresh token is returned without me needing to know about these details.

## Blocked by
- `1-bundled-oauth-provider-database`
- `2-core-oauth-authorization-flow`

## Acceptance Criteria
- For providers requiring explicit consent prompts (e.g. Gmail's `prompt=consent`), the authorization request includes the required parameter.
- For providers requiring explicit offline-access flags (e.g. Gmail's `access_type=offline`), the authorization request includes the required parameter and a refresh token is returned.
- For providers requiring confirmation flags (e.g. Yandex's `force_confirm=true`), the authorization request includes the required parameter.
- Provider-specific parameters are read from the provider database, not hard-coded in flow logic.
- The application requests only the minimum scopes sufficient for mail access (read, write, send) — no profile, contacts, calendar, or other scopes unless inseparably bundled by the provider.
- For providers offering a proprietary mail-sending API via a broader OAuth grant (e.g. Outlook Graph API), the application requests the broader grant when it improves SMTP reliability, transparently to the user.

## Mapping to Epic
- FR-37 (consent prompt parameters)
- FR-38 (offline-access flags)
- FR-39 (broader grant for proprietary send API)
- NFR-6 (minimal scopes)
- US-5 (minimum necessary permissions)
- AC-14 (Gmail consent/offline parameters, refresh token returned)
- Design Note N-5 (Graph API as provider variant)
- Design Note N-6 (provider-specific quirks in provider database)

## HITL / AFK
AFK — provider-specific parameters are applied automatically; the user sees no difference.

## Notes
- Open Question OQ-4 asks whether the Graph API send path belongs in this epic or a separate one. The epic currently includes it minimally via FR-39. This story covers requesting the necessary scopes; the actual Graph API transport implementation may belong elsewhere.
- The existing Android codebase maintains `AUTH_TYPE_GRAPH` (4) as a separate auth type for Outlook Graph. The desktop implementation should decide whether to model this as a provider variant or a separate auth type.
