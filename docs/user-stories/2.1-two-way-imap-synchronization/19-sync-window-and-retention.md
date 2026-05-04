# Sync Window and Local Retention Configuration

## Parent Feature
#2.1 Two-Way IMAP Synchronization

## User Story
As any user with a large mailbox, I want to configure how far back the application synchronizes (sync window) and how long messages are kept locally (keep window), per folder, so that I can balance completeness against storage and bandwidth.

## Blocked by
17-full-sync-fallback

## Acceptance Criteria
- The user can configure a sync window (number of days) per folder.
- The user can configure a keep window (number of days) per folder. The keep window must be >= the sync window.
- Messages outside the sync window are not checked for flag changes on routine sync cycles (AC-24).
- Messages outside the keep window are removed from local storage. This removal is local only — messages are not deleted from the server (FR-44).
- Messages within the keep window but outside the sync window remain accessible locally (read-only flags, not actively synced).
- Default values are provided for accounts/folders that the user has not explicitly configured.

## HITL / AFK
**HITL** — requires user configuration (settings UI).

## Estimation
Medium — per-folder configuration, enforcement during sync, and local cleanup logic.

## Notes
- US-25, FR-41, FR-42, FR-43, FR-44, AC-24 are the primary drivers.
- OQ-6 (sync window interaction with search) is an open question in the epic that may affect this story's edge cases.
