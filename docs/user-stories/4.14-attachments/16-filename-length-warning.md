## Parent Feature

#4.14 Attachments

## What to build

When an attachment has a filename longer than 60 characters, display a visible warning in the compose view citing interoperability concerns — some mail clients may not handle long filenames correctly.

Covers epic sections: US-26, FR-30, AC-14.

## Acceptance criteria

- [ ] A file with a name of 65 characters triggers a visible filename-length warning (AC-14).
- [ ] The warning indicates interoperability concerns with some mail clients.
- [ ] The warning does not prevent the user from sending — it is advisory.

## Blocked by

- Blocked by `2-compose-attachment-list`

## User stories addressed

- US-26
