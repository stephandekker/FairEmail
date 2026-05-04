# Local Draft Persistence with Dirty State Tracking

## Parent Feature
#4.8 Auto-Save Drafts

## User Story
As any user composing an email, I want the application to be able to persist the current state of my draft body to local storage silently and without interactive prompts, so that there is a reliable foundation for all auto-save triggers to build on.

## Blocked by
(none — this is the foundation slice)

## Acceptance Criteria
- The draft body can be persisted to local durable storage so that it survives application crash, unexpected termination, and system reboot.
- A dirty-state flag tracks whether the draft body has been modified since the last save; only dirty drafts are eligible for auto-save.
- The dirty flag is set when the body content changes and cleared when a save completes.
- Saving is silent: no progress spinner, no blocking of user input, no encryption dialogs or interactive prompts (encryption is deferred to explicit send or manual save).
- Local persistence completes within a timeframe imperceptible to the user (well under one second) and does not block the compose editor — the user can continue typing while the save is persisted in the background.
- On next application launch after a crash, a previously saved draft is recoverable.

## Mapping to Epic
- FR-4 (dirty state guard)
- FR-6 (silent saves, no interactive prompts)
- FR-8 (persist full current state of draft body)
- NFR-1 (latency)
- NFR-2 (concurrency — non-blocking)
- NFR-3 (durability)
- US-21, US-22

## HITL / AFK
AFK — no human review needed; this is internal plumbing with clear acceptance criteria.

## Estimation
Small-to-medium — one persistence mechanism, one boolean flag, crash-recovery verification.
