## Parent Feature

#1.3 Quick Setup Wizard

## What to build

The last-resort detection strategy: port scanning (FR-10.7). When all other detection strategies (bundled database, DNS, ISPDB, vendor autodiscovery) fail to produce settings, the wizard attempts connections to well-known IMAP/SMTP ports on the MX host or the domain itself.

**Behavior:**
- Attempt connections to standard IMAP ports (993/TLS, 143/STARTTLS) and SMTP ports (465/TLS, 587/STARTTLS) on the MX host or the domain.
- Produce a candidate with the lowest confidence score of all strategies (FR-11, Design Note N-3).

**Security considerations (Open Question OQ-2):**
- Port scanning sends the user's password to whatever server answers on standard ports. The epic flags this as a safety concern.
- The progress feedback (slice 6) should indicate that port scanning is in progress.

## Acceptance criteria

- [ ] When all other detection strategies fail, port scanning is attempted as a last resort (FR-10.7)
- [ ] Standard IMAP and SMTP ports are probed on the MX host or domain
- [ ] Port-scan candidates have the lowest confidence score (FR-11)
- [ ] Progress feedback indicates port scanning is in progress (FR-14)

## Blocked by

- Blocked by 2-bundled-provider-database

## User stories addressed

- US-8 (discover settings as a last resort when other strategies fail)

## Notes

- Open Question OQ-2 in the epic asks whether the wizard should warn the user before port scanning or require explicit opt-in, because the password is sent to whatever server answers. This is an unresolved design decision. The initial implementation should document the risk and implement whichever approach is chosen. This may warrant a HITL decision before implementation begins.
