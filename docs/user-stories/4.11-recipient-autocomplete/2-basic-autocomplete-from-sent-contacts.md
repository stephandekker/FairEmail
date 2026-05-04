# Basic Autocomplete from Sent Contacts

## Parent Feature
#4.11 Recipient Autocomplete

## User Story
As any user, when I type two or more characters into any recipient field (To, Cc, or Bcc), I want a dropdown of matching suggestions drawn from my sent-contact history to appear, so that I can select a recipient without typing the full address.

## Blocked by
- `1-local-contact-store-with-sent-mail-learning`

## Acceptance Criteria
- After typing at least 2 characters into the To, Cc, or Bcc field, a suggestion dropdown appears below (or above, if space is constrained) the active field.
- The dropdown updates in real time as the user continues typing, narrowing results with each additional character.
- Clearing the input below 2 characters dismisses the dropdown.
- Matching is case-insensitive substring search against both the display name and the email address.
- Special characters in the search input are treated literally (no regex/glob interpretation).
- Each suggestion displays the contact's display name (or placeholder if unknown), email address, and avatar (if available).
- The user can select a suggestion via click or keyboard (arrow keys + Enter); Escape dismisses the dropdown.
- Upon selection, the address is inserted into the field formatted as an RFC 5322 address (display name + email).
- After insertion, the cursor advances so the user can immediately type the next recipient.
- The user can also type a full address manually without selecting a suggestion and have it accepted.
- The dropdown is navigable by keyboard and has appropriate screen-reader labels.

## HITL/AFK Classification
**HITL** — the dropdown positioning, visual design, and keyboard interaction patterns will benefit from human review and manual testing.

## Notes
- This story delivers the core end-to-end autocomplete experience using only sent contacts (the default-enabled source per FR-8a). Received contacts and system address-book sources are added in later stories.
- Ranking in this story can use a simple default order (e.g. frequency descending). The full ranking-mode system is delivered in story 7.
- OQ-2 (suggestion list size limit) is relevant here. This story should implement a reasonable default limit (e.g. 50 visible suggestions) and note OQ-2 for future refinement.
- OQ-3 (configurable threshold) is out of scope for this story; the threshold is fixed at 2 characters.
