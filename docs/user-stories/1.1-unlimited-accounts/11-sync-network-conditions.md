# Sync Network Conditions and Schedule Exemption

## Parent Feature
#1.1 Unlimited Accounts

## User Story
As any user, I want to constrain an account to synchronize only on unmetered networks or only when a VPN is active, and I want to exempt a critical account from the global sync schedule, so that I can control data usage, security, and responsiveness on a per-account basis.

## Blocked by
9-enable-disable-sync

## Acceptance Criteria
- Each account has an "unmetered network only" toggle (FR-7). When enabled, sync is suppressed on metered connections (US-29).
- Each account has a "VPN only" toggle (FR-7). When enabled, sync is suppressed when no VPN is active (US-29, AC-13).
- Each account has a "schedule exemption" flag (FR-7). When enabled, the account continues to sync even during off-hours defined by the global schedule (US-30).
- These conditions are evaluated independently and all must be satisfied for sync to proceed.
- When a condition is not met, the account's sync state clearly indicates why it is paused.

## Mapping to Epic
- US-29, US-30
- FR-7
- AC-13

## HITL / AFK
AFK

## Notes
- The epic's OQ-6 flags that whether schedule exemption belongs in account settings or a centralized scheduling UI is a design decision. This story assumes it is in account settings, consistent with the source application behavior.
