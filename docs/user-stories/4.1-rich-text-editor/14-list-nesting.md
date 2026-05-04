# List Nesting (Indent/Outdent)

## Parent Feature
#4.1 Rich Text Editor

## User Story
As a power user, I want to increase or decrease the nesting level of a list item (indent/outdent), so that I can create multi-level outlines.

## Blocked by
`11-bullet-lists`, `12-numbered-lists`

## Acceptance Criteria
- Toolbar buttons or keyboard shortcuts for indent and outdent within lists are available.
- Indenting a list item increases its nesting level, visually moving it inward.
- Each nesting level maintains its own numbering sequence (for ordered lists).
- Outdenting a list item at the top level removes it from the list, returning it to a normal paragraph.
- Deeply nested lists (3+ levels) remain stable and render correctly.

## Mapping to Epic
- US-16
- FR-25, FR-29
- AC-10

## HITL / AFK
AFK — standard list nesting behavior.

## Notes
- Non-list paragraph indentation is a separate story (`15-non-list-indentation`).
