# OAuth Token Refresh During IDLE Sessions

## Parent Feature
#7.1 IMAP IDLE (Push)

## User Story
As a user with an OAuth-authenticated account, I want the application to proactively refresh my token before it expires during an IDLE session, so that push is not interrupted and I do not need to re-authenticate manually.

## Blocked by
- `5-keep-alive-mechanism`
- `7-connection-failure-recovery`

## Acceptance Criteria
- The application monitors token expiry relative to the keep-alive schedule for token-authenticated accounts (FR-43).
- If the token will expire before the next scheduled keep-alive, the application refreshes the token proactively or expedites the keep-alive cycle (FR-43).
- An OAuth-authenticated account whose token expires mid-IDLE session refreshes the token and resumes IDLE without user-visible interruption, provided the refresh succeeds (AC-14).
- A token refresh failure is treated as a connection error and triggers the standard recovery path (exponential backoff, reconnection) (FR-44).
- If token refresh continues to fail, the application surfaces an authentication error to the user (FR-44).
- Token values are never included in user-accessible diagnostic logs (NFR-8).

## Mapping to Epic
- FR-43, FR-44
- NFR-8
- AC-14

## HITL / AFK
AFK — no human review needed beyond normal code review.

## Notes
- This story depends on the OAuth token refresh infrastructure from epic 1.5 (OAuth2 Sign-In). It only covers the integration between token lifecycle and IDLE session management — not the token refresh mechanism itself.
- The epic's non-goal NG5 clarifies that OAuth token lifecycle management is out of scope as a standalone concern — this story only ensures token expiry does not silently kill IDLE sessions.
