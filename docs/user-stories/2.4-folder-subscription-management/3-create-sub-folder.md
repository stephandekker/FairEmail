# Create Sub-Folder

## Parent Feature
#2.4 Folder Subscription Management

## User Story
As a folder organizer, I want to create a sub-folder under any existing folder that supports children, via a context action on the parent folder, so that I can build a nested folder hierarchy.

## Blocked by
`2-create-top-level-folder` — reuses the creation form, queuing mechanism, and server-side execution path introduced there.

## Acceptance Criteria
- A "Create sub-folder" action appears in the context menu of any IMAP folder that supports child folders (inferiors = true).
- The action is disabled (greyed out) for folders that do not support children (inferiors = false).
- The creation form is the same as for top-level folder creation (name, display name, color, sync settings, etc.).
- The full folder name is constructed by concatenating the parent's name, the account's folder separator, and the user-provided name.
- Name validation applies: empty names and duplicate full names are rejected with a clear error.
- The sub-folder is queued, created on the server during sync, and auto-subscribed — same as top-level creation.
- If the account's folder separator is unknown (null), sub-folder creation is not possible and a clear error is shown to the user.

## Mapping to Epic
- Goals: G2
- User Stories: US-13, US-17
- Functional Requirements: FR-15, FR-18
- Acceptance Criteria: AC-9, AC-11

## HITL / AFK
**AFK** — the form and queuing mechanism already exist from story 2. The new work is separator handling and the inferiors guard.

## Estimation
Small — incremental on top of the top-level folder creation story.

## Notes
- OQ-6 in the epic flags that the source application silently refuses sub-folder creation when the separator is unknown. This story requires a user-facing error message instead, which is a behaviour divergence from the Android source. Flag for review if the epic owner has a different preference.
