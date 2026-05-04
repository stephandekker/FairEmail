# Auto-Save on Punctuation

## Parent Feature
#4.8 Auto-Save Drafts

## User Story
As a careful composer writing long sentences, I want the option to auto-save whenever I type sentence-ending punctuation (period, question mark, exclamation mark, colon, semicolon, comma, or full-width period), so that saves happen more frequently than paragraph breaks alone.

## Blocked by
1-local-draft-persistence

## Acceptance Criteria
- When punctuation auto-save is enabled and the user types an end character (`.`, `。`, `:`, `;`, `?`, `!`, `,`), and the preceding character was **not** also an end character, an auto-save is triggered (provided the draft is dirty).
- Typing multiple consecutive end characters (e.g. `...` or `?!`) does **not** trigger multiple redundant saves — only the first end character after a non-end character fires. (AC-4)
- The save is silent: no spinner, no dialog, no perceptible input lag. (AC-2, AC-17)
- This trigger is **disabled by default**. (US-5)

## Mapping to Epic
- FR-2 (end-character trigger with deduplication guard)
- FR-4 (dirty state guard)
- FR-6 (silent save)
- US-4, US-5, US-6
- AC-2, AC-4, AC-17

## HITL / AFK
AFK — same pattern as paragraph-break trigger with a different character set.

## Estimation
Small — mirrors the paragraph-break trigger with a broader character set; shares the same dedup pattern.
