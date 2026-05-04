# Poll Mode Fallback for Non-IDLE Servers

## Parent Feature
#7.1 IMAP IDLE (Push)

## User Story
As a legacy-server user, when my server does not support IDLE, I want the application to automatically fall back to periodic polling at a reasonable default interval, so that I still receive mail without errors or manual setup.

## Blocked by
- `1-capability-detection`

## Acceptance Criteria
- When a server does not advertise IDLE, the application enters poll mode for all synchronized folders on that account.
- The default polling interval is 15 minutes (FR-11).
- Polling checks for new messages, removals, and flag changes — the same change types as IDLE.
- No error or warning is shown to the user when falling back to poll mode (AC-2).
- The user can configure the polling interval per account.
- Polling is scheduled via a system-level timer/alarm, not an in-process sleep loop (Design Note N-2).
- Poll mode serves as the universal safety net: there is no state where a synchronized folder receives neither push nor poll (Design Note N-5).

## Mapping to Epic
- US-3
- FR-10, FR-11
- NFR-5
- AC-2

## HITL / AFK
AFK — no human review needed beyond normal code review.

## Notes
- This story does not include per-folder poll factors (story 10) or poll-only overrides on IDLE-capable servers (story 9). It covers only the base case: entire account falls back to poll because the server lacks IDLE.
- The polling interval configuration UI is covered in story 16 (global/per-account settings). This story provides the underlying mechanism and the default.
