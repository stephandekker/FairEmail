## Parent Feature

#12.1 Local Contact Store

## What to build

Add the ability to mark a contact as "favorite" or "ignored" and cycle through the three states (default → favorite → ignored → default) from the contact list (FR-24).

Integrate the state into autocomplete ranking: favorites appear first in suggestions (before frequency/recency sorting), and contacts with state "ignored" are excluded entirely (FR-14, FR-15). The full autocomplete sort order becomes: (1) favorite state first, (2) contacts with a known avatar/photo before those without, (3) times-contacted descending, (4) last-contacted descending.

## Acceptance criteria

- [ ] The user can mark a contact as "favorite" from the contact list (US-18)
- [ ] The user can mark a contact as "ignored" from the contact list (US-19)
- [ ] The user can cycle a contact's state: default → favorite → ignored → default (FR-24, US-20)
- [ ] A favorite contact appears before a non-favorite with higher times-contacted in autocomplete (AC-5)
- [ ] An ignored contact does not appear in autocomplete suggestions (AC-6)
- [ ] State changes persist across application restart

## Blocked by

- Blocked by 3-basic-autocomplete (integrates into autocomplete ranking)
- Blocked by 4-contact-list-view-and-search (requires the contact list UI for state cycling)

## User stories addressed

- US-8 (autocomplete ranking: favorites first, then frequency/recency)
- US-9 (ignored contacts excluded from autocomplete)
- US-18 (mark as favorite)
- US-19 (mark as ignored)
- US-20 (cycle state)
