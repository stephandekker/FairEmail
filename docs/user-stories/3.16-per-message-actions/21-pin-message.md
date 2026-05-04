## Parent Feature

#3.16 Per-Message Actions

## What to build

Allow users to create a persistent system-level shortcut (desktop shortcut, panel launcher entry, or equivalent) for a specific message or conversation thread. The shortcut includes a user-editable label (defaulting to the message subject) and an icon derived from the sender's avatar or a generic mail icon. Activating the shortcut opens the application directly to the pinned conversation (FR-15 through FR-17, Design Note N-3).

## Acceptance criteria

- [ ] User can pin a message or conversation via a per-message action
- [ ] A system-level shortcut is created (desktop file, panel entry, or equivalent) (AC-5)
- [ ] The shortcut label defaults to the message subject and is user-editable
- [ ] The shortcut icon uses the sender's avatar or a generic mail icon
- [ ] Activating the shortcut opens the application directly to the conversation (AC-5)
- [ ] Pin is separate from flag/star — they share no state (Design Note N-3)

## Blocked by

None — can start immediately.

## User stories addressed

- US-14 (pin a message to a system-level shortcut)
- US-15 (activating shortcut opens the conversation)

## Notes

- Open question OQ-1: the desktop equivalent of a mobile home-screen shortcut needs design work. Options include `.desktop` files, panel launcher entries, or an in-application "pinned messages" virtual folder. This story requires at least one working system-level shortcut mechanism; the exact form is a HITL design decision.
