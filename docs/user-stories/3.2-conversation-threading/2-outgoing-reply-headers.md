# Outgoing Reply Headers

## Parent Feature
#3.2 Conversation Threading

## User Story
As any user, when I compose a reply, I want the application to set correct `In-Reply-To` and `References` headers on my outgoing message so that recipients' mail clients can thread my reply correctly.

## Blocked by
1-rfc-header-thread-computation

## Acceptance Criteria
- [ ] When composing a reply, `In-Reply-To` is set to the `Message-ID` of the message being replied to (FR-47).
- [ ] The `References` header is set to the replied-to message's `References` value followed by its `Message-ID`, per RFC 5322 (FR-47).
- [ ] The `References` header length is limited on outgoing messages to prevent excessively large headers on deeply nested threads (FR-48).
- [ ] Outgoing replies carry correct headers as verified by inspecting raw sent messages (AC-20).

## HITL / AFK
AFK — straightforward header composition with well-defined RFC rules.

## Notes
- This is the "write" side of RFC threading (story 1 is the "read" side). Together they form a complete RFC-threading loop.
