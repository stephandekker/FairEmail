# Auto Mode Dark Variant Preference

## Parent Feature
#9.1 Light / Dark / Black Themes

## User Story
As a day/night automatic user, I want to control whether the dark side of auto mode uses standard dark or true-black, so that auto-switching respects my OLED preference.

## Blocked by
- `9-system-color-scheme-detection`
- `4-true-black-variant-chrome`

## Acceptance Criteria
- When in auto mode, the user can specify whether the dark side uses the standard dark variant or the true-black variant (FR-12).
- With "true-black for dark" enabled, auto-switching to dark mode uses pure-black backgrounds (AC-3).
- With "true-black for dark" disabled, auto-switching to dark mode uses dark-grey backgrounds.
- This preference is persisted alongside other theme preferences (FR-27).

## Mapping to Epic
- US-7, US-10
- FR-12
- AC-3

## HITL / AFK
AFK — no human review needed beyond normal code review.

## Notes
- Design Note N-2 clarifies that auto mode delegates only brightness, not scheme. The color scheme remains unchanged across the light/dark switch.
