# OAuth Revocation Notification and Re-Authorization

## Parent Feature
#1.6 Authentication Methods

## User Story
As a user whose OAuth authorization has been revoked or whose refresh token has expired, I want the application to notify me clearly and offer a one-click re-authorization flow, so that I can restore access without deleting and re-adding the account.

## Acceptance Criteria
- When a refresh attempt fails due to revocation or expiration, the application surfaces a notification to the user.
- The notification includes an actionable option to re-authorize (one-click triggers the browser OAuth flow again).
- Re-authorization restores the account without losing folders, messages, or settings.
- The application does not repeatedly attempt refresh after a definitive revocation response.

## Blocked by
4-oauth-automatic-token-refresh

## HITL / AFK
AFK

## Notes
- NFR-7 (degradation clarity) applies here: if a provider has deprecated password access, error messaging should guide toward OAuth or app-specific passwords.
