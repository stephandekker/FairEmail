# Message Removal Detection (Server to Local)

## Parent Feature
#2.1 Two-Way IMAP Synchronization

## User Story
As any user, when a message is deleted or moved away from a folder on the server (by another client, by a server-side rule, or via webmail), I want the desktop client to reflect that change — the message should disappear from the local folder.

## Blocked by
10-new-message-detection

## Acceptance Criteria
- During a sync cycle, the application detects messages that exist locally but no longer exist on the server (within the sync window).
- Such messages are removed from the local store (or marked as server-deleted, depending on retention policy).
- A message deleted on the server by another client disappears from the desktop client after the next sync cycle (AC-9).
- A message moved on the server appears in the new folder and disappears from the old folder after sync.
- No false removals: messages outside the sync window that are still within the keep window are not removed just because they weren't seen in the server response.

## HITL / AFK
**AFK** — automated removal with no user decisions.

## Estimation
Medium — requires careful UID comparison to avoid false removals for messages outside the sync window.

## Notes
- US-11, FR-5, FR-7, AC-7, AC-9 are the primary drivers.
- The interaction with sync window / keep window is important: only messages within the sync window are evaluated for removal.
