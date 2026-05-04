# Bulk Clear Operations by Type

## Parent Feature
#7.5 Offline Storage and Operations

## User Story
As a power user, I want to clear operations in bulk by type (e.g. clear all failed operations, clear all pending fetch operations), so that I can manage a backlog efficiently.

## Acceptance Criteria
- The user can delete operations in bulk, filtered by type (e.g. all failed, all fetch, all move, all flag, all delete, all send).
- Bulk-clearing failed operations removes them from the queue without affecting pending operations.
- When bulk-cleared operations introduced optimistic UI state, that state is reverted appropriately.
- The bulk action is confirmed before execution (to prevent accidental mass-cancellation).
- The operations view updates immediately after bulk clear.

## Complexity
Small

## Blocked by
14-operations-queue-view
15-cancel-operation-with-revert

## HITL/AFK
AFK

## Notes
- This builds on the individual cancel/revert capability (story 15) by applying it in bulk with type-based filtering.
