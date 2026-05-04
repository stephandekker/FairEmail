# Toggle Folder Subscription

## Parent Feature
#2.4 Folder Subscription Management

## User Story
As a subscription-aware user, I want to toggle subscription on or off for any IMAP folder via a context action, so that I can manage which folders I care about and have the change propagated to the server for consistency across all my clients.

## Blocked by
`1-global-subscription-setting` — the subscription toggle is only visible when the global subscription management setting is enabled.

## Acceptance Criteria
- When subscription management is enabled, a subscribe/unsubscribe toggle appears in the context menu of every non-read-only IMAP folder.
- The toggle is a checkable item that reflects the folder's current subscription state (checked = subscribed, unchecked = unsubscribed).
- Toggling subscription queues a subscription operation that propagates the new state to the IMAP server.
- The toggle does **not** appear on read-only folders (the server would reject the change).
- The toggle does **not** appear when the global subscription management setting is disabled.
- Every IMAP folder has a persistent **subscribed** property that may be null (unknown), true, or false.
- Changing subscription does not alter the folder's synchronization setting, unified-inbox membership, notification setting, or any other property.
- The toggle appears to complete locally within one second; server propagation happens asynchronously during the next sync cycle.

## Mapping to Epic
- Goals: G1
- User Stories: US-3, US-4, US-6
- Functional Requirements: FR-4, FR-5, FR-6, FR-37
- Acceptance Criteria: AC-2, AC-3
- Non-Functional Requirements: NFR-1, NFR-5

## HITL / AFK
**AFK** — straightforward toggle with well-defined behaviour.

## Estimation
Small-to-medium — involves a persistent property, a context menu toggle gated by the global setting, and queued server propagation.
