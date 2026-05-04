# Plain Text Mode Toggle and Preferences

## Parent Feature
#4.1 Rich Text Editor

## User Story
As a plain-text purist, I want to toggle a "plain text only" mode that hides the formatting toolbar and switches to a monospace font, and I want preferences to make this the default, so that I compose in my preferred mode without distraction.

## Blocked by
`1-wysiwyg-editor-surface`, `2-formatting-toolbar-shell`

## Acceptance Criteria
- A toggle for "plain text only" mode is available in the compose window.
- When plain text mode is active: the formatting toolbar is hidden, the editing surface uses a monospace font, and the message will be sent without an HTML part.
- A user preference controls whether plain-text mode is the default for all new compositions.
- A user preference controls whether replies to plain-text-only messages automatically use plain-text mode.
- Switching from rich text to plain text strips all formatting from the message content, with an appropriate warning if the message contains formatting that will be lost.
- Switching from plain text to rich text re-enables the toolbar (previously entered text remains as-is, unformatted).

## Mapping to Epic
- US-27, US-28, US-29
- FR-45, FR-46, FR-47, FR-48
- AC-15
- N-6

## HITL / AFK
AFK — well-defined toggle with clear preference semantics.

## Notes
- N-6 clarifies that the editor's rich text capabilities are purely about composition and HTML output. Whether a plain-text alternative MIME part is also included is a transmission-layer concern outside this epic.
