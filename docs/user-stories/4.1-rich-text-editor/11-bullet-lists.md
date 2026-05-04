# Bullet (Unordered) Lists

## Parent Feature
#4.1 Rich Text Editor

## User Story
As any user, I want to convert selected paragraphs into a bullet (unordered) list via a toolbar button, so that I can present items as a scannable list.

## Blocked by
`2-formatting-toolbar-shell`

## Acceptance Criteria
- A toolbar button for bullet lists is present and functional.
- Selecting one or more paragraphs and tapping the bullet list button converts them into an unordered list with one list item per paragraph.
- Pressing Enter at the end of a list item creates a new list item at the same level.
- Pressing Enter on an empty list item exits the list and returns to normal paragraph mode.
- The list is visible immediately in the WYSIWYG surface with bullet markers.
- List formatting persists after saving as a draft and reopening.

## Mapping to Epic
- US-13, US-15
- FR-23, FR-26, FR-27
- AC-8

## HITL / AFK
AFK — standard list creation behavior.

## Notes
- Numbered lists with multiple numbering styles are a separate story (`12-numbered-lists`).
- List nesting (indent/outdent) is a separate story (`14-list-nesting`).
