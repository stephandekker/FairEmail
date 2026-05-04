# Trash and Junk Inclusion Toggle

## Parent Feature
#5.1 Server-Side IMAP Search

## User Story
As any user, I want to choose whether to include messages in the Trash and Junk/Spam folders in my search results, so that deleted or spam messages do not pollute my results by default but are findable when I need them.

## Blocked by
1-search-dialog-local-text-search

## Acceptance Criteria
- The search dialog provides toggles to include or exclude Trash and Junk/Spam folders from search results.
- By default, Trash and Junk/Spam are excluded.
- When included, messages from Trash and Junk/Spam folders appear in both local and server search results.
- The toggles work for both local multi-folder search and server single-folder search contexts.

## Mapping to Epic
- US-12
- FR-10

## Notes
- This slice is independent of server search (#2) because it also applies to local search. It only requires the base search dialog from slice #1.
- For server search in a specific folder, if the user has selected the Trash or Junk folder directly, these toggles are not relevant (the user explicitly chose that folder). The toggles matter primarily for local multi-folder search and for the "continue on server" escalation path.
