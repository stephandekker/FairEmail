# Selected-Text Quoting

## Parent Feature
#3.15 Conversation Actions

## User Story
As any user, when I select text in a message before invoking reply, I want only the selected text quoted in my reply (not the entire message body), so that I can respond to a specific part of the message.

## Blocked by
`2-basic-reply`

## Acceptance Criteria
- If the user has selected text in the message view before clicking reply, only the selected text is quoted in the compose window (FR-13, AC-4).
- If no text is selected, the full message body is quoted as normal.
- The reply header line ("On [date], [sender] wrote:") still precedes the selected-text quote.
- Selected-text quoting works for reply, reply-all, and reply-to-list actions.
- Threading headers and subject prefix are unaffected by the quoting mode.

## Mapping to Epic
- US-6
- FR-13
- AC-4

## HITL / AFK
AFK — the behavior is straightforward.

## Notes
- The mechanism for capturing the current text selection from the message view and passing it to the compose action needs to be coordinated with the message rendering layer. The exact integration point depends on how the message view exposes selections.
