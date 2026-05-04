# Auto-Expand Character Styles to Word Boundaries

## Parent Feature
#4.1 Rich Text Editor

## User Story
As any user, when I tap a character-style button with no text selected, I want the formatting to apply to the entire word under the cursor (auto-expanding to word boundaries), so that I do not have to manually select a single word before formatting it.

## Blocked by
`3-bold-italic-underline-strikethrough`

## Acceptance Criteria
- With the cursor inside a word and no text selected, tapping Bold makes the entire word bold.
- The same auto-expand behavior applies to all character styles: italic, underline, strikethrough, subscript, superscript.
- Word boundaries are determined by whitespace and punctuation (standard word-boundary rules).
- If the cursor is on whitespace (not inside a word), the action has no visible effect (no invisible zero-width style insertion).
- Toggle behavior still applies: if the word is already bold, tapping Bold with cursor inside it removes bold.

## Mapping to Epic
- US-4
- FR-10
- AC-2
- N-1

## HITL / AFK
AFK — well-defined behavior described in epic design note N-1.

## Notes
- N-1 explains the rationale: auto-expand prevents the confusing state where formatting is "on" but invisible, and matches the most common intent of styling an entire word.
