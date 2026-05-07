# Operation Retry and Error Handling

## Parent Feature
#2.1 Two-Way IMAP Synchronization

## User Story
As any user, I want failed operations to be retried automatically for transient errors (with backoff), marked as permanently failed for non-transient errors (with a clear message), and I want the ability to manually retry or dismiss failed operations.

## Blocked by
14-pending-operation-visibility

## Acceptance Criteria
- Transient errors (network timeout, temporary server unavailability) trigger automatic retry with exponential backoff up to a configurable maximum number of attempts.
- Permanent errors (folder does not exist, message not found, quota exceeded, permission denied) cause the operation to be marked as failed and removed from the active queue after notifying the user.
- The application distinguishes transient from permanent errors based on server response codes.
- After max retries, a transient-error operation is marked as permanently failed with a clear error message (AC-17).
- The user can manually retry a failed operation from the queue view.
- The user can dismiss (cancel) a failed operation from the queue view (AC-18).
- A single failed operation does not block processing of unrelated operations in the queue (NFR-7).
- If a batched operation fails, individual operations within the batch are retried separately (FR-39).

## HITL / AFK
**AFK** for retry logic; **HITL** for manual retry/dismiss actions (user-triggered).

## Estimation
Medium — error classification, backoff logic, and per-operation isolation require careful implementation.

## Notes
- US-20, US-21, US-22, FR-36, FR-37, FR-38, FR-39, FR-40, AC-17, AC-18, NFR-7 are the primary drivers.
