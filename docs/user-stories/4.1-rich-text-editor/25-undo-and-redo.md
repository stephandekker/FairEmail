# Undo and Redo

## Parent Feature
#4.1 Rich Text Editor

## User Story
As a power user, I want undo and redo actions available via keyboard shortcuts, so that I can reverse accidental formatting or text changes risk-free.

## Blocked by
`1-wysiwyg-editor-surface`

## Acceptance Criteria
- Undo and redo operations reverse and re-apply text and formatting changes.
- Undo is accessible via Ctrl+Z (or platform equivalent); redo via Ctrl+Shift+Z (or platform equivalent).
- Undo/redo may also be accessible via context menu.
- A user preference controls whether undo/redo is enabled (opt-in).
- When disabled, the keyboard shortcuts have no effect and no undo history is maintained.
- Undo correctly reverses both text changes and formatting changes (e.g. undoing a bold action restores the text to non-bold).

## Mapping to Epic
- US-34, US-35
- FR-54, FR-55, FR-56
- AC-19

## HITL / AFK
AFK — standard undo/redo with well-defined preference toggle.

## Notes
- The epic makes undo/redo optionally enabled via preference (FR-56). This is unusual — most editors enable it by default. The design may want to reconsider whether opt-in is the right default, but the epic is authoritative.
