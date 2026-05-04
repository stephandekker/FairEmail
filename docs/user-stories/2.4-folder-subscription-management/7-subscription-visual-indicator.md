# Subscription Visual Indicator

## Parent Feature
#2.4 Folder Subscription Management

## User Story
As a subscription-aware user, I want to see a visual indicator on each folder in the folder list showing whether it is currently subscribed, so that I can tell at a glance which folders are part of my subscription set.

## Blocked by
`6-toggle-folder-subscription` — requires the subscribed property to exist and be togglable.

## Acceptance Criteria
- When subscription management is enabled, a visual indicator (icon or badge) in the folder list distinguishes subscribed folders from unsubscribed ones.
- The indicator updates immediately when the user toggles subscription.
- The indicator updates when subscription state changes during server sync (e.g., changed by another client).
- The indicator is not shown when the global subscription management setting is disabled.
- The indicator is accessible: it has a screen-reader label and obeys the application-wide theme.

## Mapping to Epic
- Goals: G1 (partial — visibility of state)
- User Stories: US-5
- Functional Requirements: FR-7
- Acceptance Criteria: AC-4
- Non-Functional Requirements: NFR-6

## HITL / AFK
**HITL** — the specific visual treatment (icon choice, placement, theme integration) needs UX/design review.

## Estimation
Small — a single visual element driven by an existing property.
