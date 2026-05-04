## Parent Feature

#4.14 Attachments

## What to build

Render inline images within the message body at their Content-ID reference positions when reading a received message, subject to the user's remote-content and auto-download settings. Provide a toggle to independently control visibility of inline images (separate from regular attachment behavior).

Covers epic sections: US-32, FR-37, FR-38.

## Acceptance criteria

- [ ] Inline images are rendered within the message body at their CID reference positions.
- [ ] Inline image display respects the user's remote-content and auto-download settings (FR-37).
- [ ] The user can toggle visibility of inline images independently of regular attachments (FR-38).

## Blocked by

- Blocked by `20-reading-attachment-list`

## User stories addressed

- US-32
