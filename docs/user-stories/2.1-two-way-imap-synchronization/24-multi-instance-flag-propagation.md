# Multi-Instance Flag Propagation

## Parent Feature
#2.1 Two-Way IMAP Synchronization

## User Story
As a user with a label-based mail system (where the same message appears in multiple folders simultaneously), when I change a flag on one instance of a message, I want that flag change propagated to all instances on the server.

## Blocked by
3-flag-unflag-local-to-server, 4-server-flag-change-detection

## Acceptance Criteria
- When a message exists in multiple folders (as with label-based systems), a flag change on one instance is propagated to all other instances on the server (FR-35).
- The application identifies related instances by message identifier (Message-ID header or equivalent).
- After propagation, all instances of the message show the same flag state.
- If one instance is in a read-only folder, that instance is skipped (no error, just not updated).

## HITL / AFK
**AFK** — transparent propagation.

## Estimation
Medium — requires cross-folder message identification and multi-target flag operations.

## Notes
- FR-35 is the primary driver.
- OQ-4 (keyword/label sync for label-based providers) is closely related. This story covers flag propagation across instances; full label-management semantics may require additional work.
- Uncertainty: it is unclear how common this scenario is on standard IMAP servers vs. label-based providers (e.g. Gmail). The implementation priority may depend on target provider support.
