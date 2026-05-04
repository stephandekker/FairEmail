## Parent Feature

#1.3 Quick Setup Wizard

## What to build

When the user enters an email address belonging to a known proprietary provider that does not support standard IMAP/SMTP (e.g. ProtonMail, Tutanota), the wizard immediately rejects the address with a clear, user-friendly message — without attempting any network-based detection or connection (FR-13, Design Note N-5).

**Behavior:**
- After the user enters their email address and triggers the check, the wizard first checks the domain against the proprietary-provider list.
- If the domain matches, an immediate, non-technical message is displayed explaining that the provider does not support standard email protocols and is not compatible with the application.
- No DNS lookups, ISPDB queries, or connection attempts are made for proprietary domains.

## Acceptance criteria

- [ ] Entering a ProtonMail domain (e.g. protonmail.com, proton.me) displays an immediate rejection message (AC-6)
- [ ] Entering a Tutanota domain (e.g. tutanota.com, tuta.io) displays an immediate rejection message (AC-6)
- [ ] The rejection message is user-friendly and non-technical (AC-6, FR-25)
- [ ] No network requests are made for proprietary provider domains (FR-13)
- [ ] The rejection does not prevent the user from going back and trying a different address

## Blocked by

- Blocked by 2-bundled-provider-database

## User stories addressed

- US-9 (proprietary provider rejection with clear message)

## Notes

- Open Question OQ-3 in the epic asks whether the proprietary-provider list should be part of the provider database or a static deny-list. This is a design decision to be resolved during implementation. The slice should work regardless of where the list is stored.
