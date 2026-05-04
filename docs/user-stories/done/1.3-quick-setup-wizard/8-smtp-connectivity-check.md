## Parent Feature

#1.3 Quick Setup Wizard

## What to build

The SMTP portion of the connectivity check: connect to the outgoing server, authenticate, and query the maximum message size (FR-17 steps 4-6).

**Behavior:**
- Connect to the SMTP server using the hostname, port, and encryption mode from the detected provider settings (FR-17.4).
- Authenticate using the user's credentials (FR-17.5).
- Query the outgoing server's maximum message size, if advertised (FR-17.6).

**Critical invariant:** both IMAP and SMTP must succeed for the account to be saved. If IMAP succeeds but SMTP fails, the user sees an error — the account is NOT silently saved in a half-working state (AC-7, Design Note N-2).

## Acceptance criteria

- [ ] The wizard connects to the SMTP server using detected hostname, port, and encryption mode
- [ ] Authentication succeeds with valid credentials
- [ ] The server's maximum message size is queried and stored if advertised (FR-17.6)
- [ ] If IMAP succeeds but SMTP fails, the user sees an error and the account is not saved (AC-7)
- [ ] If SMTP succeeds but IMAP failed, the account is not saved
- [ ] The SMTP check result is available for downstream slices (review screen, account creation)

## Blocked by

- Blocked by 7-imap-connectivity-check

## User stories addressed

- US-11 (connectivity check tests both IMAP and SMTP)
