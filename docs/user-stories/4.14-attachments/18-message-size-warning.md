## Parent Feature

#4.14 Attachments

## What to build

When the combined message size (body + all attachments) exceeds a configurable threshold, display a size warning indicating that some providers may reject the message. Provider-specific limits (e.g. 25 MB for Gmail, 20 MB for Outlook, 25 MB for Yahoo) are documented and referenced in the warning.

Covers epic sections: US-28, FR-32, AC-16 (implicit — large messages).

## Acceptance criteria

- [ ] A size warning is displayed when the combined message size exceeds the configurable threshold (FR-32).
- [ ] The warning references provider-specific limits (Gmail 25 MB, Outlook 20 MB, Yahoo 25 MB).
- [ ] The size threshold is configurable.
- [ ] The warning is advisory and does not prevent sending.

## Blocked by

- Blocked by `2-compose-attachment-list`

## User stories addressed

- US-28

## Notes

- OQ-1 from the epic asks whether the application should enforce hard per-provider caps or continue with advisory warnings only. This story implements advisory warnings per the current epic specification.
