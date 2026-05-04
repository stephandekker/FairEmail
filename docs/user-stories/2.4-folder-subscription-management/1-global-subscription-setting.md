# Global Subscription Management Setting

## Parent Feature
#2.4 Folder Subscription Management

## User Story
As a user, I want a global setting in the synchronization preferences that enables or disables subscription management UI controls, so that subscription-related concepts are hidden unless I explicitly opt in.

## Blocked by
_(none — this is the foundation for all subscription-related stories)_

## Acceptance Criteria
- A "subscription management" toggle exists in the synchronization settings area.
- The toggle defaults to **disabled**.
- When disabled, no subscribe/unsubscribe toggle appears in any folder context menu.
- When disabled, no "subscribed only" filter appears in the folder list menu.
- When enabled, subscription controls become available (their specific behaviour is covered by downstream stories).
- The setting persists across application restarts.
- The setting applies application-wide (not per-account).

## Mapping to Epic
- Goals: G6
- User Stories: US-1, US-2
- Functional Requirements: FR-1, FR-2, FR-3
- Acceptance Criteria: AC-1, AC-2 (partial — the "appears" half)

## HITL / AFK
**AFK** — no human review needed during implementation beyond normal code review. The setting is non-destructive and defaults to off.

## Estimation
Small — one setting, one persistence check, conditional visibility of downstream controls.

## Notes
- The epic does not prescribe the exact label for this setting (see OQ-7). The implementer should choose a clear label such as "Manage IMAP subscriptions" and note the decision for UX review.
