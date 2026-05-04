# Search Preference Persistence

## Parent Feature
#5.1 Server-Side IMAP Search

## User Story
As any user, I want the search dialog to remember my last-used settings (which fields were enabled, whether trash/junk was included, whether device-first was selected) and my most recent search queries, so that I do not have to reconfigure the dialog each time I search.

## Blocked by
3-search-field-toggles

## Acceptance Criteria
- The search dialog persists the user's last-used settings for each toggleable criterion (senders, recipients, subject, keywords, body, include trash, include junk, device-first) across application restarts.
- The application persists the user's most recent search queries (at least three) across sessions.
- The most recent queries appear as quick-access buttons in the search dialog.
- Quick-access buttons are available after application restart.
- Tapping a quick-access button populates the query field with that search text.

## Mapping to Epic
- US-23, US-24, US-3
- FR-2 (quick-access buttons), FR-27, FR-28
- AC-15, AC-16

## Notes
- This slice depends on #3 (field toggles) because the persisted settings include the per-field toggles introduced in that slice.
- Epic Design Note N-7 explains the rationale: search behavior is highly personal and repetitive, so remembering preferences reduces friction.
