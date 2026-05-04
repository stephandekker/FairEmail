# List Editing Behavior (Enter, Exit, Renumber)

## Parent Feature
#4.1 Rich Text Editor

## User Story
As any user, when I press Enter at the end of a list item I want a new item created automatically, and when I press Enter on an empty list item I want the list to end and normal paragraph mode to resume, so that list editing feels natural.

## Blocked by
`11-bullet-lists`, `12-numbered-lists`

## Acceptance Criteria
- Pressing Enter within a list item creates a new list item at the same level.
- Pressing Enter on an empty list item (or a dedicated "exit list" action) terminates the list and returns to normal paragraph mode.
- In ordered lists, inserting a new item automatically numbers it correctly in sequence.
- In ordered lists, removing an item automatically renumbers all subsequent items.
- These behaviors work consistently for both bullet and numbered lists.
- These behaviors work correctly at all nesting levels.

## Mapping to Epic
- US-15
- FR-26, FR-27, FR-28
- AC-8 (enter/exit portion), AC-9

## HITL / AFK
AFK — standard list editing semantics.

## Notes
- Stories 11 and 12 establish basic list creation. This story focuses on the editing *interactions* within lists. Some overlap with AC in stories 11/12 is intentional — this story hardens the edge cases (mid-list insertion, deletion with renumbering, nested exit behavior).
