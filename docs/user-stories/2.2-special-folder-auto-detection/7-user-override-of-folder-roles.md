# User Story: User Override of Folder Roles

## Parent Feature
#2.2 Special-Folder Auto-Detection

## Blocked by
- `1-role-taxonomy-and-folder-data-model`
- `6-role-triggered-default-properties`

## Description
As any user, I want to manually assign or reassign any selectable folder to any of the assignable roles (Sent, Drafts, Trash, Spam, Archive) through the account settings interface, so that I can correct detection errors or accommodate unusual server configurations.

## Motivation
Auto-detection cannot cover every edge case. The user must always have final authority over folder roles. This story is Tier 3 of the detection strategy and is the ultimate safety net.

## Acceptance Criteria
- [ ] The account settings screen lists, for each assignable role (Sent, Drafts, Trash, Spam, Archive), the currently assigned folder (if any) and allows the user to select a different folder from all available selectable folders. _(FR-13, US-10, US-11)_
- [ ] A user override takes **immediate, authoritative precedence** over any prior automatic detection for that role. _(FR-14)_
- [ ] When the user assigns a role to a new folder, the folder that previously held that role **reverts to User** type. _(FR-15, US-12, AC-16)_
- [ ] The reassignment takes effect immediately — the user does not need to restart or re-sync. _(AC-5)_
- [ ] User overrides **persist** across application restarts, folder re-synchronization, and server-side folder renames. _(FR-16, US-13)_
- [ ] User overrides are **never overwritten** by subsequent automatic detection runs. _(FR-16, N-8)_
- [ ] Default properties (from Story 6) are applied when a role is assigned to a new folder via override.
- [ ] The user can see which folder currently holds each role for any account. _(NFR-5)_

## Sizing
Medium — settings UI for role assignment, persistence of override flag, integration with detection pipeline to respect overrides.

## HITL / AFK
AFK — the behaviour is fully specified. UI layout is an implementation detail.

## Notes
- The epic does not specify whether the user can override the Inbox role from account settings (FR-13 lists only Sent, Drafts, Trash, Spam, Archive). This appears intentional — the Inbox is always determined by server convention (the "INBOX" name). If a user needs to change their Inbox, that is a server-side concern.
- The Android code uses `DaoFolder.setFolderType()` to persist overrides and restricts UI-driven type changes to User folders (`FragmentFolder.java` line 363). The desktop implementation should allow changing from any type to any assignable type, per FR-13.
