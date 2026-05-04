# Auto Mode Runtime Switching

## Parent Feature
#9.1 Light / Dark / Black Themes

## User Story
As a day/night automatic user, when my system switches from light to dark (or vice versa) while the application is running, I want the application to transition automatically within a few seconds without requiring a restart, so that the experience is seamless.

## Blocked by
- `9-system-color-scheme-detection`
- `8-immediate-theme-application`

## Acceptance Criteria
- In auto mode, the application monitors for system color-scheme changes at runtime (FR-11).
- When the system preference changes, the application applies the corresponding brightness variant without user action or application restart (FR-11).
- The transition occurs within five seconds of the system change (AC-2).
- The auto-switch does not flicker or oscillate if the system signal changes rapidly; rapid changes are debounced or coalesced (NFR-4).
- Navigation state is preserved across automatic switches.

## Mapping to Epic
- US-6
- FR-11
- NFR-4
- AC-2

## HITL / AFK
AFK — no human review needed beyond normal code review.

## Notes
- Open Question OQ-7 asks about debounce window duration. A reasonable default might be 500ms–1s to avoid flicker during GNOME transition animations while still feeling responsive.
