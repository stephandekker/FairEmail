# Account-Level Monitoring Conditions

## Parent Feature
#8.9 Background Daemon (System Tray Icon)

## User Story
As a multi-account user, I want the daemon to automatically determine which accounts to monitor based on each account's settings, so that only eligible accounts maintain server connections.

## Blocked by
- `1-daemon-process-lifecycle`

## Acceptance Criteria
- The daemon monitors an account only if ALL of the following are true:
  - The global synchronization toggle is enabled.
  - The account's own synchronization setting is enabled.
  - The account is not configured as on-demand-only.
  - The account has at least one folder configured.
  - The account is not marked for deletion.
- The daemon continues running to execute pending operations for an account even if that account does not meet all monitoring conditions, provided the account's sync setting is enabled and it is not marked for deletion.
- Changes to account-level settings take effect on the daemon within seconds without requiring a manual restart.
- Adding a new account (while the global toggle is enabled) causes the daemon to begin monitoring it within seconds.
- Removing or disabling the last eligible account causes the daemon to enter the "no accounts" state (triggering the grace period from story 9).

## Mapping to Epic
- FR-22, FR-23, FR-24
- AC-13 (new account monitored immediately), AC-14 (last account removed triggers shutdown)

## HITL / AFK
AFK — the conditions are explicitly enumerated in the epic.

## Notes
- This story defines the daemon's account eligibility logic. It does not define the account model itself (that belongs to the accounts epic 1.1). The daemon consumes account state; it does not own it.
