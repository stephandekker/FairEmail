# Connection Lifecycle Logging and Metrics

## Parent Feature
#7.1 IMAP IDLE (Push)

## User Story
As any user, I want the application to log significant connection lifecycle events and record per-account metrics (current keep-alive interval, stability state, last successful interaction), so that I can diagnose push issues when they occur.

## Blocked by
- `5-keep-alive-mechanism`
- `7-connection-failure-recovery`

## Acceptance Criteria
- The application logs significant connection lifecycle events: connect, disconnect, keep-alive failure, interval tuning, auto-optimization trigger (FR-41).
- Logs are accessible to the user via a log or debug view (FR-41).
- The application records per-account metrics: current keep-alive interval, whether the interval is confirmed stable, and the timestamp of the last successful server interaction (FR-42).
- No credentials or tokens are included in user-accessible diagnostics (NFR-8).

## Mapping to Epic
- FR-41, FR-42
- NFR-8

## HITL / AFK
AFK — no human review needed beyond normal code review.

## Notes
- This story covers the data model and logging infrastructure. The user-facing presentation (where logs appear, how metrics are displayed) overlaps with story 14's status display but focuses on the diagnostic/debug view rather than the at-a-glance status.
