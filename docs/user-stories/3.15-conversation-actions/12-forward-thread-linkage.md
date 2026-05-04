# Forward Thread Linkage Configuration

## Parent Feature
#3.15 Conversation Actions

## User Story
As any user, I want to configure whether a forwarded message remains linked to the original conversation thread or starts a new thread, so that I can control how the forward appears in my sent folder and the recipient's inbox.

## Blocked by
`10-basic-forward`

## Acceptance Criteria
- A configurable option controls whether forwards create a new thread or remain linked to the original (FR-24, US-14).
- When linked: the `In-Reply-To` and `References` headers are set, continuing the original conversation (FR-26).
- When starting a new thread: no threading headers are set on the forwarded message (FR-26).
- The default is "linked" (matching the behavior of most desktop clients, per Design Note N-4).
- The setting persists across sessions and applies to all forward actions.

## Mapping to Epic
- US-14
- FR-24, FR-26
- Design Note N-4

## HITL / AFK
AFK — a single boolean preference with clear behavior.

## Notes
- N-4 explains both camps: some users want FYI forwards to stay in the thread (team workflows), others want a clean break. The configurable default satisfies both.
