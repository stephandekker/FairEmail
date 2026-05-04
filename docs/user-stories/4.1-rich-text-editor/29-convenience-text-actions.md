# Convenience Text Actions (Bracket and Quote Wrapping)

## Parent Feature
#4.1 Rich Text Editor

## User Story
As any user, when I have text selected, I want quick actions to wrap the selection in brackets or quotation marks (toggling: if already wrapped, unwrap), so that I can add common punctuation without precise cursor placement.

## Blocked by
`1-wysiwyg-editor-surface`

## Acceptance Criteria
- When text is selected, quick actions are available (via toolbar, context menu, or keyboard shortcut) to wrap/unwrap the selection in parentheses or quotation marks.
- Wrapping adds the punctuation around the selected text (e.g. "hello" becomes "(hello)" or "\"hello\"").
- If the selection is already wrapped in the target punctuation, the action removes the wrapping (toggle behavior).
- The action works in both rich text and plain text modes.

## Mapping to Epic
- US-41
- FR-68

## HITL / AFK
AFK — simple text manipulation with toggle behavior.

## Notes
- The epic specifies "brackets or quotation marks" — the exact set of wrapping pairs (parentheses, square brackets, curly braces, single quotes, double quotes) should be clarified during implementation.
