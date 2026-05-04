# Bulk Toggle for Descendant Folders

## Parent Feature

#3.1 Unified Inbox

## What to build

When the user acts on a folder that has descendant (child) folders, offer a bulk action to enable or disable unified-inbox membership for all descendants in one operation (FR-11, US-6). The action should be idempotent — descendants already matching the desired state are silently left alone (see OQ-5).

## Acceptance criteria

- [ ] Acting on a parent folder offers an option to apply the toggle to all descendant folders.
- [ ] Enabling unified membership on a parent folder with 3 children sets all 3 children (and the parent) to unified = true (AC-6).
- [ ] Disabling unified membership on a parent folder with descendants sets all to unified = false.
- [ ] Descendants already matching the target state are silently left unchanged.
- [ ] The bulk change is reflected immediately in the Unified Inbox view.

## Blocked by

- Blocked by `5-toggle-membership-context-action`

## User stories addressed

- US-6 (bulk toggle for parent + all descendants)

## Notes

Open question OQ-5 asks whether the bulk action should report per-folder results or be silently idempotent. The FairEmail Android source is silently idempotent. This story assumes the same behavior but the decision should be confirmed during implementation.
