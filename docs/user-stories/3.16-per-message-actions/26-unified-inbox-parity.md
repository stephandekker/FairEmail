## Parent Feature

#3.16 Per-Message Actions

## What to build

Ensure that every per-message action available in a single-folder view is also available when viewing the same message in the Unified Inbox. Actions invoked from the Unified Inbox apply to the message in its original folder and account. The behaviour must be identical regardless of which view the action was invoked from (FR-76, NFR-3).

## Acceptance criteria

- [ ] All per-message actions are available in the Unified Inbox (AC-23)
- [ ] An action on a message in the Unified Inbox applies to the correct source folder/account
- [ ] Behaviour is identical whether invoked from single-folder view or Unified Inbox (NFR-3)
- [ ] Move, delete, flag, snooze, hide, keywords, notes, and all other actions work from Unified Inbox
- [ ] Undo works correctly for actions invoked from the Unified Inbox

## Blocked by

None — can start immediately (but should be validated after individual action stories are complete).

## User stories addressed

- (Cross-cutting requirement FR-76; touches all user stories indirectly)

## Notes

- This story is best implemented as an integration/validation pass after the individual action stories are complete. It can start immediately as a test harness or acceptance test suite, but full validation requires the actions to exist.
