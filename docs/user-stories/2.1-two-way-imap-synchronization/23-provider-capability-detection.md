# Provider Capability Detection and Adaptation

## Parent Feature
#2.1 Two-Way IMAP Synchronization

## User Story
As any user, I want the application to detect my server's capabilities at connection time and adapt its behavior accordingly, so that it works correctly on all IMAP servers — using optimizations when available and degrading gracefully when they are not.

## Blocked by
6-move-message-local-to-server, 8-permanent-delete-expunge, 16-incremental-sync-modseq

## Acceptance Criteria
- At connection time, the application queries and stores the server's advertised capabilities (e.g. MOVE, UIDPLUS, CONDSTORE, QRESYNC, user-defined flags).
- Behavior adapts based on detected capabilities: atomic move vs. copy+delete, per-UID expunge vs. folder-wide, MODSEQ vs. full comparison, user keywords vs. local-only.
- The application does not require any optional IMAP extension to function (FR-47, AC-23).
- A compatibility profile mechanism exists to disable specific optimizations for providers known to implement them incorrectly (FR-46).
- The compatibility profile handles known provider-specific quirks (e.g. providers that advertise MOVE but implement it incorrectly).

## HITL / AFK
**AFK** for detection and adaptation; **HITL** if the compatibility profile is user-editable (OQ-2 leaves this open).

## Estimation
Medium — capability detection is straightforward; the compatibility-profile mechanism and quirk handling add complexity.

## Notes
- FR-45, FR-46, FR-47, AC-23 are the primary drivers.
- OQ-2 asks whether the compatibility profile should be user-editable or developer-maintained. The source application hard-codes provider exceptions. This decision should be made during design.
- G6 (support the full range of IMAP servers with graceful degradation) is the overarching goal.
