# Paragraph Alignment

## Parent Feature
#4.1 Rich Text Editor

## User Story
As a visual communicator, I want to set paragraph alignment to left, center, or right, so that I can lay out headings, signatures, and callouts.

## Blocked by
`2-formatting-toolbar-shell`

## Acceptance Criteria
- Toolbar buttons or a selector for Left, Center, and Right alignment are present and functional.
- Alignment applies to complete paragraphs, even if only part of a paragraph is selected.
- If the selection spans multiple paragraphs, all affected paragraphs are aligned.
- The alignment change is visible immediately in the WYSIWYG surface.
- Alignment persists after saving as a draft and reopening.

## Mapping to Epic
- US-12
- FR-21, FR-22
- AC-7
- N-2

## HITL / AFK
AFK — standard paragraph alignment with well-defined scope rules.

## Notes
- N-2 explains that paragraph-level actions always expand to full paragraph boundaries, regardless of selection. This prevents visually incoherent partial-paragraph formatting.
