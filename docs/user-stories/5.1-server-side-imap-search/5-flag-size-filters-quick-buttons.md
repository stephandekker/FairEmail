# Flag Filters, Size Criterion, and Quick-Filter Buttons

## Parent Feature
#5.1 Server-Side IMAP Search

## User Story
As any user, I want to filter search results by message flags (unread, flagged/starred) and by minimum message size, and I want quick-filter buttons for common flag-based searches, so that I can combine text search with non-text criteria or quickly run common filtered searches.

## Blocked by
2-server-side-search-single-folder

## Acceptance Criteria
- The search dialog provides filter options for: flagged/starred only, unread only.
- The search dialog provides a minimum message size filter.
- Flag and size filters are translated to IMAP FLAGGED, UNSEEN, and LARGER criteria for server search.
- Flag and size filters combine with text and date criteria using AND logic.
- The search dialog provides quick-filter buttons for common searches: flagged, unread, hidden, with attachments, with calendar invitations, with notes.
- Quick-filter buttons populate the same criteria model as the advanced options (they are shortcuts, not a separate search path).
- Searching with the flagged filter returns only flagged messages; with the unseen filter, only unread messages.
- Searching with a size criterion returns only messages exceeding the specified size.

## Mapping to Epic
- US-10, US-11
- FR-3, FR-7 (Flagged, Unseen, Size — greater than)
- AC-6, AC-18

## Notes
- Some quick-filter buttons (with attachments, with notes, hidden) correspond to local-only criteria. Their interaction with server search is handled in slice #16.
- Uncertainty: the epic lists quick-filter buttons for "hidden" and "with notes" which are local-only metadata. These buttons should still work for local search; the question of how they interact with server search is deferred to slice #16.
