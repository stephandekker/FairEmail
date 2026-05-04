# Local Message Body and Attachment Storage for Offline Reading

## Parent Feature
#7.5 Offline Storage and Operations

## User Story
As a traveler, I want to read the full body and view attachments of any message that was previously downloaded, without needing a network connection, so that my reading flow is uninterrupted.

## Acceptance Criteria
- Message bodies are stored locally once downloaded and remain readable offline until explicitly purged or until the message leaves the configured retention window.
- Attachments are stored locally once downloaded and remain readable offline.
- A previously downloaded message body is readable offline, including inline images that were fetched at download time.
- A previously downloaded attachment is openable offline.
- Local content persists across application restarts.
- Storage respects the user's configured retention settings (days to keep, days to download) and does not grow unbounded.

## Complexity
Medium

## Blocked by
10-local-message-metadata-storage

## HITL/AFK
AFK

## Notes
- OQ-6 in the epic raises the question of whether a "pin for offline" mechanism should override retention for specific messages. This is out of scope for the initial implementation but worth noting as a potential enhancement.
- The epic distinguishes between bodies and attachments in its FRs (FR-34 and FR-35) but they follow the same pattern (store once downloaded, serve offline, respect retention). They are combined here as one vertical slice since the behavior is the same.
