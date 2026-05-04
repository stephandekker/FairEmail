# Per-Account Notifications

## Parent Feature
#1.1 Unlimited Accounts

## User Story
As any user, I want to enable or disable notifications independently for each account, with a dedicated notification channel per account where the system supports it, so that I can control which accounts generate alerts and customize notification behavior at the system level.

## Blocked by
1-create-imap-account

## Acceptance Criteria
- Each account has a notification enabled/disabled toggle, independent of sync enablement (FR-39, AC-19).
- When notifications are enabled, a dedicated notification channel is created for that account (where the desktop notification system supports channels) (FR-40).
- The notification channel allows system-level customization of sound, priority, and behavior (FR-40).
- Disabling or deleting an account removes its notification channel (FR-41).
- Notification settings survive toggling synchronization off and on (AC-19).

## Mapping to Epic
- US-38, US-39
- FR-39, FR-40, FR-41
- AC-19

## HITL / AFK
AFK

## Notes
- Linux desktop notification systems (e.g. freedesktop.org notifications via D-Bus) may not support "channels" in the same way Android does. The implementation should use the closest available mechanism. This is a design/platform decision.
