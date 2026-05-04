# Clear Formatting (Selection and All)

## Parent Feature
#4.1 Rich Text Editor

## User Story
As any user, I want a "clear formatting" action that removes all character and paragraph styles from the current selection, and a "clear all formatting" action that strips all styles from the entire message, so that I can reset badly-formatted regions or the whole message.

## Blocked by
`3-bold-italic-underline-strikethrough`

## Acceptance Criteria
- A "clear formatting" action is accessible from the toolbar.
- Invoking "clear formatting" on a selection removes all character styles (bold, italic, underline, strikethrough, subscript, superscript, font, size, color, highlight) and paragraph styles (alignment, indentation) from the selected range, restoring it to default appearance.
- A "clear all formatting" action strips all styles from the entire message body.
- Both clear actions preserve hyperlinks, embedded images, and other structural/non-style content.
- The clear actions produce visible results immediately in the WYSIWYG surface.

## Mapping to Epic
- US-24, US-25, US-26
- FR-42, FR-43, FR-44
- AC-13, AC-14

## HITL / AFK
AFK — well-defined behavior with clear preservation rules.

## Notes
- The preservation of hyperlinks and images during clear formatting is explicitly required by FR-44. This means clear formatting strips *styles* but not *structural content*.
