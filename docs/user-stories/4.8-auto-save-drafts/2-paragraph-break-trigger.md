# Auto-Save on Paragraph Break

## Parent Feature
#4.8 Auto-Save Drafts

## User Story
As a fast typist, when I press Enter to start a new paragraph, I want the application to automatically save my draft, so that if the application crashes I lose at most the current paragraph.

## Blocked by
1-local-draft-persistence

## Acceptance Criteria
- When the user inserts a newline character and the preceding character was **not** also a newline, an auto-save is triggered (provided the draft is dirty).
- Typing multiple consecutive newlines (blank lines) does **not** trigger multiple redundant saves — only the first newline after non-newline content fires. (AC-3)
- The save is silent: no spinner, no dialog, no perceptible input lag. (AC-2 analogue, AC-17)
- This trigger is **enabled by default** — no configuration needed for protection. (US-2)
- Killing the application immediately after a paragraph-break save and restarting recovers the draft including the paragraph just completed. (AC-1)

## Mapping to Epic
- FR-1 (newline trigger with deduplication guard)
- FR-4 (dirty state guard)
- FR-6 (silent save)
- US-1, US-2
- AC-1, AC-3, AC-17

## HITL / AFK
AFK — straightforward event-detection logic with well-defined dedup rule.

## Estimation
Small — one event listener, one preceding-character check, delegates to the persistence layer from story 1.
