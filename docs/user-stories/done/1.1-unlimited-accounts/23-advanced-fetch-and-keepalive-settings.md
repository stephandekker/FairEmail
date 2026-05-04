# Advanced Fetch and Keep-Alive Settings

## Parent Feature
#1.1 Unlimited Accounts

## User Story
As a power user, I want to configure advanced fetch and keep-alive settings per account — partial fetch, raw fetch, date header preference, UTF-8 support, and NOOP-instead-of-IDLE — so that I can tune connection behavior for specific servers.

## Blocked by
1-create-imap-account

## Acceptance Criteria
- Each account exposes advanced fetch settings: partial-fetch mode, raw-fetch mode, ignore-size-limits flag, date-header preference (server time, Date header, or Received header), and unicode/UTF-8 support flag (FR-51).
- Each account exposes keep-alive settings: polling interval (shared with story 10) and use-NOOP-instead-of-IDLE flag (FR-52).
- All advanced settings are hidden behind an expandable "Advanced" section to avoid overwhelming new users (FR-53).
- Defaults are sensible for the common case (no advanced tuning needed for typical servers).

## Mapping to Epic
- FR-51, FR-52, FR-53

## HITL / AFK
AFK
