# On-Demand Sync and Polling Interval

## Parent Feature
#1.1 Unlimited Accounts

## User Story
As any user, I want to mark an account as "on-demand" so it syncs only when I explicitly request it, and I want to set the polling / keep-alive interval per account, so that accounts with different responsiveness needs can sync at different rates.

## Blocked by
9-enable-disable-sync

## Acceptance Criteria
- Each account has an "on-demand" toggle (FR-6). When enabled, the account synchronizes only when the user explicitly triggers a sync (US-27, AC-12).
- Each account has a configurable polling / keep-alive interval (FR-6, US-28).
- On-demand and polling interval settings are independent of each other and of the sync enabled/disabled toggle.
- Changes take effect without requiring application restart.

## Mapping to Epic
- US-27, US-28
- FR-6 (on-demand flag, polling interval)
- FR-52 (NOOP vs IDLE — deferred to story 23 advanced settings)
- AC-12

## HITL / AFK
AFK
