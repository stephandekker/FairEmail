# Autocomplete Configuration Settings

## Parent Feature
#4.11 Recipient Autocomplete

## User Story
As any user, I want all autocomplete-related settings to be collected in one place under the send/compose settings area, so that I can easily find and adjust my autocomplete preferences.

## Blocked by
- `2-basic-autocomplete-from-sent-contacts`
- `4-chip-rendering`
- `6-received-contacts-source`
- `7-ranking-modes`
- `10-account-scoping`
- `11-auto-identity-selection`
- `13-contact-purge`

## Acceptance Criteria
- The following settings are exposed under the send/compose settings area:
  - **Suggest names**: whether display names are included in suggestions and chips (default: on).
  - **Suggest sent contacts**: whether sent-to addresses appear in suggestions (default: on).
  - **Suggest received contacts**: whether received-from addresses appear in suggestions (default: off).
  - **Suggest frequently used**: whether to rank by frequency/recency rather than source priority (default: off).
  - **Suggest from current account only**: whether to restrict suggestions to the current sending account (default: off).
  - **Auto-select identity**: whether to automatically switch sender identity based on recipient (default: off).
  - **Contact purge age**: how old a contact entry must be before purge eligibility, in months (default: 1).
  - **Contact purge frequency**: minimum interaction count below which an entry is purge-eligible (default: 0).
  - **Show chips**: whether to render recipients as chips or plain text (default: on).
- Each setting has a clear label and, where helpful, a brief description.
- Toggling "suggest names" off causes suggestions and chips to show only email addresses (no display names).
- All settings persist across application restarts.

## HITL/AFK Classification
**HITL** — settings layout, labeling, and grouping should be reviewed by a human for clarity and discoverability.

## Notes
- FR-40 defines the full settings list. Individual stories (6, 7, 10, 11, 13) may implement their own toggles as part of their slice. This story ensures all settings are consolidated in the correct location with consistent presentation.
- The "suggest names" toggle (FR-40a) affects both suggestions and chip labels. When off, chips show only the email address, and suggestions omit the display name.
