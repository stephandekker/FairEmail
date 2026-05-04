# Delete User Folder

## Parent Feature
#2.4 Folder Subscription Management

## User Story
As a folder organizer, I want to delete any user-created folder via a context action on the folder list or from the folder properties screen, with appropriate safeguards, so that I can remove folders I no longer need without accidental data loss.

## Blocked by
_(none — delete is an independent operation, though it logically follows folder creation)_

## Acceptance Criteria
- A delete action is available in the folder context menu and on the folder properties screen for user-created folders.
- The delete action is **not** available for:
  - System folders (Inbox, Sent, Drafts, Trash, Spam, Archive).
  - Read-only folders.
  - Folders that have child folders.
- Activating the delete action presents a confirmation dialog that names the folder and warns the user that deletion is permanent and all messages in the folder will be lost.
- If the folder has pending operations (queued moves, flags, etc.), deletion is refused with an error message indicating the number of pending operations.
- Upon confirmation, the folder is marked "to be deleted" locally and the deletion is executed on the server during the next sync cycle.
- Server-side deletion first unsubscribes the folder, then deletes it.
- Upon successful server-side deletion, the folder is removed from the local database.
- If the folder no longer exists on the server at the time of deletion, the operation succeeds silently (idempotent).
- Deletion does not alter other folders' synchronization settings, unified-inbox membership, or notification settings.

## Mapping to Epic
- Goals: G4
- User Stories: US-23, US-24, US-25, US-26, US-27, US-28, US-29
- Functional Requirements: FR-29, FR-30, FR-31, FR-32, FR-33, FR-34, FR-35, FR-37
- Acceptance Criteria: AC-15, AC-16, AC-17, AC-18, AC-21
- Non-Functional Requirements: NFR-7

## HITL / AFK
**HITL** — the confirmation dialog wording needs UX review. This is a destructive operation.

## Estimation
Medium — involves multiple guard conditions, a confirmation dialog, queued server execution with unsubscribe-then-delete sequencing, and idempotent handling.

## Notes
- Design Note N-5 explains why deletion is blocked when children exist: recursive deletion is dangerous and orphaning children is confusing. The user must delete children first, bottom-up.
- Design Note N-6 explains why deletion is blocked when pending operations exist: queued moves/flags targeting the folder would fail in confusing ways.
