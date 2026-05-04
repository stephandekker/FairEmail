## Parent Feature

#4.14 Attachments

## What to build

Display the list of attachments in the compose view with full item details and collapsible behavior. When attachments are present, the compose view shows them in a list. Each item displays a type-appropriate icon, filename, file size, disposition indicator (inline items visually dimmed), and a progress indicator during processing. The list can be collapsed to a single summary line showing the total attachment count, and the collapsed/expanded state persists across compose sessions as a user preference.

Covers epic sections: US-17, US-18, US-19, US-24, FR-20, FR-21, FR-22, NFR-2, NFR-7, AC-9, N-4, N-7.

## Acceptance criteria

- [ ] Attachments are displayed in a list with type icon, filename, file size, and disposition indicator.
- [ ] Inline items are visually distinguished (e.g. dimmed) from regular attachments.
- [ ] A processing progress indicator is shown for items being processed.
- [ ] The list can be collapsed to a single summary line showing total attachment count.
- [ ] Expanding the collapsed list reveals all items.
- [ ] The collapsed/expanded state persists after closing and reopening the draft (AC-9).
- [ ] The list remains usable with at least 50 attachments without degrading performance (NFR-2).
- [ ] All list items and actions are keyboard-navigable and carry screen-reader labels (NFR-7).

## Blocked by

- Blocked by `1-basic-file-attachment`

## User stories addressed

- US-17
- US-18
- US-19
- US-24
