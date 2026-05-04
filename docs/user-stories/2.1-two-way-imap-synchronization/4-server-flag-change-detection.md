# Server Flag Change Detection

## Parent Feature
#2.1 Two-Way IMAP Synchronization

## User Story
As a multi-device user, when I (or another client, or a server-side rule) change a message's flags on the server, I want the desktop client to detect that change on its next sync cycle and update my local view automatically.

## Blocked by
2-mark-read-local-to-server

## Acceptance Criteria
- During a sync cycle, the application compares server-side flags for messages within the sync window against locally-known "confirmed server state."
- When a flag has changed on the server and no pending local operation exists for that flag on that message, the local state is updated to match the server.
- A message marked read via webmail appears as read in the desktop client after the next sync cycle.
- A message flagged via another client appears flagged locally after the next sync cycle.
- Flag changes made by server-side rules (e.g. Sieve) are detected and applied locally.
- Changes to the Seen, Flagged, Answered, Deleted, and Draft flags are all detected.

## HITL / AFK
**AFK** — automated detection with no user decisions required.

## Estimation
Medium — requires implementing the sync-cycle flag comparison logic.

## Notes
- US-9, US-12, US-13, FR-5, FR-8, FR-15, AC-6, AC-14 are the primary drivers.
- This story does NOT cover the conflict case (pending local operation exists) — that is story 5.
- N-1 (dual-state fields) describes the "confirmed server state" vs "user-intended state" pattern that this story relies on.
