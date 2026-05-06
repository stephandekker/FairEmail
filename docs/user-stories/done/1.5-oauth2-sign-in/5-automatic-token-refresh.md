# Automatic Token Refresh

## Parent Feature
#1.5 OAuth2 Sign-In

## User Story
As a user with an OAuth-authenticated account, I want the application to refresh my access token automatically before it expires, so that my mail keeps synchronizing without any action from me.

## Blocked by
- `2-core-oauth-authorization-flow`

## Acceptance Criteria
- The application refreshes the access token proactively (before expiry), not reactively (after a server rejection).
- Token refresh completes within the duration of a normal connection setup, so the user perceives no delay.
- While a refresh is in progress, the most recently valid access token continues to be used for in-flight operations.
- Concurrent refresh attempts for the same provider are serialized (one refresh in flight per provider at a time) to avoid rate-limit triggers.
- A successful refresh atomically updates the stored token state (access token, refresh token if rotated, expiry timestamp).
- Mail synchronization continues uninterrupted for at least 7 days without user interaction after initial setup.

## Mapping to Epic
- FR-15 (automatic refresh without user interaction)
- FR-16 (proactive, not reactive)
- FR-19 (serialize concurrent refreshes per provider)
- FR-20 (atomic update of stored token state)
- NFR-1 (refresh latency within normal connection setup)
- NFR-7 (concurrency serialization)
- US-6 (automatic refresh)
- US-9 (in-flight operations use last valid token)
- AC-2 (7 days uninterrupted sync)
- Design Note N-3 (one refresh at a time per provider)

## HITL / AFK
AFK — token refresh is fully automatic and invisible to the user.

## Notes
- The existing Android app uses a `ReentrantLock` per provider ID and a minimum 15-minute interval between forced refreshes. The desktop implementation should adopt equivalent safeguards.
- This story covers the happy path. Error handling (transient failures, revoked tokens) is in story `6-token-refresh-error-handling`.
