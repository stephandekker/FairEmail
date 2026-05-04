# Operations Queue View

## Parent Feature
#7.5 Offline Storage and Operations

## User Story
As a power user, I want to view all pending operations in a dedicated operations screen, showing operation type, target message/folder, account, and current status, so that I know what is waiting to happen.

## Acceptance Criteria
- The application provides a dedicated operations view accessible from the main UI.
- The view shows all pending and failed operations.
- Each operation displays: operation type, target message/folder, owning account, status (pending/executing/failed), error message (if failed), and creation time.
- The view updates in real-time as operations are created, executed, or fail.
- The view is usable both online and offline.

## Complexity
Medium

## Blocked by
1-persist-operation-record
7-permanent-failure-handling

## HITL/AFK
AFK

## Notes
- This story provides the read-only view. Manipulation actions (cancel, bulk clear) are separate stories that build on this view.
- The specific UI layout is not prescribed by the epic — only the information that must be shown (FR-27).
