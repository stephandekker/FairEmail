# Gmail Thread Index Strategy

## Parent Feature
#3.2 Conversation Threading

## User Story
As a multi-provider user with a Gmail account, I want the option to use Gmail's native thread grouping (`X-GM-THRID`) so that my conversations match what I see in the Gmail web interface.

## Blocked by
1-rfc-header-thread-computation

## Acceptance Criteria
- [ ] When the Gmail thread index strategy is enabled and a message originates from Gmail, the provider-supplied thread identifier is extracted and used as the thread identifier for that message (FR-9, AC-5).
- [ ] Messages from non-Gmail accounts are unaffected by this setting (AC-5).
- [ ] The Gmail thread index strategy is disabled by default (FR-10).
- [ ] The user interface informs the user that enabling Gmail thread index applies only to newly received messages and may alter existing conversation groupings (FR-11, US-8).
- [ ] The Gmail thread index strategy takes priority over RFC-header-based threading when enabled (FR-1, FR-3).

## HITL / AFK
AFK — provider-specific identifier extraction with clear priority rules.

## Notes
- Per NG-3 and design note N-7, this is a global setting, but it has a de facto per-account effect because it only activates for messages originating from Gmail.
