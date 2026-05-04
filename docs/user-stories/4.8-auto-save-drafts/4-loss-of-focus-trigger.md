# Auto-Save on Loss of Focus

## Parent Feature
#4.8 Auto-Save Drafts

## User Story
As any user, when the compose window loses focus (I switch to another window, minimize the application, or navigate away from the editor), I want the application to immediately save the current draft, so that my work is not lost if I do not return.

## Blocked by
1-local-draft-persistence

## Acceptance Criteria
- Switching away from the compose window (to another application window, minimizing, or navigating away) triggers an auto-save of the current draft, provided the draft is dirty. (AC-5)
- This trigger is **unconditional** — it cannot be disabled by the user. (US-8)
- The save completes before the compose window fully relinquishes control, so that the saved state is durable even if the application is terminated immediately afterward. (FR-7)
- The save is silent: no spinner, no dialog. (FR-6)

## Mapping to Epic
- FR-3 (loss-of-focus trigger, unconditional)
- FR-4 (dirty state guard)
- FR-6, FR-7 (silent, completes before relinquishing control)
- US-7, US-8
- AC-5

## HITL / AFK
AFK — well-defined lifecycle event, clear requirements.

## Notes
- FR-7 specifies the save must complete before the window relinquishes control. On a desktop platform this likely means a synchronous (blocking) write in the focus-lost handler, unlike the paragraph/punctuation triggers which are async. This is a deliberate difference.

## Estimation
Small — one lifecycle listener, synchronous save path.
