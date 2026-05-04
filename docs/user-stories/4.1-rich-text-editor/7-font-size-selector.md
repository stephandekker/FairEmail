# Font Size Selector

## Parent Feature
#4.1 Rich Text Editor

## User Story
As a visual communicator, I want to choose a text size from a set of predefined sizes (extra-small, small, medium/default, large, extra-large), so that I can create visual hierarchy without guessing pixel values.

## Blocked by
`2-formatting-toolbar-shell`

## Acceptance Criteria
- A font size selector is accessible from the toolbar, presenting at least five distinct sizes.
- The sizes are labelled in human-readable terms (e.g. extra-small, small, medium, large, extra-large).
- Selecting a size visually changes the selected text in the WYSIWYG surface immediately.
- Sizes are expressed as relative proportions, not absolute pixel values, so they scale with the recipient's display settings.
- The size change persists after saving as a draft and reopening.

## Mapping to Epic
- US-7
- FR-15, FR-16
- AC-5
- N-5

## HITL / AFK
AFK — straightforward picker with well-defined relative sizing.

## Notes
- N-5 explains why relative sizing is used over absolute: it ensures messages respect the recipient's base font size preference and scale across display densities.
