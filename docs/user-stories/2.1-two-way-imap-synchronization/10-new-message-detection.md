# New Message Detection (Server to Local)

## Parent Feature
#2.1 Two-Way IMAP Synchronization

## User Story
As any user, when a new message arrives on the server (delivered by another sender, or appended by another client), I want it to appear in my local folder automatically on the next sync cycle without any manual action.

## Blocked by
1-persistent-operation-queue

## Acceptance Criteria
- During a sync cycle, the application detects messages on the server that are not in the local store (within the sync window).
- New messages are added to the local store with their full envelope and flags.
- Body and attachment download follows the configured policy (e.g. headers-only, full download, on-demand).
- The new message appears in the correct local folder.
- No duplicate messages are created if the sync cycle runs again before new messages arrive.
- A message delivered to the server appears in the desktop client after the next sync cycle (AC-8).

## HITL / AFK
**AFK** — automated detection with no user action required.

## Estimation
Medium — involves UID-based comparison logic and envelope/body fetch decisions.

## Notes
- US-10, FR-5, FR-6, AC-8 are the primary drivers.
- This story does not define *when* a sync cycle runs (that's feature 7.1 / scheduling policy NG5). It defines what happens when one does run.
