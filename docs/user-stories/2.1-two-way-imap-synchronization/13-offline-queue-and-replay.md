# Offline Queue and Automatic Replay

## Parent Feature
#2.1 Two-Way IMAP Synchronization

## User Story
As an offline worker, when I have no network connectivity, I want to be able to perform all actions (mark read, flag, move, delete) with immediate local effect, and when connectivity returns, I want all queued actions to execute automatically in order without me re-doing anything.

## Blocked by
1-persistent-operation-queue, 2-mark-read-local-to-server

## Acceptance Criteria
- With no network connectivity, all user-initiated actions are applied to the local state immediately.
- Operations are queued in the persistent operation queue (story 1) as normal.
- The application detects when connectivity is restored.
- On connectivity restoration, the application automatically begins processing the operation queue without user intervention.
- Operations execute in the order the user performed them (for same-message operations).
- After replay completes, the server state converges with the user's offline actions (AC-11).
- The queue can hold a large number of offline operations (at least 1,000) without degradation.

## HITL / AFK
**AFK** — fully automatic; the user just works and the system catches up.

## Estimation
Medium — connectivity detection and automatic replay trigger are the new elements; the queue itself exists from story 1.

## Notes
- US-14, US-15, FR-18, FR-19, FR-20, AC-10, AC-11 are the primary drivers.
- OQ-5 (operation queue size limits) is an open question — no explicit limit was found in the source.
