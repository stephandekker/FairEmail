# Export Account Configurations

## Parent Feature
#1.1 Unlimited Accounts

## User Story
As any user, I want to export account configurations to a portable file, optionally password-protected, so that I can back up my setup or transfer it to another machine.

## Blocked by
1-create-imap-account

## Acceptance Criteria
- User can export all account configurations, including nested identities, folder mappings, rules, and contacts, to a portable file (FR-47).
- The export supports optional password-based encryption (FR-48).
- The user can selectively choose which accounts and which categories of data to include in the export (FR-50, US-45).
- The export file preserves each account's unique identifier for use in duplicate detection on import (N-8).
- The export flow provides clear feedback on success or failure.

## Mapping to Epic
- US-43, US-45
- FR-47, FR-48, FR-50
- AC-15 (export portion)

## HITL / AFK
AFK

## Notes
- The portable file format is not specified by the epic. This is an implementation decision — could be JSON, encrypted archive, or a custom format. Should be documented once decided.
- NFR-5 (secure credential storage) implies that exported passwords/tokens should be protected — the optional password encryption addresses this.
