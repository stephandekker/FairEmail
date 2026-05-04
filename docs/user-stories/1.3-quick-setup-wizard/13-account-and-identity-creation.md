## Parent Feature

#1.3 Quick Setup Wizard

## What to build

When the user confirms on the review screen, the wizard creates both the incoming-mail account and the sending identity in a single step (FR-27).

**Account creation (FR-27a):**
- An incoming-mail account with the detected IMAP settings, credentials, and provider-specific tuning parameters (keep-alive interval, NOOP flag, partial-fetch flag, TLS ceiling, certificate fingerprint if accepted).

**Identity creation (FR-27b):**
- A sending identity with the detected SMTP settings, the user's display name, email address, credentials, and the outgoing server's maximum message size.

**Primary account designation (FR-28):**
- The first account created is designated as the primary account.
- Subsequent accounts do not override the primary designation.

Provider-specific tuning parameters from the provider database entry (keep-alive, NOOP, partial-fetch, TLS ceiling — FR-15g through FR-15j) are applied silently and invisibly to the user (Design Note N-7).

## Acceptance criteria

- [ ] Confirming save creates an incoming-mail account with detected IMAP settings and credentials (FR-27a)
- [ ] Provider-specific tuning parameters are applied to the account (FR-27a, Design Note N-7)
- [ ] A sending identity is created with detected SMTP settings, display name, and email (FR-27b)
- [ ] The outgoing server's max message size is stored with the identity (FR-27b)
- [ ] If accepted, the certificate fingerprint is stored with the account (FR-27a)
- [ ] The first account created is designated as primary (AC-11, FR-28)
- [ ] A second account does not override the primary designation (AC-11, FR-28)
- [ ] Both account and identity are created in a single user action (US-21)

## Blocked by

- Blocked by 12-account-review-screen

## User stories addressed

- US-21 (create account and identity in one step)
- US-22 (first account is primary)
- US-29 (provider-specific tuning applied automatically)
