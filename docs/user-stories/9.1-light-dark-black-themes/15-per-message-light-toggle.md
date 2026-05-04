# Per-Message Force-Light Toggle

## Parent Feature
#9.1 Light / Dark / Black Themes

## User Story
As any user viewing a dark-themed message, I want a per-message toggle (toolbar button or menu item) to switch the content area between light and themed rendering for the currently viewed message, without changing my global preference.

## Blocked by
- `13-force-light-message-viewer`

## Acceptance Criteria
- A per-message override control (e.g. toolbar button or menu item) is available when viewing a message (FR-18).
- Activating the toggle switches the content area rendering between light and themed for the current message only (AC-7).
- The toggle does not change the global "force light for messages" setting (AC-7).
- The override state resets when navigating to a different message (it is not persisted per-message).

## Mapping to Epic
- US-14 (partial)
- FR-18
- AC-7

## HITL / AFK
AFK — no human review needed beyond normal code review.

## Notes
- This gives users an escape hatch for individual messages that look bad in dark mode, without committing to a global override.
