# Block Quotes with Split and Nesting

## Parent Feature
#4.1 Rich Text Editor

## User Story
As any user, I want to wrap selected paragraphs in a block quote with a visible left-border style, split a quote by pressing Enter inside it, and nest quotes within quotes, so that I can visually distinguish and interleave quoted material with my own writing.

## Blocked by
`2-formatting-toolbar-shell`

## Acceptance Criteria
- A toolbar button for block quote is present and functional.
- Selecting paragraphs and tapping the block quote button wraps them in a block quote, visually indicated by a colored left border with increased left margin.
- Pressing Enter inside a block quote splits the quote into two, with an unquoted paragraph between them, allowing the user to interleave commentary.
- Block quotes are nestable: a quote inside a quote renders with additional visual nesting.
- Block quote formatting persists after saving as a draft and reopening.

## Mapping to Epic
- US-18, US-19
- FR-32, FR-33, FR-34
- AC-25
- OQ-6

## HITL / AFK
AFK — well-defined behavior, though OQ-6 (visual style) may warrant a brief design review.

## Notes
- OQ-6 asks whether the desktop application should match the Android app's block quote visual style (colored left border with configurable color, width, and gap) or adopt platform-native styling. This needs a design decision.
- OQ-8 (interaction with reply quoting) is tangentially relevant: should toolbar actions modify quoted portions of replied messages? This is not addressed in this story but may affect block quote behavior in the reply context.
