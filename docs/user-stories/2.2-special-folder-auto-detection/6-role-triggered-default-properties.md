# User Story: Role-Triggered Default Properties

## Parent Feature
#2.2 Special-Folder Auto-Detection

## Blocked by
- `1-role-taxonomy-and-folder-data-model`

## Description
As a typical user, when the application assigns a role to a folder (whether by auto-detection or manual override), I want sensible default properties to be applied automatically — synchronization, notifications, retention, and classification settings — so that system folders are immediately operational without extra configuration.

## Motivation
Detecting folder roles is only useful if the application acts on them. This story wires up the "so what" — the concrete behaviours that make Inbox notifications work, Drafts retention longer, and Spam a classification source, all without the user lifting a finger.

## Acceptance Criteria
- [ ] When a folder is assigned the **Inbox** role, it is automatically configured with: synchronization enabled, unified-inbox membership enabled, notifications enabled, and classification-source enabled. _(FR-20, AC-9)_
- [ ] When a folder is assigned any system role (**Sent, Drafts, Trash, Spam, Archive**), it is automatically configured with synchronization enabled and polling/download settings appropriate to the role. _(FR-21)_
- [ ] The **Drafts** role triggers a longer default retention period than other system folders (e.g. 365 days vs. 30 days). _(FR-22, AC-10)_
- [ ] The **Spam** role triggers automatic configuration as a classification-training source. _(FR-23)_
- [ ] Default properties are set **only at the time of initial role assignment**. Once set, re-synchronization does not reset them. _(FR-24, N-7)_
- [ ] After defaults are applied, the user is free to change any of them, and those changes are respected. _(N-7)_

## Sizing
Small-Medium — a mapping from role → default property set, applied once at assignment time.

## HITL / AFK
AFK — the defaults are fully specified by the epic; no judgment calls needed.

## Notes
- The Android code implements this in `EntityFolder.setProperties()` (lines 295–317) with per-type arrays for sync, poll, and download defaults. The desktop implementation should replicate the same default values.
- This story has no dependency on a specific detection tier — it fires whenever a role is assigned, regardless of how (Tier 1, Tier 2, or user override). It does depend on the data model (Story 1) to know what "assigning a role" means.
