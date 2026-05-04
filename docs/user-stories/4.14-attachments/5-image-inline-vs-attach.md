## Parent Feature

#4.14 Attachments

## What to build

When a user adds an image to a compose draft, present a dialog that lets them choose between inserting it inline (embedded in the message body via Content-ID reference) or attaching it as a regular file attachment. The choice determines the MIME disposition (inline vs. attachment). Inline images are rendered within the message body at their CID reference point; regular attachments appear only in the attachment list.

This is the minimal image-add path — resize, metadata strip, and HEIC re-encoding are layered on in subsequent slices.

Covers epic sections: US-4, FR-2, FR-12 (inline/attach choice only), AC-2 (inline/attach part), N-1.

## Acceptance criteria

- [ ] When adding an image, a dialog asks the user to choose inline or attachment disposition (AC-2, partial).
- [ ] Choosing "inline" inserts the image in the message body via Content-ID reference.
- [ ] Choosing "attachment" adds the image as a regular file attachment.
- [ ] The disposition is reflected in the attachment list (inline items dimmed per N-4).

## Blocked by

- Blocked by `2-compose-attachment-list`

## User stories addressed

- US-4

## Notes

- OQ-4 from the epic (drag-and-drop into body vs. attachment area defaulting to inline) is relevant here. The resolution of that open question will affect how this dialog interacts with the drag-and-drop slice.
