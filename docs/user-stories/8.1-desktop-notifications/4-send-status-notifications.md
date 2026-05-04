## Parent Feature

#8.1 Desktop Notifications

## What to build

Provide visibility into outgoing mail status via the system-tray icon tooltip and/or transient notifications. While messages are queued for sending, indicate the queue size. During active transmission, show a progress indication. When a send operation fails, emit a high-urgency notification including the recipient address and error reason.

Covers epic sections: §7.6 (FR-22, FR-23, FR-24, FR-25).

## Acceptance criteria

- [ ] While messages are queued to send, the system-tray tooltip or a notification shows the number of queued messages (AC-9)
- [ ] During active send, a progress indication is visible (AC-9)
- [ ] A failed send produces a high-urgency notification showing the recipient address and error reason (AC-8)
- [ ] Send-failure notifications use the Warning or Error notification category
- [ ] Send status uses the Send Status notification category

## Blocked by

- Blocked by `1-notification-categories-and-basic-new-mail`

## User stories addressed

- US-15 (send-in-progress indication)
- US-16 (send-failure notification with recipient and error)
- US-17 (send-failure notifications are high-urgency)
- US-18 (send progress in tray tooltip or notification)

## Notes

- This story depends on the system-tray icon existing (epic 8.9). If the tray icon is not yet implemented, the send-status indication can be delivered as a transient notification only, with tray integration added when 8.9 is available. The epic's non-goal NG1 explicitly states the tray icon lifecycle is covered by 8.9.
