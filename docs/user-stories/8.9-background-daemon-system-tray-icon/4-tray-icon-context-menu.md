# Tray Icon Context Menu

## Parent Feature
#8.9 Background Daemon (System Tray Icon)

## User Story
As any user, I want the tray icon to provide a context menu (e.g. via right-click) with basic controls, so that I have quick access to common actions without opening the full UI.

## Blocked by
- `2-system-tray-icon-presence`

## Acceptance Criteria
- Right-clicking (or the platform-appropriate gesture) on the tray icon shows a context menu.
- The context menu contains at minimum:
  - "Open" — opens or brings to focus the main application window.
  - "Quit" — terminates both the main window and the daemon, removes the tray icon, and closes all server connections.
- Menu items are keyboard-accessible and have screen-reader labels.
- The context menu renders correctly on GNOME, KDE Plasma, and XFCE.

## Mapping to Epic
- US-13
- FR-16
- NFR-7 (accessibility)
- AC-4, AC-5

## HITL / AFK
HITL — OQ-3 asks whether additional menu items (e.g. "Synchronize now," "Pause," "Compose") should be included. The minimum spec is "Open" and "Quit," but a design review should confirm whether more items belong in v1.

## Notes
- OQ-3 (context menu richness) is directly relevant. The epic requires only "Open" and "Quit" as mandatory items. Additional items are optional and should be decided during design. This story implements only the mandatory items; additional items can be added in a follow-up story if the design review requests them.
- The "Quit" action wired here implements the quit-vs-close semantics further specified in story 5.
