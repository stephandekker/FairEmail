# Force-Light Message Content Viewer

## Parent Feature
#9.1 Light / Dark / Black Themes

## User Story
As a content-reader, I want an option to force the original message view to always use a light background regardless of the application's overall theme, so that HTML messages and images render as their authors intended.

## Blocked by
- `3-dark-variant-chrome`

## Acceptance Criteria
- A "force light for original messages" option is available in theme settings (FR-15).
- When enabled and the application is in a dark or black theme, the message content area renders with a light background while the rest of the application remains dark (AC-4).
- When no content override is active, the message content area follows the application's current theme (US-13).
- The preference is persisted across restarts (FR-27).

## Mapping to Epic
- US-11, US-13
- FR-15
- AC-4

## HITL / AFK
AFK — no human review needed beyond normal code review.

## Notes
- Design Note N-3 explains why this exists: many HTML emails contain hard-coded light-background assumptions that break in dark mode. This is an escape valve for readability.
