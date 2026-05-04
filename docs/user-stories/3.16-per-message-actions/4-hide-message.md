## Parent Feature

#3.16 Per-Message Actions

## What to build

A toggle action that hides a message indefinitely from normal message views without moving or deleting it. The message remains in its original folder but is excluded from the default message list and from notifications. Invoking hide on an already-hidden message unhides it, restoring normal visibility. Hide state is local-only — it is never synchronised to the server (FR-10 through FR-13).

Per Design Note N-2, the source application implements hide as a snooze with an indefinite wake-up time. This story does not prescribe that implementation but requires the hide behaviour to be complete and demoable on its own.

## Acceptance criteria

- [ ] Hiding a message removes it from the normal message list (AC-4)
- [ ] Hidden messages are excluded from notifications
- [ ] Invoking hide on a hidden message unhides it, restoring visibility (AC-4)
- [ ] Hide state is local-only and not transmitted to the server (NFR-8)
- [ ] Hidden messages survive application restart in their hidden state

## Blocked by

None — can start immediately.

## User stories addressed

- US-10 (hide a message indefinitely)
- US-11 (unhide a hidden message)
- US-12 (hidden messages excluded from notifications)

## Notes

- US-13 and FR-14 (shared filter toggle for snoozed/hidden) are addressed in story 7-snoozed-hidden-filter, which depends on both this story and the snooze story.
- Open question OQ-7: whether "hide" is distinct enough from "archive" in users' mental models is unresolved. This story implements hide as specified; any renaming or repositioning is deferred.
