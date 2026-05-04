# Batch Replay of Similar Operations with Fallback

## Parent Feature
#7.5 Offline Storage and Operations

## User Story
As any user, I want the replay process to batch similar operations together where possible (e.g. marking 50 messages as read in one server exchange), so that replay completes quickly even after a long offline session — and if a batch fails, I want operations retried individually so a single problematic message doesn't block the rest.

## Acceptance Criteria
- When multiple operations of the same type target different messages in the same folder (e.g. 20 mark-as-read operations), they are batched into a single server exchange where the protocol supports it.
- The batch size is bounded by a configurable upper limit to prevent excessively large server commands.
- If a batch is rejected by the server, the system falls back to smaller batches or individual execution.
- When a batched group fails, operations are retried individually to isolate the specific failing operation(s).
- After an extended offline period (100+ queued operations), replay completes within a reasonable time by batching.

## Complexity
Medium

## Blocked by
4-operation-priority-ordering

## HITL/AFK
AFK

## Notes
- OQ-2 in the epic asks whether batch size should be configurable or auto-tuned. For the initial implementation, a configurable upper limit (with a sensible default) is sufficient.
- Design Note N-4 describes the "batch-then-fallback" strategy from the source application. This two-phase approach must be preserved.
