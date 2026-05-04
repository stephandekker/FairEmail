# Beige Mode for Light Variant

## Parent Feature
#9.1 Light / Dark / Black Themes

## User Story
As a user of the light variant, I want an option to use a warm beige background for message cards and list items instead of pure white, so that the interface is less harsh in bright environments, and I want to be able to disable it for standard white if I prefer.

## Blocked by
- `2-light-variant-chrome`

## Acceptance Criteria
- A beige toggle is available in theme settings (US-15, US-16).
- When the light variant is active and beige is enabled, card backgrounds use a warm off-white tint (FR-22, AC-11).
- When beige is disabled, card backgrounds use standard white (FR-22, AC-11).
- The beige preference is persisted across restarts (FR-27).
- Beige is only relevant in the light variant; the toggle has no effect in dark or black variants.

## Mapping to Epic
- US-15, US-16
- FR-22
- AC-11

## HITL / AFK
AFK — no human review needed beyond normal code review.

## Notes
- Design Note N-6 explains that beige exists because pure-white cards on a white background create a flat, harsh appearance. The warm tint adds depth and reduces eye fatigue.
- The source application has beige on by default. Whether the desktop version follows this default is a design choice.
