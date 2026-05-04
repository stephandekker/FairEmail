# Basic Reply

## Parent Feature
#3.15 Conversation Actions

## User Story
As a conversationalist, when I reply to a message, I want the To field populated with the sender's reply-to address, the subject prefixed with "Re:", the original body quoted below my cursor, and correct threading headers set, so that my response reaches the right person and appears in the correct conversation thread.

## Blocked by
`1-action-menu-infrastructure`

## Acceptance Criteria
- Selecting "Reply" from the action menu opens a compose window (FR-7).
- The "To" field is set to the Reply-To header address; if no Reply-To exists, the From address is used (FR-7).
- The subject is prefixed with "Re:" (FR-10).
- If the subject already begins with "Re:", the prefix is not duplicated (FR-10, AC-17).
- The `In-Reply-To` header is set to the Message-ID of the original message (FR-11).
- The `References` header carries the full reference chain of the conversation (FR-11).
- The original message body is quoted below the composition area, preceded by a reply header line (e.g., "On [date], [sender] wrote:") (FR-12).
- The compose window opens in under one second for locally-available messages (NFR-1).
- Generated headers conform to RFC 5322 (NFR-2).
- Threading headers result in correct thread grouping in major mail clients (NFR-3).
- Reply works offline for downloaded messages, with the send queued for later delivery (NFR-4, AC-22).

## Mapping to Epic
- US-1, US-2, US-3, US-4
- FR-7, FR-10, FR-11, FR-12
- NFR-1, NFR-2, NFR-3, NFR-4
- AC-1, AC-17, AC-22

## HITL / AFK
AFK — straightforward implementation of well-defined RFC behavior.

## Notes
- This story covers the minimal reply path. Self-reply detection (story 6), selected-text quoting (story 7), and quoting configuration options (story 8) are separate slices.
- Identity auto-selection is handled in story 3; this story can use a sensible default (account's default identity) until then.
