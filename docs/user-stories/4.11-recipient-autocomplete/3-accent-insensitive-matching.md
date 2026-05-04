# Accent-Insensitive Matching

## Parent Feature
#4.11 Recipient Autocomplete

## User Story
As any user, I want autocomplete matching against display names to be accent-insensitive, so that typing "jose" matches "José García" and typing "garcia" matches "García", without needing to type the exact accented characters.

## Blocked by
- `2-basic-autocomplete-from-sent-contacts`

## Acceptance Criteria
- Typing an unaccented string matches contacts whose display names contain the accented equivalent (e.g. "jose" matches "José", "garcia" matches "García").
- Accent-insensitive matching applies to all display-name matching across all data sources (local sent contacts now; received contacts and system address book when added later).
- Email-address matching remains simple case-insensitive substring (email addresses do not contain accents in practice).
- Matching handles Unicode combining characters correctly.
- Alphabetical ordering in suggestion results is locale-aware and accent-insensitive.

## HITL/AFK Classification
**AFK** — the behaviour is well-specified; implementation is a matching-algorithm concern.

## Notes
- OQ-1 in the epic asks whether accent-insensitive matching should be applied uniformly across all sources. The Android codebase applies it for system address-book queries but uses simpler matching for local store queries. This story applies accent-insensitive matching uniformly, as that is the cleaner user experience. If the decision changes, update accordingly.
- FR-6 and FR-20 both require accent-insensitivity. This story covers both matching and sort ordering.
