# Distribution-Channel Gating & Password Fallback

## Parent Feature
#1.5 OAuth2 Sign-In

## User Story
As a user running a build that does not include OAuth client credentials (e.g. a community-maintained package), I want the application to clearly inform me that OAuth is unavailable for my build and guide me to use password or app-password authentication instead, so that I am not confused by a flow that would fail.

## Blocked by
- `1-bundled-oauth-provider-database`
- `3-setup-wizard-oauth-integration`

## Acceptance Criteria
- The application detects at runtime when OAuth client credentials are not available for a given provider.
- When OAuth is unavailable, the OAuth option is hidden (not shown and then failing) for affected providers.
- A clear message explains why OAuth is unavailable and directs the user to password-based setup.
- For providers that support both OAuth and password authentication, the user can choose password authentication even when OAuth is available.
- OAuth is the recommended path when available, but not a locked gate — the manual-setup path always allows password authentication.

## Mapping to Epic
- FR-29 (detect missing OAuth credentials, guide to password)
- US-15 (inform user OAuth unavailable, guide to password)
- US-16 (choose password auth even when OAuth available)
- AC-6 (no OAuth option when credentials missing; message + password redirect)
- Design Note N-1 (OAuth as default, not exclusive)
- Design Note N-7 (distribution-channel gating)

## HITL / AFK
HITL — user reads the message and proceeds with password setup.

## Notes
- The existing Android codebase gates OAuth availability per distribution channel (e.g. F-Droid builds lack Google OAuth credentials). The desktop implementation needs an equivalent mechanism — likely a build-time flag or presence/absence of a credentials file.
- This story does not cover *converting* an existing OAuth account to password — that is story `14-auth-method-conversion`.
