## Parent Feature

#1.3 Quick Setup Wizard

## What to build

After the account is saved, ensure all standard system folders exist and configure default synchronization settings, then trigger immediate sync and navigate to the inbox (FR-29, FR-30, FR-31).

**Missing folder creation (FR-29):**
- If any standard system folders (Drafts, Sent, Archive, Trash, Spam) are missing on the server, create them in the account's namespace using the server's namespace conventions (Design Note N-6).

**Default sync settings (FR-30):**
- Inbox: synchronize and download messages, with push/idle if supported (FR-30a).
- Drafts, Sent, Archive: synchronize and download messages, polled periodically (FR-30b).
- Trash, Spam: synchronize (polled) but do not download message bodies by default (FR-30c).
- User-created folders: do not synchronize by default (FR-30d).

**Post-save behavior (FR-31):**
- Trigger an immediate synchronization cycle.
- Navigate the user to a view where incoming messages will appear.

## Acceptance criteria

- [ ] After account creation, system folders (Drafts, Sent, Trash, Spam, Archive) are present in the folder list (AC-10)
- [ ] If any system folders were missing on the server, they have been created (AC-10, FR-29)
- [ ] Inbox is configured for sync + download + push/idle (FR-30a)
- [ ] Drafts, Sent, Archive are configured for sync + download, polled (FR-30b)
- [ ] Trash, Spam are configured for sync only (no body download) (FR-30c)
- [ ] User-created folders are not synchronized by default (FR-30d)
- [ ] After saving, the Inbox is synchronized and messages appear without further user action (AC-9, FR-31)
- [ ] The user is navigated to a view where incoming messages appear (FR-31)

## Blocked by

- Blocked by 13-account-and-identity-creation

## User stories addressed

- US-23 (sync begins immediately, navigate to inbox view)

## Notes

- Open Question OQ-8 in the epic asks whether missing-folder creation should be optional or configurable (e.g. for servers that don't permit folder creation, or users who prefer different folder names/languages). The initial implementation should create missing folders per the epic's specification and flag this as a future consideration.
