# UID Validity Reset and Full Resync

## Parent Feature
#2.1 Two-Way IMAP Synchronization

## User Story
As any user, if my mail server reconstructs a mailbox (invalidating all previously known message UIDs), I want the application to detect this automatically and perform a full resync, so that I do not end up with stale or mismatched messages.

## Blocked by
10-new-message-detection, 11-message-removal-detection

## Acceptance Criteria
- On each sync cycle, the application checks the server's reported UID validity value for the folder.
- If UID validity has changed since the last sync, the application discards all local UID mappings for that folder.
- A full resynchronization is performed automatically without user intervention.
- No messages are duplicated or lost as a result of the resync (AC-20).
- Pending operations targeting messages in the affected folder are re-evaluated (operations targeting now-invalid UIDs are cancelled gracefully).

## HITL / AFK
**AFK** — automatic detection and recovery.

## Estimation
Small — the detection is a simple comparison; the response (discard + full resync) reuses existing sync logic.

## Notes
- US-26, FR-9, AC-20 are the primary drivers.
- N-6 explains that UID validity change is treated as an unconditional full-resync trigger with no heuristic UID matching.
