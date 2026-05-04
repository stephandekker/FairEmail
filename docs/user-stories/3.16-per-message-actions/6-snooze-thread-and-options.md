## Parent Feature

#3.16 Per-Message Actions

## What to build

Extend snooze to operate at thread scope and add optional configurable behaviours. When a message in a conversation is snoozed, all messages in the thread are snoozed together and all reappear when the snooze expires (Design Note N-1). Add independently toggleable options: auto-flag on snooze, auto-unflag on wake, auto-mark-high-importance on wake, auto-cancel-snooze on manual move (FR-5, FR-9).

## Acceptance criteria

- [ ] Snoozing one message in a three-message conversation hides all three (AC-3)
- [ ] All three messages reappear together when the snooze expires (AC-3)
- [ ] With "auto-flag on snooze" enabled, snoozing a message sets its flag
- [ ] With "auto-unflag on wake" enabled, the flag is cleared when the message reappears
- [ ] With "auto-mark-high-importance on wake" enabled, woken messages are marked high importance
- [ ] With "auto-cancel-snooze on move" enabled, moving a snoozed message cancels its snooze
- [ ] Each optional behaviour is independently toggleable in settings

## Blocked by

- Blocked by 5-snooze-basic

## User stories addressed

- US-6 (snooze entire thread together)
- US-7 (auto-flag on snooze / auto-unflag on wake)
- US-8 (auto-mark high importance on wake)
- US-9 (auto-cancel snooze on move)
