# Offline Behavior

## Parent Feature
#5.1 Server-Side IMAP Search

## User Story
As any user, when my application is offline, I want the server-search option to be unavailable and the search dialog to allow only local search, so that I am not offered an action that will fail, and I can still search my local cache.

## Blocked by
2-server-side-search-single-folder

## Acceptance Criteria
- When the application is offline, the server-search toggle is disabled or hidden.
- The search dialog still allows local search when offline.
- If connectivity is lost mid-search, the application displays an appropriate error and retains any partial results already loaded.
- When connectivity is restored, the server-search option becomes available again.

## Mapping to Epic
- NFR-4
- AC-17

## Notes
- This slice is small but important for robustness. It ensures the search feature degrades gracefully in the absence of network connectivity, complementing the server-side capability degradation in slice #10.
