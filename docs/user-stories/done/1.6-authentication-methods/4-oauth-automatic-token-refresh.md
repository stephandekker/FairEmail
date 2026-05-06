# OAuth Automatic Token Refresh

## Parent Feature
#1.6 Authentication Methods

## User Story
As a user with an OAuth-authenticated account, I want the application to refresh my access token automatically in the background before it expires, so that I am never interrupted by expiration unless my authorization has been genuinely revoked.

## Acceptance Criteria
- Access tokens are refreshed automatically before they expire.
- Token refresh does not visibly delay mail synchronization under normal network conditions (NFR-2).
- A minimum interval between consecutive refresh attempts for the same account is enforced to prevent hammering the provider's token endpoint (NFR-3 / AC-15).
- Even if multiple sync cycles trigger within the minimum interval window, refresh does not occur more frequently than the configured minimum.
- Cached OAuth tokens are stored durably so the application can authenticate immediately after restart without a network round-trip, as long as the token has not expired (NFR-5).

## Blocked by
3-oauth-browser-authorization-flow

## HITL / AFK
AFK

## Notes
- The minimum refresh interval value is not specified in the epic; this is a design decision to be made during implementation.
