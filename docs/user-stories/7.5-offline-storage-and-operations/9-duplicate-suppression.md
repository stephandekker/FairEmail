# Duplicate Operation Suppression

## Parent Feature
#7.5 Offline Storage and Operations

## User Story
As any user, when I perform the same action on a message multiple times in quick succession (e.g. toggling a flag on and off), I want only the final state to be sent to the server — not a sequence of redundant operations — so that replay is efficient and the server is not burdened with unnecessary commands.

## Acceptance Criteria
- If multiple consecutive identical operations are queued for the same message (same type, same arguments), the system suppresses the duplicates and executes only one.
- Performing the same action on a message multiple times in quick succession (e.g. toggling a flag) results in only the final state being sent to the server.
- Suppression applies only to truly redundant operations (same type, same target, same arguments) — operations that differ in any argument are not suppressed.

## Complexity
Small

## Blocked by
1-persist-operation-record

## HITL/AFK
AFK

## Notes
- The epic says "consecutive identical operations" — this implies that if a flag and unflag are queued, both could be suppressed (net no-op). The implementation needs to determine whether to coalesce toggle pairs into nothing, or simply deduplicate exact duplicates. AC-18 in the epic says "only the final state being sent" which suggests coalescing to net effect.
