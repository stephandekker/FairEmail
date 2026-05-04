# Foreground Text Color Picker

## Parent Feature
#4.1 Rich Text Editor

## User Story
As a visual communicator, I want to pick a foreground text color from a color chooser, so that I can color-code or emphasize specific words.

## Blocked by
`2-formatting-toolbar-shell`

## Acceptance Criteria
- A foreground color picker is accessible from the toolbar.
- The picker offers a visual color palette or wheel, a text input for hex color values, and a reset button to remove the applied color (returning to default).
- Selecting a color immediately changes the selected text's color in the WYSIWYG surface.
- The color change persists after saving as a draft and reopening.

## Mapping to Epic
- US-8, US-11
- FR-17, FR-19 (foreground portion)
- AC-6 (foreground portion)
- OQ-4

## HITL / AFK
AFK — standard color picker behavior.

## Notes
- OQ-4 (default text color preference) is relevant: should there be a user-configured default compose text color, and should it be global or per-account? This needs a design decision.
- Background highlight color is a separate story (`9-background-highlight-and-mark`).
