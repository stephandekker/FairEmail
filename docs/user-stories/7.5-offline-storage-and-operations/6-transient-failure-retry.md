# Transient Failure Retry with Backoff

## Parent Feature
#7.5 Offline Storage and Operations

## User Story
As any user, I want transient failures (timeouts, temporary server errors, rate limits) to be retried automatically a reasonable number of times, so that I am not bothered by normal network hiccups.

## Acceptance Criteria
- Each operation maintains a try counter, incremented on each execution attempt.
- On transient failure (network timeout, temporary server error, rate limit), the operation is retried automatically up to a configured maximum number of attempts.
- A brief delay is introduced between retry attempts to avoid hammering a struggling server.
- An operation that fails due to a transient error and succeeds on a subsequent retry completes without user action.
- The operation remains in "pending" state during retries (not marked as failed until max attempts exhausted).

## Complexity
Small

## Blocked by
3-replay-single-operation

## HITL/AFK
AFK

## Notes
- OQ-1 in the epic asks whether the retry ceiling should be user-configurable or a fixed default. For the initial implementation, a sensible fixed default is sufficient; configurability can be added later if needed.
- The backoff strategy is not specified in detail by the epic. A simple fixed delay or linear backoff is acceptable; exponential backoff is an option but not required.
