# Move Message — Local to Server

## Parent Feature
#2.1 Two-Way IMAP Synchronization

## User Story
As any user, when I move a message from one folder to another, I want the message to appear in the destination folder on the server and disappear from the source folder, so that the move is real and visible on all devices.

## Blocked by
1-persistent-operation-queue

## Acceptance Criteria
- Moving a message locally creates a "move" operation in the queue and updates the local folder assignment immediately.
- When the server supports atomic move, the application uses it (single MOVE command).
- When the server does not support atomic move, the application falls back to COPY-to-destination then DELETE-from-source, ensuring the copy succeeds before the delete is issued.
- After a successful move, the application locates the message in the destination folder (by message identifier) and updates the local UID mapping.
- Subsequent operations on the moved message target the correct server object in the destination folder.
- The message disappears from the source folder on the server and appears in the destination folder (verifiable via webmail).
- Moving to Junk/Spam sets appropriate junk-indicator keywords (if supported) and removes not-junk keywords; the reverse applies when moving out of Junk.

## HITL / AFK
**AFK** — automated move propagation.

## Estimation
Large — involves capability detection (atomic move vs. fallback), UID remapping, and junk-keyword handling.

## Notes
- US-3, FR-1, FR-22, FR-23, FR-24, FR-25, AC-3 are the primary drivers.
- N-4 explains why atomic move is preferred over copy+delete.
- Provider-specific move quirks (OQ-2) are handled in the provider compatibility story (20).
