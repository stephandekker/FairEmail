# Numbered (Ordered) Lists with Numbering Styles

## Parent Feature
#4.1 Rich Text Editor

## User Story
As any user, I want to convert selected paragraphs into a numbered (ordered) list and choose from multiple numbering styles (decimal, lower-alpha, upper-alpha, lower-roman, upper-roman), so that I can create sequenced instructions or outlines.

## Blocked by
`11-bullet-lists`

## Acceptance Criteria
- A toolbar button for numbered lists is present and functional.
- Selecting paragraphs and tapping the numbered list button converts them into an ordered list.
- The user can select a numbering style: decimal, lower-alpha, upper-alpha, lower-roman, upper-roman.
- Inserting or removing list items automatically renumbers subsequent items.
- Pressing Enter at the end of a list item creates a new numbered item; pressing Enter on an empty item exits the list.
- A numbered list using "lower-roman" style displays items as i, ii, iii, iv.
- List formatting persists after saving as a draft and reopening.

## Mapping to Epic
- US-14, US-15
- FR-23, FR-24, FR-28
- AC-9, AC-26
- OQ-7

## HITL / AFK
HITL — the UI for selecting numbering styles (sub-menu, cycling button, or dialog) needs a design decision per OQ-7.

## Notes
- OQ-7 asks how the user selects between numbering styles. This needs a design decision before implementation.
- This story builds on the list infrastructure established in story 11 (bullet lists).
