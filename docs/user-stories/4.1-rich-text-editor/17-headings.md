# Headings

## Parent Feature
#4.1 Rich Text Editor

## User Story
As a visual communicator, I want to apply heading styles (at least two distinct levels) to a paragraph, so that I can create document-like structure within a longer message.

## Blocked by
`2-formatting-toolbar-shell`

## Acceptance Criteria
- A heading selector is accessible from the toolbar, offering at least two distinct heading levels plus a "normal text" option.
- Heading styles apply to complete paragraphs, even if only part is selected.
- Heading levels are visually distinct from body text in size and weight.
- Headings are visually distinct from each other (level 1 is larger/bolder than level 2).
- Heading formatting persists after saving as a draft and reopening.

## Mapping to Epic
- US-23
- FR-35, FR-36
- OQ-1

## HITL / AFK
HITL — OQ-1 (how many heading levels, and whether they map to HTML `<h1>`–`<h6>` or use size/weight combinations) needs a design decision.

## Notes
- OQ-1 flags that the exact number of heading levels and their HTML mapping is ambiguous. Design should clarify how many levels are needed for email use cases before implementation.
