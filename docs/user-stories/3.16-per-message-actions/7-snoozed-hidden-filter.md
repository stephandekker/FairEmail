## Parent Feature

#3.16 Per-Message Actions

## What to build

A single filter toggle in the message list that, when enabled, reveals all currently snoozed and hidden messages in the current view. This shared toggle covers both snooze and hide states. Snoozed messages should show a wake-up time indicator; hidden messages should be distinguishable from snoozed ones (FR-14, and the filter aspect of US-5 and US-13).

## Acceptance criteria

- [ ] A filter toggle is available in the message list view
- [ ] Enabling the toggle reveals hidden messages in the current folder (AC-4)
- [ ] Enabling the toggle reveals snoozed messages with their wake-up time
- [ ] Snoozed and hidden messages are visually distinguishable in the filtered view
- [ ] Disabling the toggle re-hides suppressed messages
- [ ] The toggle works in both single-folder views and the unified inbox

## Blocked by

- Blocked by 4-hide-message
- Blocked by 5-snooze-basic

## User stories addressed

- US-5 (filter toggle to see snoozed messages)
- US-13 (hidden messages share the filter toggle with snoozed messages)
