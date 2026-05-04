# User Story: Re-Sync — Handle Renamed and Disappeared Folders

## Parent Feature
#2.2 Special-Folder Auto-Detection

## Blocked by
- `8-resync-preserve-existing-assignments`

## Description
As any user, when a system folder is renamed on the server or disappears entirely, I want the application to handle this gracefully — updating the name while preserving the role for renames, and re-assigning or prompting me for disappearances — so that my configuration remains functional after server-side changes.

## Motivation
Server-side folder renames and deletions are outside the user's control in the email client. The application must handle these changes without breaking operations like "move to trash" or "save draft".

## Acceptance Criteria
- [ ] When a system folder is **renamed** on the server and the application re-synchronizes, the folder's name is updated in the application but its **role assignment is preserved**. _(FR-19, AC-11)_
- [ ] When a system folder **disappears** from the server (deleted or unsubscribed) and a new folder appears that matches the role, the application re-assigns the role to the new folder (or prompts the user). _(US-15)_
- [ ] When a system folder disappears and no replacement is found, the role becomes unassigned (rather than being assigned to a random folder). The user is informed the next time they attempt an operation that requires that role.
- [ ] Rename tracking uses server-provided folder identity (e.g. UIDVALIDITY or similar) where available, rather than relying solely on name matching.

## Sizing
Medium — rename tracking logic, disappearance handling, and edge cases around identity.

## HITL / AFK
AFK — the happy paths are clear, though some edge cases (e.g. server provides no stable folder identity) may need implementation judgment.

## Notes
- The epic (US-14) specifies that the application should "track the renamed folder and preserve its role assignment". The mechanism for tracking (UIDVALIDITY, folder path history, etc.) is an implementation decision not prescribed by the epic.
- The epic (US-15) says "re-assign the role to the new folder (or prompt me)" — leaving it ambiguous whether this should be automatic or interactive. For a non-interactive sync cycle, automatic re-assignment with a notification seems most practical. Flag for design review if needed.
- OQ-5 in the epic asks whether there should be a user-triggered "re-detect" action. This story does not implement that; it could be a follow-up.
