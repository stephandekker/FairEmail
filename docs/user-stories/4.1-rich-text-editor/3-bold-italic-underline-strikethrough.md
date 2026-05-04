# Bold, Italic, Underline, and Strikethrough

## Parent Feature
#4.1 Rich Text Editor

## User Story
As a casual composer, I want to select text and tap Bold, Italic, Underline, or Strikethrough buttons on the toolbar to apply or remove those styles, so that I can add emphasis to my messages without knowing HTML.

## Blocked by
`1-wysiwyg-editor-surface`, `2-formatting-toolbar-shell`

## Acceptance Criteria
- Toolbar buttons for Bold, Italic, Underline, and Strikethrough are present and functional.
- Selecting text and tapping Bold makes the selection bold; tapping Bold again removes bold (toggle behavior).
- Same toggle behavior applies to Italic, Underline, and Strikethrough independently.
- Multiple character styles can be combined on the same text run (e.g. bold + italic + underline simultaneously).
- The formatting change is visible immediately in the WYSIWYG surface (within 100ms).
- The styles are preserved when the message is saved as a draft and reopened.

## Mapping to Epic
- US-1, US-2, US-5
- FR-8, FR-9, FR-11
- AC-1
- NFR-1

## HITL / AFK
AFK — standard formatting behavior with well-defined toggle semantics.

## Notes
- Auto-expand to word boundaries when no text is selected is a separate story (`4-auto-expand-word-boundaries`).
- Subscript and superscript are also separate (`5-subscript-superscript`).
