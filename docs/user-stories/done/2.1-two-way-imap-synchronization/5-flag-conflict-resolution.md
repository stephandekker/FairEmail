# Flag Conflict Resolution

## Parent Feature
#2.1 Two-Way IMAP Synchronization

## User Story
As a multi-device user, if I change a flag locally while another client simultaneously changes the same flag on the server, I want the application to resolve this deterministically — my most recent local intent wins — without data loss or inconsistency.

## Blocked by
4-server-flag-change-detection

## Acceptance Criteria
- When a server-side flag change is detected for a message that has a pending local operation of the same type, the local pending operation takes precedence — the server change does not overwrite the user's intent.
- When no pending local operation exists for a given flag, the server state is accepted (covered by story 4, but validated here in combination).
- If a message is flagged locally while a pending unflag from a previous session exists, the most recent local action wins (the flag is set on the server).
- Conflict resolution never results in data loss — in ambiguous cases, the application prefers to keep the message rather than delete it.
- The dual-state model (confirmed server state vs. user-intended state) is maintained consistently.

## HITL / AFK
**AFK** — deterministic rule-based resolution, no user input needed.

## Estimation
Small — the logic is straightforward given the dual-state model; the complexity is in ensuring correctness.

## Notes
- US-17, FR-14, FR-15, FR-17, AC-13, AC-14 are the primary drivers.
- N-1 (dual-state fields) is the key design note here.
