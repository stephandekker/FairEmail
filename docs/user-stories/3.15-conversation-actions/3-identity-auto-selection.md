# Identity Auto-Selection

## Parent Feature
#3.15 Conversation Actions

## User Story
As any user, when I invoke a conversation action, I want the "From" identity automatically selected based on the action type and the original message, so that I do not need to manually switch identities for each response.

## Blocked by
`2-basic-reply`

## Acceptance Criteria
- For reply, reply-all, and reply-to-list: the "From" identity defaults to the identity whose address matches a recipient address on the original message. If no match, the account's default identity is used (FR-50).
- For forward and forward-as-attachment: the "From" identity defaults to the account that holds the message being forwarded (FR-51).
- For all actions: the user can override the pre-selected identity before sending (FR-52).
- Identity selection is visible in the compose window and clearly indicates which identity is active.

## Mapping to Epic
- US-23 (partially — edit-as-new identity is covered in story 16)
- FR-50, FR-51, FR-52

## HITL / AFK
AFK — logic is well-defined by the epic's rules.

## Notes
- Reply-to-list has a specific identity rule (FR-18: use the address that received the message). This story implements the general identity selection framework; the reply-to-list specialisation is applied when that action is built (story 5).
- Edit-as-new has its own identity rule (FR-37: match the original sender's identity). That is handled in story 16.
