# Unified Inbox Navigation Entry

## Parent Feature

#3.1 Unified Inbox

## What to build

Add the Unified Inbox as a clearly-labeled, top-level entry in the application's primary navigation pane, separate from per-account folder trees (FR-35, US-30). The entry must be one click away at all times (US-9). Provide a setting to collapse or hide the unified-inbox area of the navigation pane without losing any per-folder membership choices (FR-36, US-31, AC-17). Clicking the entry opens the Unified Inbox view (delivered in the next slice).

This slice establishes the navigation surface only — the actual message list is a separate story.

## Acceptance criteria

- [ ] The Unified Inbox appears as a top-level, clearly-labeled entry in the navigation pane, distinct from per-account folder trees (US-30).
- [ ] Clicking the entry navigates to the Unified Inbox view.
- [ ] A setting allows the user to collapse/hide the unified-inbox navigation entry (US-31).
- [ ] Hiding the navigation entry does not modify any folder's unified-inbox membership state (AC-17).
- [ ] The entry is reachable via keyboard and has appropriate screen-reader labels (NFR-7).

## Blocked by

None — can start immediately (parallel with slice 1).

## User stories addressed

- US-9 (open Unified Inbox from primary navigation)
- US-30 (clearly-labeled root entry in nav pane)
- US-31 (collapse/hide the unified-inbox nav area)
