# Search from Unified Inbox

## Parent Feature

#3.1 Unified Inbox

## What to build

When a search is initiated from the Unified Inbox, it must span every folder that is a current member of the Unified Inbox across every synchronized account (FR-28). A search initiated from any other context (single folder, single account) must remain scoped to that context and must not be affected by unified membership (FR-29).

## Acceptance criteria

- [ ] Searching from the Unified Inbox returns results from at least two distinct accounts when both have matching messages in unified-member folders (AC-10).
- [ ] Searching from a single folder returns results only from that folder (AC-11).
- [ ] Search results respect the current unified membership: removing a folder from unified and re-searching excludes that folder's messages.
- [ ] Search works offline for already-fetched messages.

## Blocked by

- Blocked by `4-basic-unified-message-list`

## User stories addressed

- US-22 (search spans all unified-member folders)
