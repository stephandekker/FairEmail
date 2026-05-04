## Parent Feature

#3.16 Per-Message Actions

## What to build

A keyword management dialog that lists all IMAP keywords on a selected message (or the union across a multi-selection, with indeterminate checkboxes for partial application). Users can add/remove keywords via checkboxes, create new custom keywords by name (sanitised to IMAP format), and the application enforces a maximum of 32 keywords per message. Keywords sync bidirectionally with the server. Users can assign a local display color and alias (friendly name) to any keyword. System-internal keywords are hidden from the UI. Users can define a set of global keywords available for quick selection across all folders/accounts. Users can search for messages by keyword locally and via server-side IMAP search (FR-41 through FR-46, FR-48 through FR-50).

## Acceptance criteria

- [ ] Keyword management dialog lists all user-visible keywords on the selected message(s)
- [ ] Multi-selection shows indeterminate state for partially-applied keywords
- [ ] User can add a new keyword by typing a name; invalid characters are sanitised
- [ ] Maximum of 32 keywords per message is enforced
- [ ] Adding/removing a keyword syncs to the IMAP server (AC-12)
- [ ] User can assign a display color and alias to a keyword (AC-13)
- [ ] System-internal keywords (forwarded, delivery-receipt-sent, filtered, etc.) are hidden
- [ ] User can define global keywords available across all folders/accounts
- [ ] User can search for messages by keyword locally and via server-side search

## Blocked by

None — can start immediately.

## User stories addressed

- US-35 (view, add, remove keywords via dialog)
- US-36 (create new custom keywords)
- US-37 (assign display color to keywords)
- US-38 (assign display alias to keywords)
- US-39 (bidirectional keyword sync)
- US-40 (search by keyword)
- US-42 (hide system-internal keywords)
- US-43 (global keywords for quick selection)

## Notes

- Open question OQ-9: whether the 32-keyword limit should be server-detected or fixed is unresolved. This story implements a fixed cap of 32 matching the source application; detecting actual server limits is deferred.
