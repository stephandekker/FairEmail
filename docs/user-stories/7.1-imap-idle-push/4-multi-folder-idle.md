# Multi-Folder Concurrent IDLE Sessions

## Parent Feature
#7.1 IMAP IDLE (Push)

## User Story
As a multi-folder monitor, I want push to be active on every folder I have marked for synchronization (not just the Inbox), so that I am notified of changes in project folders, shared folders, and other non-Inbox locations in near-real-time.

## Blocked by
- `2-single-folder-idle-session`

## Acceptance Criteria
- Enabling synchronization on any folder (on an IDLE-capable server) results in that folder entering push mode automatically, independently of the Inbox (AC-3).
- Multiple IDLE sessions are maintained concurrently — one dedicated connection per eligible folder — across all configured accounts (FR-8).
- There is no arbitrary cap on concurrent IDLE sessions below the user's configured scope (FR-9).
- Each folder's IDLE session operates independently: a failure on one folder does not affect IDLE on other folders.
- With 5 accounts and 10 push-enabled folders each (50 concurrent sessions), the application's idle CPU usage remains negligible (AC-13, NFR-2).
- Each IDLE session consumes minimal CPU when idle and minimal memory (NFR-3).

## Mapping to Epic
- US-2, US-4
- FR-5, FR-8, FR-9
- NFR-2, NFR-3
- AC-3, AC-13

## HITL / AFK
AFK — no human review needed beyond normal code review.

## Notes
- The epic specifies support for at least 200 concurrent IDLE sessions (10 accounts × 20 folders). The acceptance criteria test with 50 as a baseline, but the implementation should not have architectural limits below 200.
- Open Question OQ-5 (concurrent connection limits): some IMAP servers impose per-user connection limits. This story does not address automatic fallback when limits are exceeded — that is flagged as an open question in the epic. If discovered during implementation, document the behavior and consider a follow-up story.
- Open Question OQ-1 (NOTIFY): RFC 5465 NOTIFY could reduce connection count by monitoring multiple folders over a single connection. This story implements the per-folder IDLE model. NOTIFY exploitation is explicitly a non-goal (NG3) but the capability is detected in story 1.
