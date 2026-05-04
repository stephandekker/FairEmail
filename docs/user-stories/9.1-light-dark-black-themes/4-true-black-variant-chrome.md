# True-Black Variant Applied to Application Chrome

## Parent Feature
#9.1 Light / Dark / Black Themes

## User Story
As an OLED power-saver, I want a true-black variant that uses pure-black (#000000) backgrounds instead of dark grey, so that pixels are fully off on OLED displays and battery is conserved.

## Blocked by
- `1-theme-preference-persistence`
- `3-dark-variant-chrome`

## Acceptance Criteria
- When the brightness variant is set to "black", all application chrome surfaces use pure-black (#000000) backgrounds with light foreground text (FR-8).
- The true-black variant is visually obviously distinct from the standard dark variant (backgrounds are fully black, not dark grey) (FR-8).
- No UI element is left unstyled or shows a hard-coded color conflicting with the true-black variant (NFR-3).
- The true-black variant is available as both a fixed manual choice and as the dark side of auto mode (US-10).

## Mapping to Epic
- US-9, US-10
- FR-8, FR-9
- NFR-3
- AC-3 (partial)

## HITL / AFK
AFK — no human review needed beyond normal code review.

## Notes
- The distinction between dark and true-black is not cosmetic — it enables OLED power savings (Design Note N-1). Implementation must ensure backgrounds are actually #000000, not merely very dark grey.
