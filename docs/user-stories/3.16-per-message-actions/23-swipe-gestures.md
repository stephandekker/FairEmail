## Parent Feature

#3.16 Per-Message Actions

## What to build

Allow users to bind per-message actions to left-swipe and right-swipe gestures on message list items. Each direction is independently configurable from the full action set (archive, trash, delete permanently, junk, move-to, snooze, hide, flag, mark read/unread, set importance, reply, forward, text-to-speech, summarise, or "ask me"). Swipe actions are configurable per account with sensible defaults (left → trash, right → archive). Users can adjust swipe sensitivity or disable swipe gestures entirely (FR-73, FR-74).

## Acceptance criteria

- [ ] Swiping left on a message list item triggers the configured left-swipe action
- [ ] Swiping right triggers the configured right-swipe action
- [ ] Binding left-swipe to "snooze" and swiping left opens the snooze dialog (AC-21)
- [ ] Swipe bindings are configurable per account
- [ ] Default bindings are left → trash, right → archive
- [ ] Swipe sensitivity is adjustable
- [ ] Swipe gestures can be disabled entirely (AC-24)
- [ ] With swipe disabled or at minimum sensitivity, accidental drags do not trigger actions (AC-24)

## Blocked by

- Blocked by 22-configurable-toolbar-context-menu

## User stories addressed

- US-64 (bind actions to swipe gestures)
- US-67 (option to disable swipe gestures entirely)
