# Synchronization Scope Interaction

## Parent Feature

#3.1 Unified Inbox

## What to build

Folders belonging to an account that is fully un-synchronized (disabled, suspended) must not appear in the Unified Inbox even if their unified-inbox membership is enabled (FR-33). When synchronization is re-enabled, those folders' messages must reappear. Folders belonging to an on-demand account may appear in the Unified Inbox if membership is enabled, but only actually-fetched messages will be visible; the user must not be misled that the list is exhaustive for on-demand accounts (FR-34).

## Acceptance criteria

- [ ] Disabling sync for an account causes its messages to disappear from the Unified Inbox within the next refresh cycle, even with membership still enabled (AC-14).
- [ ] Re-enabling sync restores those messages in the Unified Inbox (AC-14).
- [ ] On-demand account messages appear only if fetched; no false completeness is implied.
- [ ] Membership state is preserved when an account is disabled/re-enabled.

## Blocked by

- Blocked by `4-basic-unified-message-list`

## User stories addressed

- US-11 (un-synchronized account messages excluded)

## Notes

Open question OQ-2 asks whether on-demand account messages should carry a visual indicator (e.g. "this stream is not live"). This story implements the data-level behavior; the visual indicator, if desired, can be added as a follow-up.
