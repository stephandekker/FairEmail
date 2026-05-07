# Pending Operation Visibility

## Parent Feature
#2.1 Two-Way IMAP Synchronization

## User Story
As an offline worker (or any user with slow connectivity), I want to see which of my actions are still pending (not yet confirmed by the server), and I want access to a queue view showing operation type, target, status, and errors, so that I understand the state of my mailbox.

## Blocked by
13-offline-queue-and-replay

## Acceptance Criteria
- Messages with pending (unconfirmed) operations are visually distinguishable from messages whose state is confirmed by the server (e.g. dimmed, italicized, or badged) (AC-12).
- The user can view the operation queue, showing: operation type, target message/folder, status (queued, in progress, failed), and error message (if any) (AC-16).
- The queue view updates in real-time as operations execute.
- The visual distinction is removed once the server confirms the operation.

## HITL / AFK
**HITL** — UI/UX decisions about visual indicators and queue view layout require design review.

## Estimation
Medium — involves UI work (visual indicators on messages + dedicated queue view).

## Notes
- US-16, US-19, FR-21, FR-40 (view part), AC-12, AC-16 are the primary drivers.
- G7 (visibility into pending operations) is a stated goal of the epic.
