# Color Scheme Reversal

## Parent Feature
#9.1 Light / Dark / Black Themes

## User Story
As any user, I want an option to reverse the primary and accent colors of my chosen color scheme, so that I can emphasize the accent as the dominant color if I prefer.

## Blocked by
- `5-color-scheme-catalog`

## Acceptance Criteria
- A "reverse" or "swap" option is available for each color scheme (FR-3).
- Activating reversal swaps the roles of the primary and accent colors throughout the application chrome.
- The reversed arrangement produces a visibly different appearance while maintaining WCAG AA contrast compliance (AC-10).
- The reversal preference is persisted as part of the theme configuration (FR-27).

## Mapping to Epic
- US-4
- FR-3
- AC-10

## HITL / AFK
AFK — no human review needed beyond normal code review.

## Notes
- Reversal must not break contrast. If any reversed combination fails WCAG AA, that is a bug in the color scheme definition (story 5) rather than this story.
