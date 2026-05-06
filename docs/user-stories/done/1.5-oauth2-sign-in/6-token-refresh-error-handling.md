# Token Refresh Error Handling

## Parent Feature
#1.5 OAuth2 Sign-In

## User Story
As a user whose token refresh fails, I want the application to distinguish between transient errors (network issues) and permanent errors (revoked token), retry gracefully for transient failures, and notify me with a clear re-authorization path for permanent failures, so that brief outages don't break my account and real problems are surfaced promptly.

## Blocked by
- `5-automatic-token-refresh`

## Acceptance Criteria
- Transient errors (network timeout, server 5xx) trigger retry with backoff; the application tolerates up to 90 seconds of network failure before declaring a permanent failure.
- During transient failures, the last valid access token continues to be used for cached/local operations.
- If the application is offline when a refresh is due, it refreshes as soon as connectivity is restored.
- Permanent errors (refresh token revoked, user consent withdrawn, HTTP 400/401 from token endpoint) cause the account to be marked as requiring re-authorization.
- The user is notified clearly when re-authorization is required, with a one-click path to re-authorize.
- Rate-limit or temporary-block errors from the provider are communicated to the user with a suggestion to wait or re-authorize.
- The application does not immediately mark the account as broken on the first refresh failure.

## Mapping to Epic
- FR-17 (retry with backoff for transient errors)
- FR-18 (mark account for re-auth on permanent error, notify user)
- NFR-2 (tolerate 90 seconds of transient failure)
- NFR-10 (offline resilience — refresh on connectivity restore)
- US-7 (notify and offer re-auth on revocation)
- US-8 (retry gracefully on transient errors)
- US-26 (rate-limit/block error communication)
- AC-3 (re-auth prompt within one refresh cycle of revocation)
- AC-8 (transient failure does not immediately break account)

## HITL / AFK
AFK for retry logic; HITL only when permanent failure requires the user to re-authorize.

## Notes
- The boundary between "transient" and "permanent" errors may be provider-specific. HTTP 400 with `invalid_grant` is generally permanent; HTTP 503 is transient. The implementation should maintain a classification of error responses.
