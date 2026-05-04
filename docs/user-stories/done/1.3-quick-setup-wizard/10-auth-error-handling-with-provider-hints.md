## Parent Feature

#1.3 Quick Setup Wizard

## What to build

User-friendly error handling for authentication failures, with provider-specific guidance. When a connectivity check fails, the wizard provides clear, non-technical explanations and actionable next steps (FR-21, FR-25).

**Authentication errors (FR-21):**
- Display a user-friendly error message on auth failure.
- If the provider entry indicates app-specific passwords are typically required (FR-15k), include that hint prominently and link to the provider's app-password setup page if available.

**Outlook/Hotmail/Live specific (FR-22):**
- For Outlook, Hotmail, or Live domains, display provider-specific guidance about modern authentication requirements or app passwords.

**Provider documentation links (FR-24):**
- On any error, offer a link to provider-specific documentation (from the provider entry) and a general support/FAQ link.

**Non-technical language (FR-25):**
- Error messages are non-technical by default. Raw server error strings are not shown directly — a "show details" expansion may be offered for power users.

## Acceptance criteria

- [ ] Authentication failure shows a user-friendly, non-technical error message (FR-25)
- [ ] If the provider entry flags app-specific password as required, the error includes that hint (FR-21)
- [ ] A link to the provider's app-password documentation is shown when available (FR-21)
- [ ] Entering an Outlook.com/Hotmail/Live address triggers provider-specific guidance (AC-2, FR-22)
- [ ] A link to provider-specific documentation is shown when available (FR-24)
- [ ] A general support/FAQ link is shown on any error (FR-24)
- [ ] Raw server error strings are hidden by default, with a "show details" option (FR-25)
- [ ] Error messages and links are accessible via keyboard and screen reader (NFR-8)

## Blocked by

- Blocked by 7-imap-connectivity-check
- Blocked by 8-smtp-connectivity-check

## User stories addressed

- US-15 (clear auth error explanation with app-password guidance)
- US-16 (app-specific password hint from provider entry)
- US-17 (Outlook/Hotmail/Live provider-specific guidance)
- US-19 (provider documentation and FAQ links from error screen)
