## Parent Feature

#1.3 Quick Setup Wizard

## What to build

The IMAP portion of the connectivity check: connect to the incoming server using detected settings, authenticate, and enumerate server-side folders with system-folder role detection (FR-17 steps 1-3).

**Behavior:**
- Connect to the IMAP server using the hostname, port, and encryption mode from the detected provider settings (FR-17.1).
- Authenticate using the user's credentials (FR-17.2).
- If the provider entry specifies multiple possible username types (full email, local part, provider template), try each in turn until authentication succeeds or all are exhausted (FR-18).
- On successful authentication, enumerate the available server-side folders and detect system-folder roles: Inbox, Sent, Drafts, Trash, Spam, Archive (FR-17.3).

This slice integrates with the provider database (slice 2) for settings and feeds results into the review screen (slice 12) and account creation (slice 13). It does NOT include SMTP checking, certificate handling, or error-message formatting — those are separate slices.

## Acceptance criteria

- [ ] The wizard connects to the IMAP server using detected hostname, port, and encryption mode
- [ ] Authentication succeeds with valid credentials
- [ ] If the first username format fails, the wizard tries alternative formats before reporting failure (FR-18)
- [ ] Server-side folders are enumerated after successful authentication (FR-17.3)
- [ ] System-folder roles (Inbox, Sent, Drafts, Trash, Spam, Archive) are detected from the folder list
- [ ] The IMAP check result (success/failure, folder list) is available for downstream slices

## Blocked by

- Blocked by 1-wizard-ui-with-input-validation
- Blocked by 2-bundled-provider-database

## User stories addressed

- US-11 (connectivity check tests IMAP authentication)
- US-12 (folder discovery during IMAP test)
- US-14 (multiple username format fallback)
