# External Surface for Unified Inbox

## Parent Feature

#3.1 Unified Inbox

## What to build

Provide at least one external application surface — system-tray indicator, panel applet, keyboard shortcut, desktop shortcut, or autostart target — that exposes the Unified Inbox without requiring the main window to be open (FR-38, US-32). The external surface must offer at minimum: a filter to "unseen only", a toggle for showing account name/color per message, and an indicator of total unseen count (FR-39, US-33). Updates must be reflected within one minute of a change (AC-18).

## Acceptance criteria

- [ ] At least one external surface exposes Unified Inbox content (AC-18).
- [ ] The surface offers a filter to unseen-only messages.
- [ ] The surface shows account name/color per message (toggleable).
- [ ] The surface shows total unseen count.
- [ ] Updates are reflected within one minute of a change (AC-18).
- [ ] The surface is functional without the main window open.

## Blocked by

- Blocked by `4-basic-unified-message-list`
- Blocked by `13-sort-and-filter`

## User stories addressed

- US-32 (at least one external surface)
- US-33 (external surface has equivalent options)

## Notes

Open question OQ-4 in the epic notes that the full inventory of external surfaces (tray vs. panel applet vs. notification daemon vs. keyboard shortcut vs. autostart) is to be decided as part of the desktop integration epic. This story requires only one surface. OQ-6 asks whether snoozed messages should be visible on external surfaces; defer to the desktop integration epic for that decision.
