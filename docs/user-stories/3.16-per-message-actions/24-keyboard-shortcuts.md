## Parent Feature

#3.16 Per-Message Actions

## What to build

Keyboard shortcuts for the most common per-message actions: at minimum, archive, delete/trash, toggle flag/star, and toggle selection. Shortcuts must be discoverable (e.g. shown in context menu items or a help overlay). All action surfaces must be reachable via keyboard for accessibility (FR-71 keyboard portion, NFR-6).

## Acceptance criteria

- [ ] Pressing the archive shortcut archives the focused/selected message(s)
- [ ] Pressing the delete/trash shortcut deletes the focused/selected message(s)
- [ ] Pressing the flag shortcut toggles the flag on the focused/selected message(s)
- [ ] Pressing the selection shortcut toggles selection of the focused message
- [ ] Shortcuts are discoverable (shown in menus or a help overlay)
- [ ] All toolbar and context menu actions are reachable via keyboard (NFR-6)
- [ ] Keyboard shortcuts work with screen readers (NFR-6)

## Blocked by

- Blocked by 22-configurable-toolbar-context-menu

## User stories addressed

- US-65 (keyboard shortcuts for common actions)
