# Automatic Keep-Alive Interval Tuning

## Parent Feature
#7.1 IMAP IDLE (Push)

## User Story
As any user, I want the application to automatically discover the optimal keep-alive interval for each account by tracking failures and reducing the interval until stable, so that I do not have to guess a value or tune it manually.

## Blocked by
- `5-keep-alive-mechanism`

## Acceptance Criteria
- The application tracks keep-alive failures per account (FR-19).
- After 3 consecutive keep-alive failures at the current interval, the application reduces the interval by 2 minutes and retries (FR-19, AC-7).
- Interval reduction continues step-wise until the configured minimum (9 minutes) is reached (FR-20).
- Once a stable interval is found (no failures over a sustained period), the interval is marked as "confirmed" and tuning stops (FR-21).
- Auto-tuning direction is always downward — the interval never increases automatically (Design Note N-3).
- The user can disable automatic tuning and lock in a manually chosen interval (FR-22) — the UI for this is in story 16.
- The confirmed interval and tuning state are persisted per account, surviving restarts.

## Mapping to Epic
- US-9, US-10, US-24
- FR-19, FR-20, FR-21, FR-22
- AC-7, AC-15 (partially — user override in story 16)

## HITL / AFK
AFK — no human review needed beyond normal code review.

## Notes
- The threshold of 3 consecutive failures and the 2-minute step size are configurable defaults from the epic. These should be constants that can be adjusted later without structural changes.
- The "confirmed" state should prevent unnecessary further reduction, but must reset if failures recur.
