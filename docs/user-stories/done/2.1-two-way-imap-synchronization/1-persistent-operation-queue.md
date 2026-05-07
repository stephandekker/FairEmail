# Persistent Operation Queue

## Parent Feature
#2.1 Two-Way IMAP Synchronization

## User Story
As any user, I want every local action I perform (mark read, flag, move, delete) to be recorded in a durable queue that survives application crashes, forced quits, and system reboots, so that no action I take is ever silently lost.

## Blocked by
_(none — this is the foundation)_

## Acceptance Criteria
- A local action (e.g. mark-read) creates a discrete operation record in a persistent store before the local UI state is updated.
- The operation record includes: operation type, target message identifier, target folder, parameters (e.g. flag name), creation timestamp, and status (queued).
- The operation store survives application restart — after a crash, previously queued operations are still present and in the correct order.
- Operations targeting the same message preserve the order in which the user performed them.
- Operations targeting different messages may be stored independently (no ordering constraint between them).
- The queue can hold at least 1,000 pending entries without degradation.

## HITL / AFK
**AFK** — no human review needed; this is an internal persistence layer with no user-facing decisions.

## Estimation
Medium — involves designing the operation data model and persistence mechanism, but no network I/O.

## Notes
- FR-2, FR-3, FR-19 and NFR-2, NFR-3, NFR-6 are the primary requirements driving this story.
- N-2 (operation priority ordering) mentions content-fetching vs. state-change priority. This story establishes the queue structure; priority ordering is addressed in a later story (19-operation-batching-and-priority).
