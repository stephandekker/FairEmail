# Continue Searching on Server (Escalation)

## Parent Feature
#5.1 Server-Side IMAP Search

## User Story
As any user, after a local search returns results, I want a clearly labeled action to "continue searching on the server" that lets me pick a specific account and folder, so that I can escalate to server search when local results are incomplete without re-entering my query.

## Blocked by
2-server-side-search-single-folder

## Acceptance Criteria
- After any search completes (local or server), the application offers an action to "continue searching on the server."
- The action allows the user to select a specific account and folder for the server-side search.
- The same query text and criteria are carried over to the server search without requiring re-entry.
- Server results for the selected folder are displayed using the standard message-list view.
- This escalation path works from the Unified Inbox and account-level views (where initial search was local-only).

## Mapping to Epic
- US-6
- FR-6
- AC-11

## Notes
- This is the bridge between local and server search that makes the "local-first, server-second" design (epic Design Note N-1) work in practice.
- Open question OQ-1 in the epic discusses whether multi-folder server search should sequentially query each folder. This story follows the epic's current specification: escalation is always to a single user-selected folder.
