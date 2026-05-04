# Toggle Membership via Context Action

## Parent Feature

#3.1 Unified Inbox

## What to build

Allow the user to toggle unified-inbox membership for any single folder via a context action exposed directly from the folder list — e.g. right-click context menu or keyboard shortcut (FR-9, US-4). The toggle must work for any folder regardless of type or role: Inbox, Sent, Drafts, user-created, etc. (US-5). The change must take effect immediately and be reflected in the Unified Inbox view, the navigation pane, and any external surface without requiring a manual refresh or restart (FR-12, NFR-1).

## Acceptance criteria

- [ ] Right-clicking (or equivalent context action) on any folder in the folder list offers a "toggle unified inbox" option.
- [ ] Toggling membership OFF removes the folder's messages from the Unified Inbox immediately (AC-3).
- [ ] Toggling membership ON for any folder (system or user-created) makes its messages appear in the Unified Inbox immediately (AC-4).
- [ ] The toggle reflects in under one second under normal load (NFR-1).
- [ ] The toggle is accessible via keyboard (NFR-7).

## Blocked by

- Blocked by `1-folder-membership-state`
- Blocked by `4-basic-unified-message-list`

## User stories addressed

- US-4 (toggle via context action on folder)
- US-5 (any folder, system or user-created)
