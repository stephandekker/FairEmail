# Server-Side IMAP Search for a Single Folder

## Parent Feature
#5.1 Server-Side IMAP Search

## User Story
As any user, when I have selected a specific folder, I want a toggle that lets me search directly on the IMAP server instead of locally, so that I can find messages that are not in my local cache.

## Blocked by
1-search-dialog-local-text-search

## Acceptance Criteria
- When a specific folder is selected, the search dialog shows a toggle to choose between device-first (local) search and direct server search.
- When the toggle is disabled (no specific folder selected, e.g. Unified Inbox or account-level view), only local search is available.
- With server search enabled, submitting a text query sends an IMAP SEARCH command to the server for the selected folder.
- Server search results include messages not present in the local cache.
- Results appear in the same standard message-list view as local results.
- All per-message actions work on server search results.

## Mapping to Epic
- US-4, US-5, US-7
- FR-4, FR-5
- FR-7 (basic text criteria: sender, recipient, subject — default set)
- FR-22, FR-26
- NFR-5
- AC-1

## Notes
- This is the first slice that actually sends an IMAP SEARCH command to the server. It establishes the server communication path that all subsequent server-search slices depend on.
- The default set of fields searched (sender, recipient, subject) is used here; per-field toggles come in slice #3.
- The codebase shows this logic lives primarily in `BoundaryCallbackMessages.search()` which builds IMAP SEARCH commands. The epic is authoritative on behaviour, but this informs granularity.
