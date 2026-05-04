# Protocol-Aware Operation Routing

## Parent Feature
#7.5 Offline Storage and Operations

## User Story
As any user with a POP3 account, I want operations that have no server-side representation (flags, keywords, mark-read) to work locally without errors, so that my workflow is consistent regardless of the account protocol.

## Acceptance Criteria
- For protocols with limited server-side capability (e.g. POP3), operations that have no server-side effect execute locally only, updating the local state without attempting server communication.
- The user experiences no errors or failures for locally-executed operations — they simply "work."
- Locally-only operations are not placed in the replay queue (since there is nothing to replay).
- The behavior is transparent: there is no explicit mode switch or protocol selection by the user.

## Complexity
Small

## Blocked by
1-persist-operation-record
2-optimistic-ui-update

## HITL/AFK
AFK

## Notes
- OQ-5 in the epic asks whether the user should be informed that POP3 actions won't persist if they switch clients. The initial implementation follows the source application's approach: silent local execution. This can be revisited if users report confusion.
- Design Note N-6 explains the rationale: route unsupported operations to local-only execution rather than failing them. This is transparent to the user.
