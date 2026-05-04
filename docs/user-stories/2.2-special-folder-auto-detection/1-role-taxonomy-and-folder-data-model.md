# User Story: Role Taxonomy and Folder Data Model

## Parent Feature
#2.2 Special-Folder Auto-Detection

## Blocked by
_(none — this is the foundation story)_

## Description
As a developer building the email client, I need a well-defined set of folder role constants and a data model that can persist a role assignment per folder per account, so that all subsequent detection, override, and default-property logic has a stable foundation to build on.

## Motivation
Every other story in this epic depends on being able to represent, store, and query folder roles. Without the taxonomy and schema in place, detection logic has nothing to write to and the UI has nothing to read from.

## Acceptance Criteria
- [ ] The application defines the following folder roles as first-class constants: **Inbox, Sent, Drafts, Trash, Spam (Junk), Archive, System, User**. _(FR-1)_
- [ ] Each folder record in the persistent store carries a `role` (or `type`) field that holds one of these values, defaulting to **User**.
- [ ] The **System** role (used for Important / Flagged) is non-assignable by the user — it is set only by server metadata. _(FR-1)_
- [ ] The data model enforces (or the application layer guarantees) that each account has **at most one folder** assigned to each of the user-assignable roles (Inbox, Sent, Drafts, Trash, Spam, Archive) at any time. _(FR-2)_
- [ ] Role assignments survive application restarts (i.e. they are persisted, not in-memory only). _(G5)_
- [ ] A query or lookup exists to retrieve the folder currently holding a given role for a given account (e.g. "get Trash folder for account X").

## Sizing
Small — data model definition, constants, persistence schema, and a uniqueness check.

## HITL / AFK
AFK — no human review needed; this is internal plumbing.

## Notes
- The existing Android codebase defines these constants in `EntityFolder.java` (lines 158–166) with `SYSTEM_FOLDER_ATTR` and `SYSTEM_FOLDER_TYPE` arrays. The desktop implementation should replicate the same taxonomy but is free to choose its own storage mechanism.
- The uniqueness constraint (FR-2) could be enforced at the database level or at the application level. Either approach satisfies the epic; the choice is an implementation decision.
