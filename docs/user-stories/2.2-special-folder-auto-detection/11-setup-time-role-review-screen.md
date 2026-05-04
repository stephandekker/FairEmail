# User Story: Setup-Time Role Review Screen

## Parent Feature
#2.2 Special-Folder Auto-Detection

## Blocked by
- `2-tier1-imap-special-use-detection`
- `4-tier2-name-heuristic-detection`
- `6-role-triggered-default-properties`
- `7-user-override-of-folder-roles`

## Description
As any user, during account setup, after the folder list has been fetched and roles detected, I want to be shown the detected system folder assignments and be able to review and correct them before the account becomes active, so that I can catch misdetections before they cause problems.

## Motivation
Even the best auto-detection can make mistakes, especially on unusual servers. A setup-time review step gives the user a chance to correct errors immediately — before sent messages go to the wrong folder or drafts disappear.

## Acceptance Criteria
- [ ] During account setup, after the folder list has been fetched and roles detected, the application presents the detected system folder assignments to the user for review. _(FR-27, AC-13)_
- [ ] The review screen shows, for each role (Inbox, Sent, Drafts, Trash, Spam, Archive), the folder that was auto-detected (or "none detected" if no folder was assigned).
- [ ] The user can change any system folder assignment from the setup screen before completing account creation. _(FR-28, AC-13)_
- [ ] If the application cannot identify a **Drafts** or **Sent** folder, a warning is shown to the user during setup. _(FR-29, AC-14, US-23)_
- [ ] If the application cannot identify a **Trash** folder, a warning is also shown. _(FR-29)_
- [ ] The user can dismiss warnings and proceed without assigning the missing roles (the account is still created), but they are informed that certain operations may not work correctly.
- [ ] Changes made on the review screen are treated as user overrides (Story 7) and have the same persistence guarantees.

## Sizing
Medium — UI screen in the account setup flow, integration with detection results, warning logic.

## HITL / AFK
AFK — the behaviour is well-specified. UI design and layout are implementation decisions.

## Notes
- This story is the last in the recommended build order because it depends on detection (Stories 2, 4), defaults (Story 6), and override persistence (Story 7) all being in place. It is the "capstone" that ties the detection pipeline into the user-facing setup flow.
- The epic's US-22 says "shown the detected assignments before finalizing". This implies the review step is mandatory (not skippable). However, the epic does not explicitly say it cannot be skipped. For v1, implementing it as a mandatory step is recommended.
- OQ-2 is relevant here: if the review screen shows low-confidence assignments, the user can correct them. This partially mitigates the confidence-threshold question.
