# Image Resize on Reply

## Parent Feature
#3.15 Conversation Actions

## User Story
As any user, when replying to a message with inline images, I want the option to automatically reduce their resolution, so that reply message sizes stay manageable.

## Blocked by
`2-basic-reply`

## Acceptance Criteria
- A configurable option controls whether inline images from the original message are automatically reduced in resolution when replying (FR-59).
- The option applies only to replies, not to forwards (FR-59).
- The option defaults to off (or a sensible default) and persists across sessions.
- When enabled, inline images in the quoted content are resized to a lower resolution before being included in the reply draft.

## Mapping to Epic
- FR-59

## HITL / AFK
AFK — a single preference with straightforward behavior.

## Notes
- The exact resize dimensions/quality are not specified by the epic and are left as an implementation detail. A reasonable default (e.g., limiting to 1024px width) should be chosen.
- This feature exists to prevent pathological message sizes in long threads with many inline images. It is a nice-to-have optimization that could be deferred if higher-priority stories are not yet complete.
