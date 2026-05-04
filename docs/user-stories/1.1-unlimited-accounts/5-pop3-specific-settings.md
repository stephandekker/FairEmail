# POP3-Specific Settings

## Parent Feature
#1.1 Unlimited Accounts

## User Story
As a POP3 user, I want to configure POP3-specific behaviors — whether to leave messages on the server, how deletions are handled, and a download cap — so that I can control the relationship between local and server-side state.

## Blocked by
4-add-pop3-account

## Acceptance Criteria
- POP3 accounts expose a "leave on server" toggle. When disabled, downloaded messages are removed from the server. When enabled, they remain (US-31, AC-14).
- POP3 accounts expose a "delete from server when deleted on device" toggle (US-32).
- POP3 accounts expose a "keep on device when deleted from server" toggle (US-33).
- POP3 accounts expose a "maximum messages to download" setting (US-34).
- These settings are only visible for POP3 accounts, not IMAP.
- Defaults are sensible for new POP3 accounts (e.g. leave on server = enabled).

## Mapping to Epic
- US-31, US-32, US-33, US-34
- FR-9
- AC-14

## HITL / AFK
AFK
