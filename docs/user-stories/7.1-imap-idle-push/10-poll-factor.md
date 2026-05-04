# Per-Folder Poll Frequency Factor

## Parent Feature
#7.1 IMAP IDLE (Push)

## User Story
As any user, I want to assign a poll factor to low-priority folders so that they are checked less frequently than the account's base interval, reducing unnecessary network traffic without disabling synchronization entirely.

## Blocked by
- `9-per-folder-push-poll-control`
- `5-keep-alive-mechanism`

## Acceptance Criteria
- Each folder may have a polling frequency multiplier (poll factor) that causes it to be checked less frequently than the account's base interval (FR-12).
- A factor of 1 means "check every cycle", 2 means "every other cycle", etc. (Design Note N-6).
- A folder with a poll factor of 2 on an account with a 15-minute keep-alive interval is checked every 30 minutes (every second cycle), not every 15 minutes (AC-17).
- Poll-only folders on push-capable accounts are checked opportunistically during keep-alive cycles, respecting their poll factor (FR-13).
- The default poll factor is 1 (check every cycle).

## Mapping to Epic
- FR-12, FR-13
- AC-17

## HITL / AFK
AFK — no human review needed beyond normal code review.

## Notes
- The poll factor is a simple multiplier on the account's base keep-alive/poll interval — not a separate interval per folder. This keeps configuration simple (Design Note N-6).
- The UI for setting the poll factor should be accessible from the folder's properties, alongside the push/poll toggle from story 9.
