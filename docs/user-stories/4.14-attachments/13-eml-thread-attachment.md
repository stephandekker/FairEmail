## Parent Feature

#4.14 Attachments

## What to build

Provide a mechanism to attach the current conversation thread as a single EML file. The generated .eml file contains the full conversation (headers, body, attachments per RFC 5322). EML attachments in the compose list also offer a "save as EML" context menu action.

Covers epic sections: US-12, FR-8, FR-27, AC-7.

## Acceptance criteria

- [ ] The compose window offers an "attach thread as EML" action.
- [ ] The resulting attachment is a single .eml file containing the full conversation (AC-7).
- [ ] The EML file conforms to RFC 5322 and is openable by other mail clients.
- [ ] EML attachments in the compose list offer a "save as EML" context menu action (FR-27).

## Blocked by

- Blocked by `1-basic-file-attachment`

## User stories addressed

- US-12
