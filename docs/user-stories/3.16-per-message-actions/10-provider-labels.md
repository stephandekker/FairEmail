## Parent Feature

#3.16 Per-Message Actions

## What to build

For accounts that use a provider-specific label system (e.g. provider-managed label APIs), display provider labels alongside IMAP keywords in the keyword management dialog but route label operations through the provider's own protocol rather than standard IMAP keyword commands. This ensures label add/remove works correctly on providers whose labels are not standard IMAP keywords (FR-47).

## Acceptance criteria

- [ ] Provider labels appear in the keyword management dialog alongside IMAP keywords
- [ ] Adding/removing a provider label uses the provider's protocol, not IMAP keyword commands
- [ ] Provider labels are visually distinguishable from standard IMAP keywords in the dialog
- [ ] Label operations sync correctly with the provider

## Blocked by

- Blocked by 9-imap-keywords-management

## User stories addressed

- US-41 (provider-specific labels displayed and managed via provider protocol)
