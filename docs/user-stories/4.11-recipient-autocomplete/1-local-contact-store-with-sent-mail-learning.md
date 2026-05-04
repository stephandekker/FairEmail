# Local Contact Store with Sent-Mail Learning

## Parent Feature
#4.11 Recipient Autocomplete

## User Story
As any user, I want the application to automatically learn and store the email addresses and display names of people I send mail to, so that autocomplete can suggest them in future compose sessions.

## Blocked by
_(none — this is the foundation story)_

## Acceptance Criteria
- A local contact store exists that persists contact entries with: email address, display name (if known), number of interactions (frequency), timestamp of most recent interaction (recency), associated account, direction (sent-to), and user-assignable state (default, favorite, ignored).
- When the user sends a message, each To/Cc/Bcc address is added to (or updated in) the local contact store, incrementing the interaction count and updating the recency timestamp.
- Contacts are stored per-account and per-direction, so a single external person may have separate entries per account they are reached through.
- Noreply addresses (e.g. `noreply@`, `no-reply@`, and common variants) are automatically excluded from storage.
- Addresses extracted from messages in Drafts, Archive, Trash, or Spam folders are not learned.
- The store operates entirely locally and is never transmitted to any server.
- The store functions fully offline.

## HITL/AFK Classification
**AFK** — no human review needed beyond normal code review; the behaviour is well-defined.

## Notes
- The existing Android codebase uses a Room entity (`EntityContact`) with `times_contacted`, `first_contacted`, `last_contacted`, and contact `type` (TO/FROM/JUNK/NO_JUNK). The desktop implementation should match this data model conceptually but is free to use a different persistence technology.
- FR-15 excludes learning from Archive folders. OQ-4 in the epic questions whether Archive should be included. This story follows the epic as written (exclude Archive); if the decision changes, this story should be updated.
- FR-13 requires per-account, per-direction storage. This is critical for account-scoped autocomplete (story 10) and auto-identity selection (story 11).
