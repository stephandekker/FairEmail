# Full Sync Fallback (No MODSEQ)

## Parent Feature
#2.1 Two-Way IMAP Synchronization

## User Story
As any user whose server does not support modification-sequence tracking, I want the application to still detect changes reliably by comparing the full message list within the sync window against local state, so that synchronization works on all servers.

## Blocked by
4-server-flag-change-detection, 10-new-message-detection, 11-message-removal-detection

## Acceptance Criteria
- When the server does not advertise CONDSTORE/QRESYNC, the application falls back to comparing the full UID list and flags for messages within the sync window.
- New messages, removed messages, and flag changes are all detected via this comparison.
- The sync window limits how far back the comparison goes (messages older than the window are not checked).
- A sync cycle triggered within a short interval of a previous successful sync (e.g. within 30 seconds) performs a full comparison rather than relying on cached state (FR-12).
- The application functions correctly without any optional IMAP extensions (AC-23).

## HITL / AFK
**AFK** — transparent fallback behavior.

## Estimation
Medium — UID/flag comparison logic for potentially large folders, with sync-window scoping.

## Notes
- FR-11, FR-12, FR-47, AC-23 are the primary drivers.
- N-7 explains the relationship between MODSEQ (optimization) and this fallback (baseline).
