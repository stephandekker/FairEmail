# Unavailable-Offline Indication for Undownloaded Messages

## Parent Feature
#7.5 Offline Storage and Operations

## User Story
As any user, when I attempt to open a message whose body has not been downloaded while I am offline, I want a clear indication that the content is unavailable offline rather than a blank or broken view, so that I understand the limitation.

## Acceptance Criteria
- Messages whose body has not been downloaded are clearly indicated in the message list UI (e.g. a visual marker distinguishing them from fully cached messages).
- Attempting to open an undownloaded message while offline presents an informative message (e.g. "Message body not available offline") rather than a blank or error state.
- The indication is visually distinct and immediately understandable without requiring user action.
- When connectivity returns, the user can open the message normally (triggering a download).

## Complexity
Small

## Blocked by
11-local-body-and-attachment-storage

## HITL/AFK
AFK

## Notes
- The specific visual treatment is not prescribed by the epic — only that it must be "clear" and not blank/broken. Design can choose the appropriate indicator.
