# Keep-Selection Mode

## Parent Feature
#4.1 Rich Text Editor

## User Story
As a power user, I want a "keep selection" preference that preserves my text selection after applying a formatting action, so that I can apply multiple styles in rapid succession without re-selecting.

## Blocked by
`3-bold-italic-underline-strikethrough`

## Acceptance Criteria
- A "keep selection" preference is available, defaulting to disabled.
- When enabled, the user's text selection is preserved after applying any formatting action (bold, italic, color, etc.).
- When disabled (default), the cursor moves to the end of the formatted range after applying a style, matching standard word-processor behavior.
- The "keep selection" toggle is accessible from the formatting toolbar or its overflow menu for quick toggling.

## Mapping to Epic
- US-36, US-37
- FR-57, FR-58, FR-59
- AC-20
- N-4

## HITL / AFK
AFK — well-defined preference with clear default behavior.

## Notes
- N-4 explains the rationale: the default (cursor moves to end) matches mainstream editors. "Keep selection" is opt-in for power users who apply multiple styles in sequence.
