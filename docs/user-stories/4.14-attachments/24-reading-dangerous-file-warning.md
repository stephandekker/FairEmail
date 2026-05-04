## Parent Feature

#4.14 Attachments

## What to build

Before opening a downloaded attachment with a dangerous file extension, display a warning to the user. The warning must not be dismissible permanently — each occurrence requires acknowledgement (NFR-5). The application never automatically executes or opens an attachment without explicit user action.

Covers epic sections: US-40, FR-46, NFR-5.

## Acceptance criteria

- [ ] Attempting to open an attachment with a dangerous extension triggers a warning dialog.
- [ ] The warning requires explicit acknowledgement before the file is opened.
- [ ] The warning cannot be permanently dismissed — it appears every time (NFR-5).
- [ ] No attachment is automatically executed or opened without user action (NFR-5).

## Blocked by

- Blocked by `22-open-save-share-attachments`

## User stories addressed

- US-40
