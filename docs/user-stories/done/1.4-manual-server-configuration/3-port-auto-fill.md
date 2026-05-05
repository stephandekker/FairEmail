# Port Auto-Fill on Encryption Mode Change

## Parent Feature

#1.4 Manual Server Configuration

## What to build

When the user changes the encryption mode on the inbound configuration form, the port field auto-fills with the conventional default for the selected protocol and encryption combination:

- IMAP: SSL/TLS -> 993, STARTTLS -> 143, None -> 143
- POP3: SSL/TLS -> 995, STARTTLS -> 110, None -> 110

The auto-fill replaces the current port value only if the current value is empty or matches one of the known defaults. A user-entered non-default port is not overwritten.

Covers epic sections: FR-6, FR-7.

## Acceptance criteria

- [ ] Changing encryption mode on an IMAP account auto-fills port: SSL/TLS -> 993, STARTTLS -> 143, None -> 143
- [ ] Auto-fill replaces the port only when the current value is empty or matches a known default
- [ ] A user-entered non-default port (e.g. 1993) is NOT overwritten when encryption mode changes
- [ ] Port defaults correctly on initial form load (SSL/TLS selected by default -> port 993 for IMAP)

## Blocked by

- Blocked by `1-create-imap-account`

## User stories addressed

- US-2 (port auto-fill on encryption change)

## Notes

POP3 port defaults are included in the acceptance criteria spec here but the POP3 protocol option is delivered in story 21. When POP3 support lands, it should verify these defaults work for POP3 as well.
