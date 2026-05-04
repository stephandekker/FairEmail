## Parent Feature

#4.14 Attachments

## What to build

When a received message contains TNEF-encapsulated content (winmail.dat), automatically unpack it and present the embedded attachments as normal attachments in the reading-side attachment list.

Covers epic sections: US-39, FR-44, AC-19.

## Acceptance criteria

- [ ] TNEF-encapsulated content is automatically detected and unpacked.
- [ ] Embedded attachments from the TNEF container are listed as normal attachments (AC-19).
- [ ] The unpacked attachments are downloadable, openable, saveable, and shareable like any other attachment.

## Blocked by

- Blocked by `20-reading-attachment-list`

## User stories addressed

- US-39
