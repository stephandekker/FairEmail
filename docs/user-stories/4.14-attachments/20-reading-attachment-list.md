## Parent Feature

#4.14 Attachments

## What to build

Display all non-inline attachments on a received message in a list showing: type icon, filename, file size, and download state (not downloaded / downloading / available). This is the reading-side foundation — subsequent slices build download, open, save, and share actions on top of this list.

Covers epic sections: US-31, FR-36, AC-17 (list display part).

## Acceptance criteria

- [ ] Received messages with attachments display a list of non-inline attachments.
- [ ] Each item shows a type-appropriate icon, filename, file size, and download state.
- [ ] Download states are visually distinguished: not downloaded, downloading, available.
- [ ] The list is keyboard-navigable and carries screen-reader labels (NFR-7).

## Blocked by

None - can start immediately.

## User stories addressed

- US-31
