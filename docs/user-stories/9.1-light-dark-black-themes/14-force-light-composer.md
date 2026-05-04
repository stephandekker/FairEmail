# Force-Light Message Composer

## Parent Feature
#9.1 Light / Dark / Black Themes

## User Story
As a content-reader, I want an option to force the message composer to always use a light background, so that I can see how my formatting will appear to recipients on light backgrounds.

## Blocked by
- `3-dark-variant-chrome`

## Acceptance Criteria
- A "force light for composer" option is available in theme settings (FR-16).
- When enabled and the application is in a dark or black theme, the compose editor renders with a light background while the rest of the application remains dark (AC-5).
- When no content override is active, the composer follows the application's current theme (US-13).
- The preference is persisted across restarts (FR-27).

## Mapping to Epic
- US-12, US-13
- FR-16
- AC-5

## HITL / AFK
AFK — no human review needed beyond normal code review.

## Notes
- This is independent from story 13 (force-light viewer). A user may want one but not the other.
