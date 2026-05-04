# Automatic Sender-Identity Selection

## Parent Feature
#4.11 Recipient Autocomplete

## User Story
As a multi-account user, I want the option to have the application automatically switch my sender identity to match the account that has previously corresponded with the first recipient I select in the To field, so that I send from the right account without manually switching.

## Blocked by
- `1-local-contact-store-with-sent-mail-learning`
- `2-basic-autocomplete-from-sent-contacts`

## Acceptance Criteria
- An "auto-select identity" toggle is available (default: off).
- When enabled, the application observes changes to the first recipient in the To field.
- Upon detecting a known address (one present in the local contact store or enabled data sources), the sender identity switches to the account and identity most recently used to correspond with that address.
- Identity selection consults the same local contact store and the same enabled data sources as autocomplete.
- If the first To recipient is not found in any data source, no identity switch occurs.
- The user can manually override the auto-selected identity after it has been applied.

## HITL/AFK Classification
**HITL** — the interaction between auto-selection and manual override, and the UX of the identity switching, should be reviewed by a human.

## Notes
- OQ-6 in the epic asks whether adding a second recipient from a different account should trigger a warning. This story implements only the first-recipient behaviour as specified in FR-32. The open question should be addressed in a future iteration if needed.
- FR-32/FR-33/FR-34 define the behaviour.
