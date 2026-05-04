# Copy Message — Local to Server

## Parent Feature
#2.1 Two-Way IMAP Synchronization

## User Story
As any user, when I copy a message to another folder, I want the copy to appear on the server in the destination folder while the original remains in place.

## Blocked by
1-persistent-operation-queue

## Acceptance Criteria
- Copying a message creates a "copy" operation in the queue.
- The operation executes an IMAP COPY command to the destination folder.
- The original message remains in the source folder (unchanged).
- The copy appears in the destination folder on the server (verifiable via webmail).
- The local store records the new UID of the copied message in the destination folder.

## HITL / AFK
**AFK** — straightforward IMAP COPY with no user decisions.

## Estimation
Small — simpler than move (no source deletion, no atomic-vs-fallback logic).

## Notes
- US-8, FR-1 are the primary drivers.
