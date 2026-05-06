# Setup Wizard OAuth Integration

## Parent Feature
#1.5 OAuth2 Sign-In

## User Story
As a mainstream user, when I enter my email address in the quick-setup wizard, I want the application to detect my provider, present OAuth as the default and recommended authentication method, initiate the OAuth flow, test the mail server connection with the new token, and create my account with discovered folders and a default sending identity, so that setup completes seamlessly in one pass.

## Blocked by
- `1-bundled-oauth-provider-database`
- `2-core-oauth-authorization-flow`

## Acceptance Criteria
- When the user enters an email address and the detected provider supports OAuth, the wizard presents OAuth as the default and recommended authentication method.
- The user can see which providers support OAuth sign-in, clearly distinguished from password-only providers.
- After successful token acquisition, the application tests the IMAP (or POP3) and SMTP connections using the new token.
- If the connection test succeeds, the application creates the account, discovers folders, and creates a default sending identity.
- If the connection test fails despite a valid token, a provider-specific error message is displayed with suggested corrective action (e.g. "Enable IMAP access in your Gmail settings").
- The wizard flow completes without the user needing to understand OAuth internals.

## Mapping to Epic
- FR-21 (OAuth as default when provider supports it)
- FR-22 (test IMAP/SMTP connection with new token)
- FR-23 (create account, discover folders, create default identity)
- FR-24 (provider-specific error on connection test failure)
- US-3, US-10, US-11, US-12
- AC-1 (completing OAuth creates a working account)

## HITL / AFK
HITL — user enters their email and completes the browser-based authorization.

## Notes
- Identity extraction (email, display name from token claims) is handled in a separate story (`4-identity-extraction-from-tokens`). This story uses whatever identity info is available, but the extraction logic itself is sliced separately.
- The wizard must gracefully handle the case where OAuth credentials are unavailable for the detected provider (covered in story `13-distribution-channel-gating-password-fallback`).
