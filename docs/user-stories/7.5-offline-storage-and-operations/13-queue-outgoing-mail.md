# Queue Outgoing Mail While Offline

## Parent Feature
#7.5 Offline Storage and Operations

## User Story
As any user, when I compose and send a message while offline, I want the message to be accepted into an outgoing queue and delivered automatically when connectivity returns, so that I can write mail at any time.

## Acceptance Criteria
- Composing and pressing "send" while offline accepts the message into the operation queue (as a send operation).
- The message is delivered automatically when connectivity returns without user intervention.
- The queued message is visible in the operations view as a pending send operation.
- The user receives no error or blocking state when sending while offline — the action is accepted instantly.
- Queued outgoing messages survive application restart and system reboot.

## Complexity
Medium

## Blocked by
1-persist-operation-record
2-optimistic-ui-update
3-replay-single-operation

## HITL/AFK
AFK

## Notes
- This story is specifically about the "send" operation type and its offline queuing semantics. The ability to *cancel* a queued send is covered in story 15.
- OQ-8 asks whether there should be an explicit undo-send grace period beyond "cancellable while queued." For this story, the behavior is simply: the message is queued and will be sent on reconnect. Cancellation is a separate story.
