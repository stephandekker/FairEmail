# Paste as Plain Text

## Parent Feature
#4.1 Rich Text Editor

## User Story
As any user, I want a "paste as plain text" action accessible from a context menu or keyboard shortcut that inserts clipboard content with all formatting stripped, so that I can paste clean text from messy sources.

## Blocked by
`22-rich-paste-with-sanitization`

## Acceptance Criteria
- A "paste as plain text" action is available via context menu and/or keyboard shortcut.
- Invoking this action inserts clipboard content with all formatting stripped, regardless of source richness.
- Only the text content is inserted — no styles, no links, no structural markup.
- The action works in both rich text mode and plain text mode (in plain text mode, standard paste already behaves this way).

## Mapping to Epic
- US-31
- FR-50
- AC-16 (plain text paste portion)

## HITL / AFK
AFK — straightforward paste variant.

## Notes
*(none)*
