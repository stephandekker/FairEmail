# Hidden and Snoozed Message Handling

## Parent Feature

#3.1 Unified Inbox

## What to build

Hidden messages must never appear in the Unified Inbox under any circumstance (FR-15, US-26). Snoozed messages must be excluded from the Unified Inbox by default, but a per-view filter must allow the user to include them (FR-16, US-27). Flagged/starred messages must always be included like any other message — flagging must never remove a message from the unified view (US-28).

## Acceptance criteria

- [ ] Hidden messages do not appear in the Unified Inbox (AC-12).
- [ ] Snoozed messages do not appear in the Unified Inbox by default (AC-12).
- [ ] Enabling a "show snoozed" filter makes snoozed messages appear in the Unified Inbox (AC-12).
- [ ] Flagged/starred messages appear normally in the Unified Inbox.
- [ ] The snoozed filter state is part of the Unified Inbox's independent filter persistence.

## Blocked by

- Blocked by `13-sort-and-filter`

## User stories addressed

- US-26 (hidden messages excluded always)
- US-27 (snoozed excluded by default, visible via filter)
- US-28 (flagged messages always included)

## Notes

Open question OQ-6 asks whether the same snoozed-hidden-by-default behavior should apply to external surfaces (tray etc.). This story covers only the in-app Unified Inbox view; external surface behavior is deferred to story 17.
