# Non-List Paragraph Indentation

## Parent Feature
#4.1 Rich Text Editor

## User Story
As any user, I want to indent or outdent paragraphs that are not part of a list, so that I can visually offset quoted or secondary content.

## Blocked by
`2-formatting-toolbar-shell`

## Acceptance Criteria
- Toolbar buttons or keyboard shortcuts for paragraph indent and outdent are available.
- Indenting a paragraph increases its left margin offset visually.
- Outdenting a paragraph decreases its left margin offset, down to zero (no negative indent).
- Indentation is independent of list membership — indenting a non-list paragraph does not convert it into a list.
- Indentation persists after saving as a draft and reopening.

## Mapping to Epic
- US-17
- FR-30, FR-31

## HITL / AFK
AFK — straightforward paragraph-level indentation.

## Notes
- List-specific indent/outdent (nesting) is handled in story `14-list-nesting`.
