## Parent Feature

#3.16 Per-Message Actions

## What to build

Provide a configurable toolbar in the message view where each action button can be independently shown or hidden via a configuration dialog. Provide a context menu accessible from every message that contains all per-message actions grouped into logical categories (operations, properties, sharing, advanced) regardless of toolbar configuration. Actions inapplicable in the current context (e.g. IMAP-only actions on POP3 accounts, write-dependent actions on read-only folders, experimental actions when not enabled) are hidden or disabled (FR-71, FR-72, FR-75, FR-79).

## Acceptance criteria

- [ ] Message view displays a toolbar with configurable action buttons
- [ ] A configuration dialog lets the user show/hide each toolbar button (AC-20)
- [ ] Toolbar configuration persists across restarts (NFR-5)
- [ ] Right-clicking a message opens a context menu with all actions in grouped categories
- [ ] Hiding a button from the toolbar does not remove it from the context menu (AC-20)
- [ ] IMAP-only actions are hidden for POP3 accounts
- [ ] Write-dependent actions are hidden for read-only folders
- [ ] Experimental actions are hidden unless experimental features are enabled

## Blocked by

None — can start immediately.

## User stories addressed

- US-62 (configure which buttons appear in toolbar)
- US-63 (all actions accessible via context menu regardless of toolbar config)
