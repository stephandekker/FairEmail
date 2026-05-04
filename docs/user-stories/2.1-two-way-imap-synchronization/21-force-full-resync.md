# Force Full Resynchronization

## Parent Feature
#2.1 Two-Way IMAP Synchronization

## User Story
As any user, when I suspect the local state has drifted from the server, I want to force a full resynchronization of a folder without having to remove and re-add the account.

## Blocked by
17-full-sync-fallback

## Acceptance Criteria
- The user can trigger a full resync for any individual folder at any time.
- A forced full resync ignores any cached MODSEQ state and performs a complete UID/flag comparison within the sync window.
- After the forced resync completes, local state matches server state (within the sync window) for that folder.
- The action is available via the UI (e.g. context menu on a folder).
- No data loss occurs — messages with pending operations are handled per the conflict resolution rules.

## HITL / AFK
**HITL** — user-initiated action.

## Estimation
Small — reuses existing full-sync logic; this story adds the user-facing trigger.

## Notes
- US-24, FR-13 are the primary drivers.
