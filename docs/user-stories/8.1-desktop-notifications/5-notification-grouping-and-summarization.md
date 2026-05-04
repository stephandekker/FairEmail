## Parent Feature

#8.1 Desktop Notifications

## What to build

When multiple new messages are pending notification, group them rather than displaying a barrage of individual pop-ups. The grouping strategy is configurable: Unified (default — all accounts in one group), Per-account, or Per-folder. Each group has a summary notification showing the total count and a list of sender names (with subjects if enabled). Individual notifications within a group are viewable when expanded. A configurable cap limits the number of individual notifications per group; additional messages are represented only in the summary count. A summary-only mode shows only the group summary with no individual notifications.

Covers epic sections: §7.3 (FR-9, FR-10, FR-11, FR-12, FR-13).

## Acceptance criteria

- [ ] When 15 messages arrive in a single sync cycle, at most the configured cap of individual notifications are displayed, with a summary showing the total count of 15 (AC-4)
- [ ] Grouping strategy is configurable: Unified, Per-account, Per-folder
- [ ] Unified grouping (default) groups all new-mail notifications under a single summary
- [ ] Each group summary shows the total count and a list of sender names (with subjects if enabled)
- [ ] Individual message notifications within a group are viewable when the group is expanded (AC-5-like)
- [ ] Summary-only mode: enabling it causes only the group summary to appear, no individual per-message notifications (AC-20)

## Blocked by

- Blocked by `1-notification-categories-and-basic-new-mail`

## User stories addressed

- US-4 (multiple messages grouped/summarized instead of barrage)
- US-5 (individual notifications expandable within a group)

## Notes

- OQ-1 is directly relevant here: desktop notification daemons vary in grouping support. This story should define a graceful degradation path (e.g. fall back to capped individual notifications if the daemon does not support grouping).
- The per-group cap value is not specified by the epic (FR-12 says "e.g. 10"). The implementation should pick a sensible default and make it configurable.
