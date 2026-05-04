# Replay Single Operation on Reconnect

## Parent Feature
#7.5 Offline Storage and Operations

## User Story
As a commuter, when my device reconnects to the network, I want all queued actions to be replayed against the server automatically without any manual intervention, so that my mailbox reaches the intended state without me thinking about it.

## Acceptance Criteria
- When network connectivity becomes available, the system automatically begins processing the operation queue without user intervention.
- A pending operation (e.g. mark-as-read) is executed against the mail server using the appropriate protocol command.
- A successfully executed operation is removed from the queue.
- No explicit "go online" or "go offline" user action is required — the transition is automatic and transparent.
- After performing actions offline and then restoring connectivity, operations are replayed to the server and the queue becomes empty.

## Complexity
Large

## Blocked by
1-persist-operation-record

## HITL/AFK
AFK

## Notes
- This story covers the simplest case: one operation, replayed sequentially. Priority ordering, batching, and parallel account processing are layered in subsequent stories.
- The connectivity detection mechanism is not prescribed by the epic (NG1 says scheduling belongs to features 7.1/7.2), but this story needs *some* trigger to know when to start replay. The minimal requirement is: "when a connection is available, try to process the queue."
- NFR-5 (idempotency) applies here: if the server already applied the operation (e.g. due to a lost ACK), replaying it must not produce duplicate effects.
