## Parent Feature

#3.16 Per-Message Actions

## What to build

Allow users to attach a plain-text, multi-line note to any message. Notes are stored locally only (never synced to server) with a clear disclaimer in the editing dialog. Each note has an optional color, selectable via a color picker, with a default color preference remembered across notes. Notes display inline in the message list below the preview text, in bold, using the note's assigned color. When a message exists as multiple copies (same message identifier) across folders within the same account, a note added to one copy appears on all copies. Users can filter to show only messages with notes and can clear a note via a reset action (FR-34 through FR-40).

## Acceptance criteria

- [ ] User can add a multi-line plain-text note to any message
- [ ] Note editing dialog shows a disclaimer that notes are local-only (AC-11)
- [ ] Notes display inline below the preview text in bold in the message list (AC-10)
- [ ] User can assign a color to a note via color picker; default color is remembered
- [ ] A note on one copy of a message (same message-id) appears on all copies in the same account
- [ ] Filtering by "has notes" includes annotated messages (AC-10)
- [ ] Notes persist across application restarts (AC-11)
- [ ] Notes are NOT visible from another email client (AC-11)
- [ ] User can clear a note and its color via a reset action

## Blocked by

None — can start immediately.

## User stories addressed

- US-29 (attach a plain-text note)
- US-30 (notes displayed inline in message list)
- US-31 (assign a color to a note)
- US-32 (clear disclaimer that notes are local-only)
- US-33 (note applied to all copies of same message)
- US-34 (filter by messages with notes)

## Notes

- Open question OQ-4: whether note content should be full-text searchable (not just presence-filterable) is unresolved. This story implements presence-filtering only, matching the source application's behaviour.
