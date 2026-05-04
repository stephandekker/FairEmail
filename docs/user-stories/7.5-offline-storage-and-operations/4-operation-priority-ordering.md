# Operation Priority and FIFO Ordering

## Parent Feature
#7.5 Offline Storage and Operations

## User Story
As any user, I want operations to be replayed in a deterministic, priority-aware order, so that dependencies between actions are respected and time-sensitive actions (like sending a reply) are not starved by bulk operations.

## Acceptance Criteria
- Each operation is assigned a priority level at creation time based on its type.
- Content retrieval (body, attachment download) executes at highest priority.
- Message creation and deletion execute at high priority.
- State changes (read, flag, keyword) execute at normal priority.
- Outgoing mail (send) executes at normal priority.
- Bulk folder operations (sync, purge, expunge) execute at lower priority.
- Within the same priority level, operations execute in creation order (FIFO).
- Two operations on the same message (e.g. mark read then flag) are both replayed in creation order, and the final server state reflects both.

## Complexity
Small

## Blocked by
3-replay-single-operation

## HITL/AFK
AFK

## Notes
- OQ-4 in the epic raises an open question about whether users might want different ordering in edge cases. For now, implement the fixed priority scheme described in FR-8. This can be revisited later.
- Design Note N-3 explains the rationale: without priorities, bulk low-value operations could starve time-sensitive actions.
