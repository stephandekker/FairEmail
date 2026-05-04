# Incremental Sync via Modification Sequences (MODSEQ)

## Parent Feature
#2.1 Two-Way IMAP Synchronization

## User Story
As any user with a large mailbox, I want synchronization to be fast — when my server supports modification-sequence tracking, the application should fetch only messages that have changed since the last sync, rather than comparing the entire folder every time.

## Blocked by
4-server-flag-change-detection, 10-new-message-detection

## Acceptance Criteria
- When the server supports CONDSTORE/QRESYNC, the application stores the highest known modification sequence after each successful sync.
- On subsequent sync cycles, the application requests only messages with a modification sequence higher than the stored value.
- Only changed messages are fetched and processed — unchanged messages are skipped.
- A sync cycle with no changes completes in under 2 seconds for a folder with 10,000 messages (AC-19).
- The application correctly handles the case where MODSEQ support is advertised but the folder has never been synced before (initial sync is full).

## HITL / AFK
**AFK** — transparent optimization with no user interaction.

## Estimation
Medium — requires CONDSTORE/QRESYNC protocol handling and stored-state management.

## Notes
- US-23, FR-10, AC-19, NFR-5 are the primary drivers.
- N-7 explains that MODSEQ is an optimization, not a requirement — the full-sync fallback (story 17) is the baseline.
