# Dark Variant Applied to Application Chrome

## Parent Feature
#9.1 Light / Dark / Black Themes

## User Story
As any user, I want a dark brightness variant that uses dim grey backgrounds with light foreground text and controls across all application chrome, so that the application is comfortable to use in dim environments and reduces eye strain.

## Blocked by
- `1-theme-preference-persistence`
- `2-light-variant-chrome`

## Acceptance Criteria
- When the brightness variant is set to "dark", all application chrome surfaces use dim grey backgrounds (not pure black) with light foreground text (FR-7, FR-9).
- The dark variant is visually distinct from both the light variant and the true-black variant.
- No UI element is left unstyled or shows a hard-coded color conflicting with the dark variant (NFR-3).
- Interactive elements remain visually distinguishable in dark mode (US-20).

## Mapping to Epic
- US-2 (partial)
- FR-7, FR-9
- NFR-3
- AC-1 (partial)

## HITL / AFK
AFK — no human review needed beyond normal code review.

## Notes
- The epic specifies dark grey backgrounds (~#121316 per Design Note N-1), distinct from true-black (#000000). This story establishes the dark-grey variant; true-black follows in story 4.
