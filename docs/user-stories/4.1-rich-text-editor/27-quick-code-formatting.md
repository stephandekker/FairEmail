# Quick-Code Formatting

## Parent Feature
#4.1 Rich Text Editor

## User Story
As a power user, I want a one-tap "code" formatting action that applies a small monospace font to the selected text, so that I can mark inline code without manually choosing font and size separately.

## Blocked by
`6-font-family-selector`, `7-font-size-selector`

## Acceptance Criteria
- A "code" button is present on the toolbar.
- Tapping the button applies small-size monospace formatting to the selected text in a single action.
- The formatting is visually distinct: monospace font at a smaller size than surrounding body text.
- The action is toggleable: applying it to already-code-formatted text removes the code formatting.
- The result persists after saving as a draft and reopening.

## Mapping to Epic
- US-38
- FR-60
- AC-21

## HITL / AFK
AFK — simple composite formatting action.

## Notes
- This story depends on the font family and font size infrastructure being in place, since "code" formatting is a composite of monospace font + small size.
