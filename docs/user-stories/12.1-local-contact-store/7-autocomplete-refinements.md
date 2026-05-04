## Parent Feature

#12.1 Local Contact Store

## What to build

Extend autocomplete with three refinements:

1. **Account scoping.** The user can configure whether autocomplete suggestions are restricted to contacts associated with the current composing account or span all accounts (FR-17).

2. **Source filtering.** The user can configure whether autocomplete draws from sent-to contacts only, received-from contacts only, or both (FR-16).

3. **Deduplication.** When the same email address exists as both a sent-to and a received-from contact, autocomplete presents it as a single entry with merged frequency data (FR-18).

These are user-configurable preferences that affect autocomplete query behavior.

## Acceptance criteria

- [ ] With account-scoped autocomplete enabled, composing from Account A shows only Account A's contacts (AC-8)
- [ ] With "suggest from sent-to" disabled, sent-to contacts do not appear in autocomplete (AC-7)
- [ ] With "suggest from received-from" disabled, received-from contacts do not appear in autocomplete (AC-7)
- [ ] The same email address appearing as both sent-to and received-from shows as a single autocomplete entry with combined frequency (AC-9)
- [ ] All three settings are persisted as user preferences

## Blocked by

- Blocked by 6-favorite-and-ignored-states (depends on full autocomplete ranking being in place)

## User stories addressed

- US-10 (account-scoped autocomplete)
- US-11 (deduplicate sent-to / received-from in autocomplete)
- US-12 (control autocomplete sources: sent, received, or both)

## Notes

The epic's design note N-2 explains that sent-to and received-from are stored as separate records. Deduplication happens at query time, not storage time. The merged entry should use the combined times-contacted and the most recent last-contacted from either record.

Open question OQ-2 (cross-account merging) is relevant here. This story implements the preference toggle for account scoping as described in the epic, but the interaction with identity inference (OQ-3) is left for future work.

Open question OQ-8 (suggest-received default) is also relevant. The epic's design note N-5 says the source application defaults "suggest from received" to off. This story should implement that default but the team may want to revisit for desktop.
