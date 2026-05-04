# Thread Consolidation

## Parent Feature
#3.2 Conversation Threading

## User Story
As any user, when a message arrives that bridges two previously separate threads — by referencing messages from both — I want the application to merge those threads into one transparently, so that the full conversation is visible in a single view.

## Blocked by
1-rfc-header-thread-computation

## Acceptance Criteria
- [ ] When a newly arrived message references (via `In-Reply-To`, `References`, or `X-Forwarded-Message-Id`) messages that belong to different existing threads, those threads are consolidated into a single thread (FR-24, AC-9).
- [ ] Consolidation updates all affected messages — both those sent before and after the arriving message — to share the same thread identifier (FR-25).
- [ ] All messages from both former threads appear in the merged conversation (AC-9).
- [ ] Consolidation respects the thread time range: only messages within the configured time window are candidates (FR-26).
- [ ] Consolidation happens transparently with no user action required (US-18).

## HITL / AFK
AFK — deterministic merge logic triggered by message arrival.

## Notes
- Per design note N-3, consolidation is eager and irreversible within the current threading pass. Splitting a merged thread requires deleting the bridging message or re-threading from scratch. This is intentional.
