# Suggestion Ranking Modes

## Parent Feature
#4.11 Recipient Autocomplete

## User Story
As any user, I want to choose between frequency-based ranking and source-priority-based ranking for autocomplete suggestions, so that I can pick the ordering model that suits my workflow.

## Blocked by
- `2-basic-autocomplete-from-sent-contacts`

## Acceptance Criteria
- Two ranking modes are available, selectable by the user via a "suggest frequently used" toggle:
  - **Frequency/recency mode** (toggle on): suggestions ordered by number of interactions (descending), then by most recent interaction (descending), then alphabetically.
  - **Source-priority mode** (toggle off, the default): contacts with a known display name ranked above email-only entries; then alphabetical by name; then alphabetical by email. When system address-book contacts are present (added in story 8), they rank above local contacts. Starred/favorite system contacts rank at the top regardless of other factors.
- Alphabetical ordering in both modes is locale-aware and accent-insensitive.
- A contact with 50 interactions appears above a contact with 2 interactions in frequency/recency mode, all else being equal.
- In source-priority mode, a contact with a display name appears above an email-only contact, all else being equal.

## HITL/AFK Classification
**AFK** — ranking logic is algorithmic; no UX review needed beyond verifying the sort order in tests.

## Notes
- The starred-contacts boost in source-priority mode (FR-19b, N-5) will only be observable once system address-book integration is added (story 8). This story should implement the ranking framework so that starred contacts are handled correctly when that source is added.
- N-4 in the epic explains the design rationale for offering two modes.
