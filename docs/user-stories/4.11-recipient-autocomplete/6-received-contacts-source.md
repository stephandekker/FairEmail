# Received Contacts as Autocomplete Source

## Parent Feature
#4.11 Recipient Autocomplete

## User Story
As any user, I want the option to include addresses I have received mail from in autocomplete suggestions, so that I can quickly address people who have contacted me.

## Blocked by
- `1-local-contact-store-with-sent-mail-learning`
- `2-basic-autocomplete-from-sent-contacts`

## Acceptance Criteria
- The local contact store learns addresses from received mail (From, Reply-To headers), stored per-account with direction "received-from".
- Noreply addresses are excluded from received-contact learning.
- Addresses from messages in Drafts, Archive, Trash, or Spam folders are not learned.
- A "suggest received contacts" toggle is available (default: off).
- When the toggle is enabled, received contacts appear in autocomplete suggestions alongside sent contacts.
- When the toggle is disabled, only sent contacts appear (received contacts are excluded).
- Deduplication by email address (case-insensitive) is applied when both sent and received entries exist for the same address.

## HITL/AFK Classification
**AFK** — behaviour is well-defined; this extends the existing learning and query paths.

## Notes
- N-2 in the epic explains why received contacts are off by default: received mail includes mailing lists, automated senders, and potential spam.
- OQ-8 questions whether received contacts should be enabled by default for a desktop client. This story follows the epic (default: off).
