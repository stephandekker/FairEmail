# Paste as Quote

## Parent Feature
#4.1 Rich Text Editor

## User Story
As any user, I want a "paste as quote" action that wraps the pasted content in a block quote, so that I can quote external material with one action.

## Blocked by
`22-rich-paste-with-sanitization`, `16-block-quotes`

## Acceptance Criteria
- A "paste as quote" action is available via context menu and/or keyboard shortcut.
- Invoking this action pastes clipboard content wrapped in a block quote (with the same visual style as block quotes created via the toolbar).
- The pasted content within the quote preserves source formatting (subject to the same sanitization as normal rich paste).
- The resulting block quote is editable and splittable like any other block quote.

## Mapping to Epic
- US-32
- FR-51
- AC-17

## HITL / AFK
AFK — combines paste and block quote infrastructure from prior stories.

## Notes
- This story depends on both the paste infrastructure (story 22) and block quote support (story 16).
