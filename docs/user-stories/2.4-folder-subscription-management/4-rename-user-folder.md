# Rename User Folder

## Parent Feature
#2.4 Folder Subscription Management

## User Story
As a folder organizer, I want to rename any user-created folder through the folder properties screen, so that I can correct naming mistakes or reorganize without deleting and recreating.

## Blocked by
_(none — rename is an independent operation, though it logically follows folder creation)_

## Acceptance Criteria
- The folder properties screen allows editing the folder name for user-created folders.
- System folders (Inbox, Sent, Drafts, Trash, Spam, Archive) have the name field disabled — they cannot be renamed.
- The new name is validated: empty names are rejected with a clear error.
- The new name is validated: duplicate names (same full name in the same account) are rejected with a clear error.
- The rename is queued locally (target name stored as a "rename" pending value) and executed on the server during the next sync cycle.
- If the folder was subscribed before rename, the old name is unsubscribed and the new name is re-subscribed on the server, preserving subscription state.
- Upon successful server-side rename, the local folder record is updated and the "rename" pending value is cleared.
- A rename triggers a full account reload so that any child folders or dependent references are updated.
- The rename does not alter the folder's synchronization setting, unified-inbox membership, or notification setting.

## Mapping to Epic
- Goals: G3
- User Stories: US-18, US-19, US-20, US-21, US-22
- Functional Requirements: FR-23, FR-24, FR-25, FR-26, FR-27, FR-28, FR-37
- Acceptance Criteria: AC-12, AC-13, AC-14, AC-21

## HITL / AFK
**AFK** — standard CRUD operation with well-defined acceptance criteria.

## Estimation
Medium — involves name validation, queued server execution, subscription preservation logic, and a full account reload on success.

## Notes
- OQ-3 in the epic asks whether the application should verify that child folders were renamed by the server or trust the server. The IMAP RENAME spec says child folders should be renamed automatically. This story relies on the full account reload (FR-28) to pick up whatever the server did. If server behaviour is unreliable, a follow-up story may be needed to verify child folder renames explicitly.
- The rename-preserves-subscription behaviour (unsubscribe old name, subscribe new name) is an explicit requirement because some IMAP servers do not automatically transfer subscription state on rename (see Design Note N-3).
