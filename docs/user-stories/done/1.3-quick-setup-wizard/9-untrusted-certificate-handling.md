## Parent Feature

#1.3 Quick Setup Wizard

## What to build

When the IMAP or SMTP server presents a certificate that is not trusted by the system trust store, the wizard shows the certificate details and allows the user to make an informed decision about whether to trust it (FR-19).

**Behavior:**
- Display the certificate's fingerprint and the DNS names it covers (FR-19a).
- Visually highlight any mismatch between the certificate's DNS names and the server hostname (FR-19b).
- Allow the user to accept the certificate (by fingerprint) for this account (FR-19c).
- If the user accepts, retry the connection using the accepted fingerprint (FR-19d).
- Store the accepted certificate fingerprint with the account for future connections (FR-20).

This applies to both IMAP and SMTP connections.

## Acceptance criteria

- [ ] When the server presents an untrusted certificate, the wizard displays the certificate fingerprint (AC-8)
- [ ] The DNS names covered by the certificate are displayed (AC-8, FR-19a)
- [ ] A mismatch between certificate DNS names and server hostname is visually highlighted (FR-19b)
- [ ] The user can accept the certificate for this account (AC-8, FR-19c)
- [ ] After accepting, the connection is retried and succeeds (AC-8, FR-19d)
- [ ] The accepted fingerprint is stored with the account for future connections (FR-20)
- [ ] The certificate dialog is keyboard-navigable and screen-reader accessible (NFR-8)

## Blocked by

- Blocked by 7-imap-connectivity-check

## User stories addressed

- US-13 (untrusted certificate review with fingerprint and DNS names)

## Notes

- Open Question OQ-5 in the epic asks whether accepted certificates should have an expiry or periodic re-prompt. This is unresolved. The initial implementation should store the fingerprint permanently (matching the source application's behavior) and flag this as a future consideration.
