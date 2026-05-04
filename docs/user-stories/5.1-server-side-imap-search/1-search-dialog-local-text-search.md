# Search Dialog with Local Text Search

## Parent Feature
#5.1 Server-Side IMAP Search

## User Story
As any user, I want to open a search dialog from any message-list context (folder, account view, or Unified Inbox), type a free-text query, and see matching messages from my local cache displayed in the standard message list, so that I have a fast baseline search experience.

## Blocked by
_(none — this is the foundation slice)_

## Acceptance Criteria
- A search action is accessible from any message-list context: a specific folder, an account-level view, or the Unified Inbox.
- The search dialog presents a text input field for the query.
- Submitting the query runs a local search across the appropriate scope (selected folder, account folders, or all folders for Unified Inbox).
- Results appear in the standard message-list view with the same display options (density, columns, preview text, account color, threading) as any other folder view.
- All standard per-message actions (reply, forward, move, delete, flag, archive, mark read/unread) are available on messages in the search results.
- The search dialog opens and is ready for input in well under one second.

## Mapping to Epic
- US-1, US-2
- FR-1, FR-2 (text input portion)
- FR-5 (local search for multi-folder contexts)
- FR-22, FR-24, FR-26
- NFR-1, NFR-7
- AC-14 (message actions on results)

## Notes
- This slice establishes the end-to-end search flow: UI dialog -> query submission -> local search execution -> result display in message list. All subsequent slices build on this foundation.
- The "advanced options" section of the dialog is created as an expandable placeholder here but populated by later slices (#3, #4, #5, #6).
