# System Native Contact Picker

## Parent Feature
#4.11 Recipient Autocomplete

## User Story
As any user, I want to invoke the system's native contact picker to select a contact as an alternative to typing and using autocomplete, so that I can browse my address book visually when I do not remember a name or address.

## Blocked by
- `8-system-address-book-integration`

## Acceptance Criteria
- A button or affordance is available in the compose view that opens the system's native contact picker.
- When the user selects a contact in the picker, the chosen address is inserted into the active recipient field.
- If the user cancels the picker, no change is made to the recipient field.
- The picker button is only shown when system address-book access has been granted; otherwise it is hidden.

## HITL/AFK Classification
**HITL** — the integration mechanism (EDS, KAddressBook, XDG portal) needs design-time decisions, and the UX of the picker button placement needs review.

## Notes
- OQ-7 in the epic flags that the exact integration path on Linux needs to be determined. This may use the same mechanism as story 8 or a different portal-based API.
- FR-39 defines this requirement.
