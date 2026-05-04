## Parent Feature

#4.14 Attachments

## What to build

Allow users to compress a non-inline attachment in-place from the compose attachment list. The action replaces the original file with a .zip archive containing it. This is available only for non-inline attachments.

Covers epic sections: US-23, FR-26, AC-12.

## Acceptance criteria

- [ ] Non-inline attachments offer a "zip" action in the compose list.
- [ ] Zipping replaces the original attachment with a .zip file of smaller or equal size (AC-12).
- [ ] The zip action is not available for inline attachments.
- [ ] Zipping works offline.

## Blocked by

- Blocked by `2-compose-attachment-list`

## User stories addressed

- US-23
