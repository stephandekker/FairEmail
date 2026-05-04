## Parent Feature

#3.16 Per-Message Actions

## What to build

A single-action toggle that flags (stars) or unflags a message and synchronises the state bidirectionally with the mail server via the IMAP `\Flagged` flag. The flag state must be visible in the message list (star icon) and in the single-message view. Users must be able to filter and sort by flagged/unflagged status. Flagging must not alter read/unread state, importance, or folder membership (FR-18, FR-20, FR-23).

This is the thinnest possible per-message action slice: one toggle, one IMAP flag, one UI indicator. It establishes the end-to-end pattern (UI → local state → operation queue → IMAP sync → UI update) that subsequent action stories reuse.

## Acceptance criteria

- [ ] Clicking the flag/star control on a message toggles it between flagged and unflagged
- [ ] The IMAP `\Flagged` flag is set/cleared on the server after toggling
- [ ] Flagged messages show a star icon in the message list and message view
- [ ] A flag set by another IMAP client appears in the application after sync
- [ ] Users can filter the message list to show only flagged messages
- [ ] Users can sort messages by flagged/unflagged status
- [ ] Flagging does not change read/unread state, importance, or folder

## Blocked by

None — can start immediately.

## User stories addressed

- US-16 (toggle flag with single action)
- US-18 (flag synchronised to server)
- US-19 (filter and sort by flagged status)
