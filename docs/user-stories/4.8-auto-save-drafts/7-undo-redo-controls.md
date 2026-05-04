# Undo / Redo Controls for Revision History

## Parent Feature
#4.8 Auto-Save Drafts

## User Story
As any user, I want visible undo and redo controls in the compose window that step backward and forward through the revision history, so that I can navigate between saved snapshots without guessing.

## Blocked by
6-revision-history-storage

## Acceptance Criteria
- The compose window provides an undo control that loads the previous revision (current revision minus one) and replaces the draft body with that revision's content. (FR-13)
- The compose window provides a redo control that loads the next revision (current revision plus one) and replaces the draft body with that revision's content. (FR-14)
- The undo control is visible only when the current revision is greater than 1. (FR-15, AC-7)
- The redo control is visible only when the current revision is less than the total number of revisions. (FR-16, AC-7)
- Navigating via undo or redo immediately updates the revision indicator and the visibility of undo/redo controls. (FR-17)
- After three edits separated by paragraph breaks, undo steps back through all three saved states and redo steps forward again. (AC-6)
- After undo or redo, the displayed draft body exactly matches the content saved at that revision — no partial state, no data corruption. (NFR-6)
- When revision history is disabled, undo and redo controls are hidden. (FR-18, AC-9)
- Undo and redo are reachable via keyboard shortcut as well as pointer interaction, with appropriate labels for screen readers. (NFR-7)

## Mapping to Epic
- FR-13 through FR-18
- NFR-6 (consistency), NFR-7 (accessibility)
- US-10, US-11, US-12, US-15
- AC-6, AC-7, AC-9

## HITL / AFK
AFK — UI controls with well-specified visibility rules and clear acceptance criteria.

## Notes
- **OQ-6 (Character-level undo interaction):** The epic notes a potential UX confusion between character-level undo (part of the rich text editor, epic 4.1) and revision-level undo (this story). The epic flags this as an open question. This story implements revision-level undo/redo only; interaction with character-level undo should be resolved at design time.
- These undo/redo controls are distinct from standard text-editor undo (Ctrl+Z). The epic's keyboard shortcuts for revision undo/redo should be chosen to avoid conflict with character-level undo.

## Estimation
Medium — UI controls with conditional visibility, keyboard shortcuts, body replacement, and accessibility labelling.
