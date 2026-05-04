## Parent Feature

#3.16 Per-Message Actions

## What to build

Extend the move action with optional configurable behaviours: move the entire conversation thread (not just the selected message), optionally include sent-folder messages when moving a thread, automatically mark moved messages as read, automatically remove the flag, automatically reset importance (except for moves to trash/archive/junk), and automatically cancel snooze on moved messages. Each option is independently toggleable (FR-54).

## Acceptance criteria

- [ ] With "move entire thread" enabled, moving one message moves all messages in the conversation
- [ ] With "include sent messages" enabled, sent-folder messages in the thread are also moved
- [ ] With "auto-mark-read" enabled, moved messages are marked as read
- [ ] With "auto-remove-flag" enabled, moved messages have their flag cleared
- [ ] With "auto-reset-importance" enabled, importance is reset (except trash/archive/junk moves)
- [ ] With "auto-cancel-snooze" enabled, snoozed messages have their snooze cancelled on move
- [ ] Each option is independently configurable in settings

## Blocked by

- Blocked by 11-move-message-with-undo

## User stories addressed

- US-47 (option to move entire conversation thread)
- US-48 (auto-mark-read and auto-remove-flag on move)
