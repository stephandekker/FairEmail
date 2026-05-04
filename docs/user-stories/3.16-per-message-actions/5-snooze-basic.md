## Parent Feature

#3.16 Per-Message Actions

## What to build

Allow users to snooze a message until a user-chosen future date and time. Snoozed messages are immediately hidden from normal views, marked as read, and excluded from notifications. When the wake-up time arrives, the message reappears as unread and optionally triggers a notification. Provide a snooze dialog with contextual presets (1 hour, 1 day, this afternoon, tomorrow morning, this weekend, next work week, next week, custom date/time picker) that suppress options referring to times already past. The default snooze duration is user-configurable (1–24 hours, default 1 hour). Users can cancel a snooze at any time to restore visibility immediately. Enforce a maximum of concurrently snoozed messages (at least 300) (FR-1 through FR-4, FR-6 through FR-8).

## Acceptance criteria

- [ ] Snoozing a message hides it immediately from normal views (AC-1)
- [ ] Snoozed message is marked as read and excluded from notifications
- [ ] Snooze dialog offers contextual presets that hide past-time options (AC-1)
- [ ] Custom date/time picker allows arbitrary future snooze times
- [ ] At wake-up time, message reappears as unread with optional notification (AC-1)
- [ ] Cancelling a snooze makes the message visible immediately (AC-2)
- [ ] Attempting to exceed the snooze limit produces an error
- [ ] Default snooze duration is configurable (1–24 hours)

## Blocked by

None — can start immediately.

## User stories addressed

- US-1 (snooze until specific date/time)
- US-2 (quick-access snooze presets)
- US-3 (message reappears at wake-up time)
- US-4 (cancel a snooze)
- US-5 (snoozed messages hidden from normal views — partial; filter toggle in story 7)

## Notes

- Open question OQ-5: the 300-message snooze limit comes from a mobile platform alarm constraint. On desktop this may not apply. This story implements the limit as a configurable safeguard; the exact value can be adjusted later.
