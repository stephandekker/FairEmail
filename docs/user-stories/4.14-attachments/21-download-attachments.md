## Parent Feature

#4.14 Attachments

## What to build

Enable downloading individual attachments on demand from the reading-side attachment list. Also provide a "download all" action. Display per-attachment download progress, and show error messages for failed downloads with a retry option.

Covers epic sections: US-33, US-34, FR-39, FR-47, AC-17.

## Acceptance criteria

- [ ] Each attachment has a "download" action that fetches the file on demand.
- [ ] A "download all" action downloads all attachments at once (FR-39).
- [ ] Download progress is displayed per attachment (AC-17, FR-47).
- [ ] Failed downloads show an error message and offer a retry action (FR-47).
- [ ] On completion, the attachment state updates to "available" (AC-17).

## Blocked by

- Blocked by `20-reading-attachment-list`

## User stories addressed

- US-33
- US-34
