# Account-Scoped Autocomplete

## Parent Feature
#4.11 Recipient Autocomplete

## User Story
As a multi-account user, I want the option to restrict autocomplete suggestions to only contacts associated with the currently selected sending account, so that I do not accidentally address someone from the wrong account.

## Blocked by
- `1-local-contact-store-with-sent-mail-learning`
- `2-basic-autocomplete-from-sent-contacts`

## Acceptance Criteria
- A "suggest from current account only" toggle is available (default: off).
- When enabled, only contacts associated with the currently selected sending account appear in suggestions.
- When disabled (default), suggestions draw from contacts across all accounts.
- Changing the sending account while composing updates the suggestion pool accordingly (when account scoping is enabled).
- Account scoping works correctly with all data sources (sent contacts, received contacts, system address book).

## HITL/AFK Classification
**AFK** — behaviour is well-defined; this is a query-filter concern.

## Notes
- N-7 in the epic explains per-account contact storage: the same external person may have multiple entries (one per account), which is intentional for this feature.
- FR-35/FR-36 define the behaviour.
