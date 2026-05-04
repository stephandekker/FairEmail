# Formatting Toolbar Shell

## Parent Feature
#4.1 Rich Text Editor

## User Story
As a casual composer, I want a formatting toolbar displayed above or below the compose area with icon buttons for formatting actions, so that I can discover and access formatting without knowledge of HTML or keyboard shortcuts.

## Blocked by
`1-wysiwyg-editor-surface`

## Acceptance Criteria
- A horizontal formatting toolbar is displayed when the editor is in rich text mode.
- The toolbar is horizontally scrollable to accommodate all buttons without truncation.
- The toolbar can be hidden and shown via a user action (e.g. a toggle in the compose menu).
- A user preference controls whether the toolbar is fixed in position or scrolls with the message content.
- All toolbar buttons have descriptive labels readable by screen readers.
- Toolbar buttons use universally recognized iconography (B for bold, I for italic, etc.) with tooltips on hover.
- The toolbar is navigable via keyboard.

## Mapping to Epic
- FR-4, FR-5, FR-6, FR-7
- NFR-5, NFR-6
- N-8

## HITL / AFK
HITL — toolbar icon selection and layout deserve a brief design review to ensure discoverability and accessibility.

## Notes
- This story creates the toolbar *container* with its scrollable, hideable, and fixed/floating behavior. Individual formatting buttons are wired up in subsequent stories; this story may include placeholder/disabled buttons for layout purposes.
- The epic specifies a single scrollable toolbar rather than context-sensitive ribbon tabs (N-8). This is a deliberate simplicity choice for email.
- OQ-5 (toolbar placement) is relevant: on desktop, fixed placement may be preferable. Record a design decision here or defer to preference.
