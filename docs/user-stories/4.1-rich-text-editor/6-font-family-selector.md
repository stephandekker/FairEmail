# Font Family Selector

## Parent Feature
#4.1 Rich Text Editor

## User Story
As a visual communicator, I want to choose a font family for the selected text from a curated list of email-safe fonts, so that I can give my messages a distinct typographic voice.

## Blocked by
`2-formatting-toolbar-shell`

## Acceptance Criteria
- A font family selector is accessible from the toolbar, presenting a curated list of fonts.
- The font list includes at least: a default proportional font, a monospace font, a serif font, and a sans-serif font.
- Optionally, bundled specialty fonts (e.g. a dyslexia-friendly font) are included.
- A user preference controls whether bundled fonts and narrow font variants are shown in the list.
- Selecting a font changes the selected text's typeface immediately in the WYSIWYG surface.
- The font change persists after saving as a draft and reopening.
- The font is rendered using a fallback stack so the recipient sees a reasonable alternative if the exact font is unavailable.

## Mapping to Epic
- US-6
- FR-12, FR-13, FR-14
- AC-4
- OQ-2

## HITL / AFK
HITL — the curated font list and fallback stack should be reviewed to ensure email compatibility.

## Notes
- OQ-2 (font list governance) is relevant: should users be able to add custom fonts, or is the curated list fixed? What fallback stack is used? This needs a design decision.
