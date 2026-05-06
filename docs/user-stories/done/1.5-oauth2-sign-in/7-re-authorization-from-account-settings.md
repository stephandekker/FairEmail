# Re-authorization from Account Settings

## Parent Feature
#1.5 OAuth2 Sign-In

## User Story
As a user whose OAuth tokens have been revoked or are malfunctioning, I want to re-run the OAuth flow from my account settings without deleting and re-adding the account, so that I can restore mail access while keeping all my account settings, folder structure, rules, and identities intact.

## Blocked by
- `2-core-oauth-authorization-flow`
- `3-setup-wizard-oauth-integration`

## Acceptance Criteria
- The account settings screen provides an option to re-authorize (re-run the OAuth flow) for OAuth-authenticated accounts.
- Re-authorization opens the system browser to the provider's authorization endpoint, just like initial sign-in.
- On successful re-authorization, the new tokens replace the old ones.
- All account settings, folder structure, rules, identities, and local messages are preserved after re-authorization.
- The re-authorization option is accessible from the notification/prompt shown when the application detects that tokens are permanently invalid (ties into story 6).

## Mapping to Epic
- FR-25 (update tokens by re-running OAuth from account settings)
- US-18 (re-authorize from account settings)
- US-19 (new tokens replace old, preserving all account state)
- AC-4 (re-auth replaces tokens, resumes sync, no data loss)

## HITL / AFK
HITL — user must interact with the browser to re-authenticate.

## Notes
- This story reuses the core OAuth flow from story 2 but triggers it from account settings rather than the setup wizard. The key new behavior is token replacement on an existing account rather than account creation.
