## Parent Feature

#1.3 Quick Setup Wizard

## What to build

After a successful connectivity check (both IMAP and SMTP), the wizard presents a review screen where the user can confirm the detected configuration before saving (FR-26).

**Review screen contents:**
- The provider name and account name (FR-26a).
- The detected system folders (Inbox, Drafts, Sent, Trash, Spam, Archive) with indicators for which were found on the server (FR-26b).
- An editable account name field (FR-26c).

The user confirms to proceed to account creation or goes back to modify inputs.

## Acceptance criteria

- [ ] After successful IMAP and SMTP checks, a review screen is presented (FR-26)
- [ ] The provider name is displayed (FR-26a)
- [ ] The account name is displayed and editable (FR-26a, FR-26c)
- [ ] Detected system folders are listed with indicators for which were found (FR-26b)
- [ ] The user can confirm to save or go back to modify inputs
- [ ] The review screen is keyboard-navigable and screen-reader accessible (NFR-8)

## Blocked by

- Blocked by 7-imap-connectivity-check
- Blocked by 8-smtp-connectivity-check

## User stories addressed

- US-20 (review detected configuration including folders before save)
