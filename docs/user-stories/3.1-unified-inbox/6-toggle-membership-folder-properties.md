# Toggle Membership via Folder Properties

## Parent Feature

#3.1 Unified Inbox

## What to build

Expose unified-inbox membership as a setting on the folder properties / edit screen, so the user has a discoverable place to find this setting alongside the folder's other properties (FR-10, US-7). The behavior is identical to the context-action toggle: the change is immediate, persistent, and independent of notification/sync settings.

## Acceptance criteria

- [ ] The folder properties / edit screen includes a toggle for unified-inbox membership.
- [ ] Changing the toggle from this screen takes effect immediately in the Unified Inbox view.
- [ ] The toggle state shown matches the current membership state (consistent with context-action toggle).
- [ ] The toggle is reachable via keyboard with a screen-reader label (NFR-7).

## Blocked by

- Blocked by `1-folder-membership-state`
- Blocked by `5-toggle-membership-context-action` (shares the same underlying toggle logic)

## User stories addressed

- US-7 (toggle via folder properties screen)
