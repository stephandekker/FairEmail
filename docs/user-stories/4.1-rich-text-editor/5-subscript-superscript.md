# Subscript and Superscript

## Parent Feature
#4.1 Rich Text Editor

## User Story
As a visual communicator, I want to apply subscript or superscript to selected text via toolbar buttons, so that I can write chemical formulas, footnote markers, or mathematical notation.

## Blocked by
`3-bold-italic-underline-strikethrough`

## Acceptance Criteria
- Toolbar buttons for Subscript and Superscript are present and functional.
- Applying subscript to selected text visually lowers and shrinks it.
- Applying superscript to selected text visually raises and shrinks it.
- Both styles are toggleable: applying to already-styled text removes the style.
- Subscript and superscript can be combined with other character styles (bold, italic, etc.).
- Auto-expand to word boundaries applies when no text is selected (per story 4).

## Mapping to Epic
- US-3
- FR-8 (subscript/superscript portion)
- AC-3

## HITL / AFK
AFK — straightforward extension of the character style infrastructure.

## Notes
- Subscript and superscript are separated from the core four character styles (story 3) because they are less commonly used and may require different rendering treatment.
