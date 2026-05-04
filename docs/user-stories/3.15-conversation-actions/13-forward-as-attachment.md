# Forward-as-Attachment

## Parent Feature
#3.15 Conversation Actions

## User Story
As any user, I want to forward a message as a complete `.eml` file attachment rather than quoting it inline, so that the recipient receives the original message with all headers intact for forensic or reporting purposes.

## Blocked by
`10-basic-forward`

## Acceptance Criteria
- A "Forward as attachment" action is available in the action menu (FR-27).
- The action opens a compose window with the original message attached as a complete `.eml` file containing full RFC 2822 content including all headers (FR-27, AC-8).
- The attached `.eml` file is a byte-accurate copy of the original message — no re-encoding, header stripping, or content modification (NFR-5).
- The user has the choice of attaching a single message or all messages in the conversation (FR-28, AC-9).
- If the raw message content is not available locally, it is downloaded from the server with a progress indicator (FR-29, US-17).
- A configurable option allows automatic download confirmation without prompting each time (FR-30).
- The compose window subject is prefixed with "Fwd:" following the same rules as basic forward.

## Mapping to Epic
- US-15, US-16, US-17
- FR-27, FR-28, FR-29, FR-30
- NFR-5
- AC-8, AC-9

## HITL / AFK
AFK — behavior is well-defined.

## Notes
- OQ-4 in the epic raises a concern about POP3 accounts configured to delete after download: the raw message may no longer exist on the server. The implementation should handle this gracefully (e.g., use the locally-cached raw message if available, or clearly communicate the limitation).
- The "attach entire conversation" option (FR-28) could be a separate, thinner slice if this story proves too large during implementation. It is included here because the UI decision (single vs. conversation) is part of the same compose flow.
