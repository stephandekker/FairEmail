# Multi-Window Theme Uniformity

## Parent Feature
#9.1 Light / Dark / Black Themes

## User Story
As any user, I want the active theme to apply uniformly to all application windows (main window, compose window, settings dialogs, pop-out message views), so that the experience is visually consistent.

## Blocked by
- `2-light-variant-chrome`
- `3-dark-variant-chrome`

## Acceptance Criteria
- The active theme applies to all application windows, dialogs, and pop-out views (FR-29).
- Opening a secondary window (compose, pop-out message, settings dialog) while in dark mode shows the secondary window also in dark mode (AC-12).
- If the theme is changed while secondary windows are open, those windows update to the new theme (or are recreated with it).

## Mapping to Epic
- US-18
- FR-29
- AC-12

## HITL / AFK
AFK — no human review needed beyond normal code review.

## Notes
- This story ensures that theme application is not limited to the main window. Any new window spawned by the application must inherit the active theme state.
