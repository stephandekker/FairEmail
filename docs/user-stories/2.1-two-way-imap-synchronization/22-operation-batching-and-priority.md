# Operation Batching and Priority Ordering

## Parent Feature
#2.1 Two-Way IMAP Synchronization

## User Story
As any user, I want the application to process operations efficiently — batching similar flag changes into single server commands and prioritizing operations that affect what I see (moves, deletes) over non-destructive ones (flags) — so that sync is fast and responsive.

## Blocked by
2-mark-read-local-to-server, 6-move-message-local-to-server

## Acceptance Criteria
- When multiple messages need the same flag change (e.g. "mark 50 messages read"), the operations are batched into chunked server commands rather than sent individually (FR-4).
- Batch chunk size is bounded to limit blast radius if one message in the batch has been expunged.
- Content-fetching operations (download body) are prioritized over state-change operations.
- Within state-change operations, destructive operations (delete, move) are prioritized over non-destructive ones (flag, seen).
- Operations targeting different messages may be parallelized for efficiency.
- A failed batch retries individual operations separately (FR-39, covered in story 15 but validated here in combination).

## HITL / AFK
**AFK** — internal optimization with no user decisions.

## Estimation
Medium — batching logic, chunk sizing, and priority scheduling.

## Notes
- FR-4, FR-39, NFR-3, NFR-4, NFR-6 are the primary drivers.
- N-2 (operation priority ordering) and N-3 (batching similar operations) describe the design rationale.
