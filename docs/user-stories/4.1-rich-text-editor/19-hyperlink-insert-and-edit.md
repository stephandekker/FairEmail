# Hyperlink Insert and Edit

## Parent Feature
#4.1 Rich Text Editor

## User Story
As any user, I want to insert or edit a hyperlink on selected text by specifying a URL, so that I can link to resources without exposing raw URLs.

## Blocked by
`2-formatting-toolbar-shell`

## Acceptance Criteria
- A toolbar button for inserting/editing a hyperlink is present and functional.
- Selecting text and invoking the link action opens a dialog to enter a URL and optionally a display title.
- On confirmation, the selected text becomes a clickable hyperlink in the WYSIWYG surface.
- Links are visually indicated in the editing surface (e.g. underline + color).
- Selecting text that is already a hyperlink and invoking the link action shows the existing URL for editing (not creating a nested link).
- The user can remove the link from the dialog (unlinking the text while preserving the text itself).
- Links persist after saving as a draft and reopening.

## Mapping to Epic
- US-21, US-22
- FR-37, FR-38, FR-39
- AC-12

## HITL / AFK
AFK — standard hyperlink editing behavior.

## Notes
*(none)*
