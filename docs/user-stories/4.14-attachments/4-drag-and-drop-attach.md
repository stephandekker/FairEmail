## Parent Feature

#4.14 Attachments

## What to build

Support drag-and-drop of files from the system file manager into the compose window. Dropped files are added as attachments using the same MIME detection and sanitization logic from the basic file attachment slice. This provides the second attachment source alongside the file picker.

Covers epic sections: US-2, FR-5, AC-5.

## Acceptance criteria

- [ ] Dragging one or more files from the system file manager into the compose window adds them as attachments (AC-5).
- [ ] Dropped files are detected with correct MIME type and sanitized filename.
- [ ] Visual feedback (drop target highlight) is shown when files are dragged over the compose area.

## Blocked by

- Blocked by `1-basic-file-attachment`

## User stories addressed

- US-2

## Notes

- OQ-4 from the epic asks whether dragging an image into the compose *body* (vs. attachment area) should default to inline or attachment. This story covers drag-and-drop for file attachment only. The inline-vs-attach image decision is handled in `5-image-inline-vs-attach`. The interaction between drop target area and disposition may need a design decision documented in that story.
