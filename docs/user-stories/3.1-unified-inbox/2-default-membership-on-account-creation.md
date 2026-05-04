# Default Membership on Account Creation

## Parent Feature

#3.1 Unified Inbox

## What to build

When a new account is added, automatically set unified-inbox membership to **enabled** for the folder identified as the account's Inbox, and **disabled** for every other folder (Sent, Drafts, Trash, Spam, Archive, Outbox, user folders) per FR-5 and FR-6. When a new folder is later discovered on the server, default its membership to disabled unless it is detected as the account's Inbox and the account did not already have one (FR-7). Optionally, provider-specific Inbox-equivalent folders may default to enabled per FR-8.

This slice ensures that a brand-new user sees their mail in the Unified Inbox immediately after account setup, without any manual configuration (US-1, US-2).

## Acceptance criteria

- [ ] After adding a new account, the account's Inbox folder has unified-inbox membership = true.
- [ ] After adding a new account, all non-Inbox folders (Sent, Drafts, Trash, Spam, Archive, Outbox, user folders) have unified-inbox membership = false (AC-2).
- [ ] When a second account is added, its Inbox is also unified by default (US-2).
- [ ] A newly discovered server folder defaults to unified = false unless it is the account's first Inbox.
- [ ] Provider-specific Inbox-equivalent folder handling is documented if implemented.

## Blocked by

- Blocked by `1-folder-membership-state`

## User stories addressed

- US-1 (newcomer sees Inbox in Unified Inbox after first account)
- US-2 (additional account Inbox auto-unified)
- US-3 (only Inbox unified by default)

## Notes

Open question OQ-1 in the epic asks which provider-specific folders should be treated as Inbox-equivalent for FR-8. This story should implement the mechanism but may defer the canonical provider list to a follow-up decision. Record any uncertainty as a code comment or design note.
