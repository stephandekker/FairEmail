# Light Variant Applied to Application Chrome

## Parent Feature
#9.1 Light / Dark / Black Themes

## User Story
As any user, I want a light brightness variant that uses bright backgrounds with dark foreground text and controls across all application chrome, so that the application is comfortable to use in bright environments.

## Blocked by
- `1-theme-preference-persistence`

## Acceptance Criteria
- When the brightness variant is set to "light", all application chrome surfaces (toolbars, sidebars, message list, folder tree, dialogs, menus, status bar, cards, floating panels) use bright (white or near-white) backgrounds with dark foreground text (FR-6, FR-9).
- No UI element is left unstyled or shows a hard-coded color conflicting with the light variant (NFR-3).
- The light variant is applied based on the persisted preference at startup.
- Interactive elements (buttons, links, toggles) are visually distinguishable from static content (US-20).

## Mapping to Epic
- US-2 (partial — immediate visual application)
- FR-6, FR-9
- NFR-3
- AC-1 (partial)

## HITL / AFK
AFK — no human review needed beyond normal code review.

## Notes
- This story uses a single default color scheme. Multiple color schemes are introduced in story 5. The focus here is on the brightness mechanics (light backgrounds, dark text) applied uniformly.
