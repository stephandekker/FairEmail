# Basic Unified Message List

## Parent Feature

#3.1 Unified Inbox

## What to build

Implement the core Unified Inbox view: a single message list that merges messages from every folder whose unified-inbox membership is enabled and whose owning account is currently synchronized (FR-13, FR-14). Messages are displayed in chronological order by default. Each message carries an unambiguous visual indicator of its owning account and/or folder — color, name, icon, or similar (FR-18, US-12, AC-15).

The view must support all message-list display options that a single folder supports: density, columns, conversation grouping toggles, date headers, account-color stripe, account name, preview text, etc. (FR-17).

This is the first slice where the user can actually *see* the Unified Inbox working end-to-end.

## Acceptance criteria

- [ ] Opening the Unified Inbox displays messages from all folders with unified-inbox membership enabled (AC-1 for newly added accounts once synced).
- [ ] Only messages from currently-synchronized accounts appear (FR-14).
- [ ] Each message shows an indicator identifying its owning account and/or folder (AC-15).
- [ ] The view supports the same display options as a single-folder message list (density, columns, date headers, preview, etc.).
- [ ] The default sort is chronological (newest first).
- [ ] The view is fully readable offline for already-fetched messages (NFR-3).
- [ ] The view remains usable with at least 20 accounts and 200 unified-member folders (NFR-2).

## Blocked by

- Blocked by `1-folder-membership-state`
- Blocked by `3-unified-inbox-nav-entry`

## User stories addressed

- US-10 (single chronological merged list)
- US-12 (visual indicator per message for account/folder)
