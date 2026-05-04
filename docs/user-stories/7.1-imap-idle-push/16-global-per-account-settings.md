# Global and Per-Account Push/Poll Settings

## Parent Feature
#7.1 IMAP IDLE (Push)

## User Story
As any user, I want a global setting to choose between push and poll mode, a per-account keep-alive interval override, and the ability to enable or disable automatic keep-alive tuning, so that I can make blanket or fine-grained configuration choices to suit my environment.

## Blocked by
- `6-keep-alive-auto-tuning`
- `3-poll-mode-fallback`

## Acceptance Criteria
- A global setting allows the user to choose between push mode (IDLE) and poll mode (with configurable interval) for all accounts (US-22).
- The user can configure the keep-alive interval per account, overriding the global default (US-21, AC-15).
- A user-configured keep-alive interval is used instead of the auto-tuned value (AC-15).
- The user can enable or disable automatic keep-alive tuning per account (US-24).
- When auto-tuning is disabled, the manually chosen interval is locked and auto-tuning does not override it (AC-15).
- The user can configure per account whether push is permitted on metered connections (FR-34 — UI surface for story 12).
- The user can exempt specific accounts from power-saving restrictions (FR-36 — UI surface for story 13).

## Mapping to Epic
- US-21, US-22, US-24
- FR-11 (configurable interval), FR-22 (tuning toggle), FR-34 (metered), FR-36 (power exemption)
- AC-15

## HITL / AFK
HITL — the settings UI layout and discoverability should be reviewed.

## Notes
- This story is the settings UI surface for configuration options whose underlying logic is implemented in other stories (keep-alive in story 5, tuning in story 6, metered in story 12, power in story 13). It should not duplicate logic — only expose configuration.
- Open Question OQ-3: should users be allowed to set keep-alive values above 29 minutes (RFC 2177 ceiling)? The epic does not resolve this. Consider enforcing a warning rather than a hard cap, to preserve user control while flagging risk.
