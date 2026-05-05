# Save with Automatic Connection Check

## Parent Feature

#1.4 Manual Server Configuration

## What to build

When the user saves an account with synchronization enabled and has not already run a successful test in this editing session, the save operation automatically performs a connection check using the current parameters. If the check fails, the save is aborted and the error is displayed. If the check succeeds, the account is saved normally.

This is the safety net described in design note N-2: the primary flow encourages explicit testing, but this catches invalid configurations when the user bypasses the manual test.

Saving with unchanged parameters (idempotent save per NFR-7) should not trigger a re-test or re-sync.

Covers epic sections: FR-42, NFR-7.

## Acceptance criteria

- [ ] Saving a new account with sync enabled and no prior successful test triggers an automatic connection check
- [ ] If the automatic check fails, the save is aborted and an error message is displayed
- [ ] If the automatic check succeeds, the account is saved normally
- [ ] Saving an existing account with unchanged parameters does not trigger a re-test or re-sync (idempotent save)
- [ ] If the user has already run a successful test in this session, no automatic check is performed on save

## Blocked by

- Blocked by `5-inbound-test-connection`

## User stories addressed

- US-22 (auto-test on save when test was skipped)
