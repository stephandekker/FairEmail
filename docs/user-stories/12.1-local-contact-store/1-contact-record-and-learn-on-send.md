## Parent Feature

#12.1 Local Contact Store

## What to build

Establish the local contact store and implement automatic contact learning when the user sends a message. This is the foundational tracer bullet: after sending an email, all recipient addresses (To, Cc, Bcc) are recorded as sent-to contacts in the local store, scoped to the sending account.

The contact record must store at minimum: associated account, contact type, email address, optional display name, optional group label, times-contacted counter, first-contacted timestamp, last-contacted timestamp, and state (default/favorite/ignored — initialised to default). The uniqueness constraint (account + type + email) must be enforced at the storage level (see epic FR-1, FR-2, FR-5 for data integrity).

On send, each recipient address is either created (times-contacted = 1, first-contacted = last-contacted = message timestamp) or updated (increment times-contacted, update last-contacted if more recent, preserve first-contacted). If the existing contact has an empty name, fill it from the message headers if available (FR-6, FR-7, FR-8).

Addresses matching well-known non-personal patterns (no-reply, mailer-daemon, postmaster, and similar) must be skipped (FR-10). Messages in Drafts, Trash, or Spam folders must not trigger contact creation or update (FR-9).

Optionally, the contact record may store a reference to the sending identity used, to support future identity inference (FR-3).

## Acceptance criteria

- [ ] A contact record structure exists with all fields specified in FR-1
- [ ] The uniqueness constraint (account + type + email) is enforced at the storage level (NFR-5)
- [ ] After sending a message to a new address, a sent-to contact appears with times-contacted = 1 and correct timestamps (AC-1)
- [ ] After sending a second message to the same address, times-contacted increments to 2, last-contacted updates, first-contacted is unchanged (AC-3)
- [ ] Addresses matching no-reply patterns (e.g. noreply@example.com, mailer-daemon@…) are not recorded (AC-10)
- [ ] Messages in Drafts, Trash, or Spam folders do not create or update contacts (AC-11)
- [ ] If a contact's name is empty and the message headers contain a display name, the name is populated on update (FR-8)
- [ ] Contacts are scoped to the sending account (FR-4)

## Blocked by

None — can start immediately

## User stories addressed

- US-1 (auto-record recipients on send)
- US-3 (contacts scoped to account)
- US-4 (update existing contact, no duplicates)
- US-5 (skip non-personal addresses)
- US-6 (skip system folders)
