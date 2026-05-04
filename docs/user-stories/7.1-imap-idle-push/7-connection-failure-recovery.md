# Connection Failure Detection and Recovery

## Parent Feature
#7.1 IMAP IDLE (Push)

## User Story
As any user, when an IDLE connection is lost due to a timeout, server close, or network error, I want the application to detect the failure, reconnect with exponential backoff, and perform a full folder check before resuming IDLE, so that I never silently miss new mail and push resumes without my intervention.

## Blocked by
- `5-keep-alive-mechanism`

## Acceptance Criteria
- When an IDLE connection is lost (timeout, server close, network error), the application attempts to reconnect after a brief delay (FR-28).
- Reconnection uses exponential backoff: starting from a short initial delay, doubling with each consecutive failure, up to a configurable maximum ceiling (FR-29, Design Note N-8).
- On successful reconnection, the application performs a full folder check before re-entering IDLE, to catch messages that arrived during the disconnection window (FR-30, AC-6).
- Disconnecting the network for 60 seconds and reconnecting results in push resuming within 30 seconds of network restoration, with missed messages fetched (AC-6).
- If reconnection fails for an extended period (e.g. 30+ minutes), the application surfaces an account-level error notification (FR-31, AC-12).
- Clearing the underlying issue and restoring connectivity causes the error notification to resolve automatically (AC-12).
- Backoff scheduling avoids thundering-herd effects when multiple accounts lose connectivity simultaneously (NFR-7).

## Mapping to Epic
- US-11, US-12, US-13, US-14
- FR-28, FR-29, FR-30, FR-31
- NFR-4, NFR-7
- AC-6, AC-12

## HITL / AFK
AFK — no human review needed beyond normal code review.

## Notes
- This story covers the core reconnection loop. Network-change-triggered recovery (detecting connectivity restoration via system events) is split out as story 11.
- The exponential backoff ceiling should be chosen to balance responsiveness against resource consumption. The epic does not specify exact values — implementation should document the chosen initial delay, multiplier, and ceiling.
- NFR-4 requires 99% push uptime over 24 hours (~15 min cumulative downtime). The reconnection strategy should be aggressive enough to meet this target on stable networks.
