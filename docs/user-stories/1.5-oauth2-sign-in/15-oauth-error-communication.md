# OAuth Error Communication

## Parent Feature
#1.5 OAuth2 Sign-In

## User Story
As a user, when the OAuth flow fails at any stage (provider error, network error, permission denied, rate limit), I want a clear, actionable error message — not a raw protocol error — so that I know what happened and what to do next.

## Blocked by
- `2-core-oauth-authorization-flow`
- `3-setup-wizard-oauth-integration`

## Acceptance Criteria
- Provider errors during authorization (e.g. access denied, invalid scope) are translated into user-friendly messages with suggested next steps.
- Network errors during token exchange are presented clearly, distinguishing temporary connectivity issues from permanent failures.
- Permission-denied errors explain what permission was missing and how to grant it.
- Rate-limit or temporary-block responses from the provider are communicated with a suggestion to wait before retrying.
- Raw protocol error codes or OAuth error strings are not shown to the user under normal operation (a debug/diagnostic mode may expose them).
- Error messages are keyboard-navigable and announced by screen readers.

## Mapping to Epic
- US-25 (clear, actionable error messages for all failure modes)
- US-26 (rate-limit/block error communication)
- NFR-9 (error messages accessible — keyboard, screen reader, themed)

## HITL / AFK
HITL — the user reads the error message and decides on next steps.

## Notes
- This story focuses on the user-facing presentation of errors. The underlying error detection and classification (transient vs. permanent) is covered in story `6-token-refresh-error-handling` for refresh errors. This story ensures all OAuth-related errors — during initial auth, token exchange, and refresh — are presented consistently and helpfully.
- The epic does not enumerate all possible error scenarios. The implementation should map common OAuth error codes (`invalid_grant`, `access_denied`, `temporarily_unavailable`, etc.) to user-friendly messages.
