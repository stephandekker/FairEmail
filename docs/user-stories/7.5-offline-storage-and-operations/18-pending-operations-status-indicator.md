# Pending Operations Status Indicator

## Parent Feature
#7.5 Offline Storage and Operations

## User Story
As any user, I want to see a persistent, non-intrusive status indicator showing the count of pending and failed operations, so that I always know whether actions are waiting to be committed and whether anything has gone wrong.

## Acceptance Criteria
- A status indicator is visible from the main application window showing the count of pending operations.
- The indicator also shows the count of failed operations (distinguishable from pending).
- The indicator updates in real-time as operations are created, executed, or fail.
- The indicator is non-intrusive — it does not block workflow or require dismissal.
- Clicking/activating the indicator navigates to the operations queue view.
- When all operations are complete and none have failed, the indicator shows a "clear" state (zero or hidden).

## Complexity
Small

## Blocked by
1-persist-operation-record
14-operations-queue-view

## HITL/AFK
AFK

## Notes
- NFR-6 (transparency) requires that the user can always determine: (a) whether they are currently online or offline, (b) how many operations are pending, and (c) whether any operations have failed. This story covers (b) and (c). Online/offline indication may be part of this indicator or a separate one.
