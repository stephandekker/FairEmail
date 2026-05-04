# Folder Unified-Inbox Membership State

## Parent Feature

#3.1 Unified Inbox

## What to build

Introduce a persistent, per-folder boolean property — "unified-inbox membership" — that records whether a folder's messages should appear in the Unified Inbox. This property must be independent of the folder's notification setting, synchronization setting, navigation-pane visibility, and role/type (FR-1, FR-3). It must survive application restart, re-login, and server folder-list re-discovery (FR-4). It must be observable and editable at runtime without data loss (FR-2).

This is the foundational data slice: no other Unified Inbox behavior can be built until this property exists and is durable.

## Acceptance criteria

- [ ] Every folder has an independent boolean "unified-inbox membership" property that defaults to `false`.
- [ ] The property persists across application restart.
- [ ] The property survives a server folder-list re-discovery (folders are re-detected without losing their membership state).
- [ ] Changing the property does not alter the folder's notification setting, sync setting, or navigation visibility.
- [ ] The property is queryable at runtime (e.g. "give me all folders where unified = true").

## Blocked by

None — can start immediately.

## User stories addressed

- US-8 (membership choices persist across restarts and re-syncing)

## Notes

The existing FairEmail Android codebase already stores a `unified` boolean on `EntityFolder` (default `false`, indexed). The Linux desktop implementation may reuse or adapt this pattern, but the epic does not prescribe a specific persistence mechanism.
