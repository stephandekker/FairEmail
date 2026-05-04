# Background Highlight Color and Mark Action

## Parent Feature
#4.1 Rich Text Editor

## User Story
As a visual communicator, I want to pick a background highlight color for selected text and have a one-tap "mark" button that applies a standard yellow highlight with black text, so that I can simulate a highlighter pen effect quickly.

## Blocked by
`2-formatting-toolbar-shell`

## Acceptance Criteria
- A background color picker is accessible from the toolbar.
- The picker offers a visual color palette or wheel, a text input for hex color values, and a reset button to remove the applied highlight.
- Selecting a background color immediately highlights the selected text in the WYSIWYG surface.
- A one-tap "mark" button applies a fixed yellow background with black foreground text, independent of the color pickers.
- The "mark" action toggles: applying it to already-marked text removes the mark.
- Highlight and mark changes persist after saving as a draft and reopening.

## Mapping to Epic
- US-9, US-10, US-11
- FR-18, FR-19 (background portion), FR-20
- AC-6 (highlight and mark portions)

## HITL / AFK
AFK — well-defined behavior with clear acceptance criteria.

## Notes
- The foreground color picker is a separate story (`8-foreground-text-color`). The two pickers may share UI components but are distinct toolbar actions.
