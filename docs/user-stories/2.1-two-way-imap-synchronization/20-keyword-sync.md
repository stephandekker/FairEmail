# Custom Keyword/Label Synchronization

## Parent Feature
#2.1 Two-Way IMAP Synchronization

## User Story
As a power user, when I assign a custom keyword or label to a message, I want that keyword stored on the server (if supported), and when a keyword is set on the server, I want it to appear locally — so that keywords are consistent across all my clients.

## Blocked by
3-flag-unflag-local-to-server, 4-server-flag-change-detection

## Acceptance Criteria
- Setting a custom keyword locally creates a "set keyword" operation that propagates to the server.
- Removing a custom keyword locally creates a "remove keyword" operation.
- Keywords set on the server appear locally after the next sync cycle (AC-21).
- If the server does not support user-defined flags (not in permanent-flags set), the keyword is stored locally only and no server sync is attempted (FR-34).
- If the server supports user flags, keywords round-trip correctly (local→server→other-client, and server→local).

## HITL / AFK
**AFK** — transparent sync when supported, graceful local-only fallback when not.

## Estimation
Small — extends the flag sync pipeline with keyword-specific handling and capability check.

## Notes
- US-7, US-12, FR-31, FR-32, FR-34, AC-21 are the primary drivers.
- OQ-4 (keyword/label sync for label-based providers) is an open question. This story covers standard IMAP keywords; label-based provider semantics may need a separate story if pursued.
