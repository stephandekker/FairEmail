# Forwarded Message Threading

## Parent Feature
#3.2 Conversation Threading

## User Story
As any user, I want the option to have forwarded messages start a new conversation thread rather than being linked to the original thread, so that forwarded discussions do not clutter the original conversation.

## Blocked by
1-rfc-header-thread-computation

## Acceptance Criteria
- [ ] A user-facing send preference ("forward as new thread") controls whether forwarded messages start a new conversation or remain linked to the original (FR-27).
- [ ] When "forward as new thread" is enabled, the application does not include the `X-Forwarded-Message-Id` header in outgoing forwarded messages and does not use that header for threading on incoming messages (FR-28, AC-12).
- [ ] When "forward as new thread" is disabled, forwarded messages are threaded with the original conversation via the `X-Forwarded-Message-Id` header (FR-29, AC-12).
- [ ] The default for new installations is "forward as new thread" enabled (design note N-6).

## HITL / AFK
AFK — a boolean preference controlling header inclusion and threading behaviour.

## Notes
- Per design note N-6, the default is enabled for new installations. Migrated installations may retain the previous default (disabled). The epic does not specify migration behaviour for the desktop application; this may need a decision during implementation.
