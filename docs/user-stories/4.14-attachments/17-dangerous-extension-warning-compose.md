## Parent Feature

#4.14 Attachments

## What to build

Maintain a list of known dangerous file extensions (matching common provider behavior per N-8: executables, scripts, etc.). When a user attaches a file with such an extension, display a warning in the compose view alerting them that the recipient's mail system may block or flag the file.

Covers epic sections: US-27, FR-31, AC-15.

## Acceptance criteria

- [ ] Attaching an .exe file triggers a dangerous-extension warning (AC-15).
- [ ] The dangerous-extension list mirrors what major providers block or warn about (N-8).
- [ ] The warning is advisory and does not prevent sending.

## Blocked by

- Blocked by `2-compose-attachment-list`

## User stories addressed

- US-27
