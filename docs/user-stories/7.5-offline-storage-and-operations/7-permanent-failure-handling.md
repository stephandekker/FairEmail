# Permanent Failure Handling and Notification

## Parent Feature
#7.5 Offline Storage and Operations

## User Story
As any user, when an operation fails permanently (e.g. the target message was deleted on the server, the destination folder no longer exists, quota exceeded), I want to be notified clearly, so that I can decide what to do.

## Acceptance Criteria
- On permanent failure (message not found on server, folder not found, quota exceeded, permission denied, read-only folder), the operation is marked as failed immediately without further retries.
- Failed operations remain in the queue with their error details recorded and visible to the user.
- The user receives a clear notification when a permanent failure occurs.
- An operation that exhausts its maximum retry attempts (transient failures) is also marked as permanently failed.
- Failed operations persist across application restart.

## Complexity
Small

## Blocked by
6-transient-failure-retry

## HITL/AFK
AFK

## Notes
- The distinction between "transient" and "permanent" failures requires categorizing server error responses. Common permanent failures include: message UID not found, folder does not exist, quota exceeded, permission denied, read-only mailbox.
- The notification mechanism is not prescribed; it could be a desktop notification, a badge, or an in-app alert. The key requirement is that it is "clear" and user-visible.
