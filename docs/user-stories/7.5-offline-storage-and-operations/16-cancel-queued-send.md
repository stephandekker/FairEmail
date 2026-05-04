# Cancel Queued Send (Undo Send)

## Parent Feature
#7.5 Offline Storage and Operations

## User Story
As any user, I want to cancel a queued outgoing message while it is still pending in the queue, recovering it as a draft, so that I can fix mistakes before the message is actually sent.

## Acceptance Criteria
- A queued send operation can be cancelled while it is still pending (not yet executed).
- Deleting a queued send operation recovers the message as a draft in the user's drafts folder.
- The recovered draft is editable and can be re-sent.
- The operation disappears from the queue after cancellation.
- If the send operation has already been executed (message delivered), cancellation is not possible and the UI communicates this clearly.

## Complexity
Small

## Blocked by
13-queue-outgoing-mail
15-cancel-operation-with-revert

## HITL/AFK
AFK

## Notes
- OQ-8 in the epic asks about an explicit configurable undo-send grace period vs. "cancellable while queued." This story implements the simpler "cancellable while queued" behavior. A timed grace period (if desired) would be a separate enhancement and may overlap with feature 4.9's send delay.
