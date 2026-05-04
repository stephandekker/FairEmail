# Quoting and Prefix Configuration

## Parent Feature
#3.15 Conversation Actions

## User Story
As any user, I want to configure quoting behavior — including reply header format, quote depth limits, signature stripping from quotes, cursor placement, and subject prefix alternatives — so that my replies match my preferred style without per-message adjustments.

## Blocked by
`2-basic-reply`

## Acceptance Criteria
- The reply header line format is configurable: separate line, extended format, or custom template with placeholders for sender, recipient, date, subject (FR-12).
- A configurable option removes signatures (content below "-- " separator) from quoted text (FR-14).
- A configurable limit on quote depth strips deeply nested quotes to prevent quote bloat (FR-15).
- The user can configure cursor placement: above the quote (top-posting) or below it (bottom-posting) (FR-16).
- The "Re:" prefix can be replaced with a configured alternative prefix (FR-10).
- A deduplication option prevents duplicate prefixes; an alternative "reply count" format (e.g., "Re[3]:") is available (FR-10).
- The "Fwd:" prefix is also configurable with the same deduplication rules (FR-21).
- All configuration options persist across sessions and apply to all reply/forward actions.

## Mapping to Epic
- FR-10, FR-12, FR-14, FR-15, FR-16, FR-21
- AC-17
- Design Note N-8

## HITL / AFK
HITL — the range of configuration options and their defaults may benefit from a brief UX review to determine sensible defaults.

## Notes
- OQ-1 in the epic asks whether alternative prefixes are for locale support or power-user customization. This story implements the configurable prefix mechanism; the decision on whether to auto-detect locale-appropriate prefixes can be deferred.
- Quote depth limiting (N-8) is a space optimization that most users will never notice but prevents pathological message sizes.
