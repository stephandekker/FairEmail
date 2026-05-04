# Gmail Raw Search

## Parent Feature
#5.1 Server-Side IMAP Search

## User Story
As a Gmail user, I want to use a `raw:` prefix in my search query to pass Gmail's native query operators directly to the server, so that I can leverage Gmail-specific search features like `raw:has:attachment larger:5M`.

## Blocked by
2-server-side-search-single-folder

## Acceptance Criteria
- For Gmail accounts, a `raw:` prefix in the query passes the remainder of the query directly to the server using Gmail's X-GM-RAW extension.
- Gmail raw search is available when searching the Archive folder (Gmail's "All Mail").
- Gmail raw search is available when searching from the Unified Inbox and only one Gmail account is configured.
- A `raw:has:attachment larger:5M` query returns results matching Gmail's native interpretation.
- When the `raw:` prefix is used with a non-Gmail account, the application ignores the prefix and performs a standard search, or informs the user that raw search is only available for Gmail.
- The application checks for the X-GM-EXT-1 capability before attempting raw search.

## Mapping to Epic
- US-14, US-15
- FR-13, FR-14, FR-15
- AC-8

## Notes
- The codebase shows this uses `GmailRawSearchTerm` and checks for `X-GM-EXT-1` capability. The epic is authoritative on the scoping rules (Archive folder, single-Gmail Unified Inbox).
- Open question OQ-4 in the epic asks whether `raw:` should be restricted to the Archive folder. The current implementation and epic both describe this restriction. This story follows the epic's specification; any relaxation would be a separate future decision.
