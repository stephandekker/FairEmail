# Incremental Result Loading

## Parent Feature
#5.1 Server-Side IMAP Search

## User Story
As any user, I want search results to load incrementally as I scroll rather than waiting for the entire result set, so that I can start reviewing matches immediately and the application remains responsive even for large result sets.

## Blocked by
2-server-side-search-single-folder

## Acceptance Criteria
- Server search results load incrementally (paged/boundary loading) as the user scrolls.
- The application does not attempt to fetch all results before displaying the first page.
- The first page of results appears as soon as the server returns the first batch of matches.
- The feature handles result sets of up to 100,000 messages without crashing or becoming unresponsive.
- Results are sorted by received date (newest first) by default.
- The user can change the sort order using the same sort options available in any message list.

## Mapping to Epic
- US-21
- FR-23, FR-24
- NFR-1 (results begin appearing immediately), NFR-2 (scale to 100k messages)
- AC-13

## Notes
- The codebase uses `BoundaryCallbackMessages` for paged loading, which aligns with the epic's "boundary loading" terminology.
- Open question OQ-5 in the epic asks whether the 100,000 result cap should be user-configurable and whether the user should be informed when the cap is reached. This story implements the cap as specified; configurability would be a separate decision.
