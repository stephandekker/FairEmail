# Per-Message Actions in Unified Inbox

## Parent Feature

#3.1 Unified Inbox

## What to build

Ensure that every per-message action available in a regular folder view is also available when viewing messages in the Unified Inbox (FR-20): read/unread, flag/unflag, snooze, hide, move, copy, delete, archive, mark important, reply, forward, etc. Actions must be applied to the original message in its real folder and account — never to a copy or proxy (FR-21). Multi-select and bulk actions must also work with the same semantics as in a folder view (FR-22).

## Acceptance criteria

- [ ] All per-message actions from a folder view are available in the Unified Inbox (FR-20).
- [ ] Marking a message read in the Unified Inbox marks it read in the original folder, and vice versa (AC-7).
- [ ] Moving a message from the Unified Inbox moves it in the original account.
- [ ] Deleting a message from the Unified Inbox deletes it in the original account.
- [ ] Multi-select works: selecting messages from different accounts and applying a bulk action (e.g. mark all read) applies to each in its respective account (US-16).
- [ ] Flag state is consistent between Unified Inbox and original folder (NFR-4).

## Blocked by

- Blocked by `4-basic-unified-message-list`

## User stories addressed

- US-14 (all per-message actions available)
- US-15 (actions applied to original message in real folder/account)
- US-16 (multi-select bulk actions)
