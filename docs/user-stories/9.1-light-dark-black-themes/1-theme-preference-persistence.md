# Theme Preference Data Model and Persistence

## Parent Feature
#9.1 Light / Dark / Black Themes

## User Story
As any user, I want my theme selection (color scheme, brightness variant, reversal option, auto-dark preference, content overrides, beige preference) to persist across application restarts, so that I configure my theme once and it stays.

## Blocked by
*(none — this is the foundational slice)*

## Acceptance Criteria
- The application stores the following theme preferences as local user configuration: selected color scheme, brightness variant (light/dark/black/auto), color reversal toggle, auto-mode dark variant (dark vs. black), force-light-for-messages toggle, force-light-for-composer toggle, beige toggle.
- Preferences persist across application restarts (FR-27).
- No network communication is required to store or retrieve theme preferences (FR-28).
- On first launch (no stored preferences), the application uses a sensible default (light variant, a neutral or flagship color scheme).
- The stored preferences are read at application startup before any UI is rendered.

## Mapping to Epic
- US-17
- FR-27, FR-28
- AC-9

## HITL / AFK
AFK — no human review needed beyond normal code review.

## Notes
- This story establishes the persistence layer only. It does not include applying the theme visually (story 2) or the selection UI (story 7). It ensures that other stories have a place to read/write preference state.
