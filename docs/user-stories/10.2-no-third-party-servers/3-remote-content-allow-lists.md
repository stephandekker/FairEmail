# Per-Sender and Per-Domain Remote Content Allow-Lists

## Parent Feature
#10.2 No Third-Party Servers

## User Story
As any user, I want to permanently authorize remote content from trusted senders or domains, so that I can choose convenience for trusted sources without lowering my default protection for everyone else.

## Blocked by
- `2-remote-content-blocking` (the blocking mechanism must exist before allow-lists can override it)

## Acceptance Criteria
- User can allow remote content permanently for a specific sender email address.
- User can allow remote content permanently for an entire domain.
- Once a sender or domain is allow-listed, opening messages from that sender/domain automatically loads remote content without prompting.
- Allow-list entries can be reviewed, edited, and removed by the user.
- Removing an allow-list entry causes the application to resume blocking remote content for that sender/domain immediately.
- Allow-list data is stored locally on the user's device.

## Mapping to Epic
- US-6
- FR-7

## HITL / AFK
AFK — no human review needed beyond normal code review.

## Notes
- The epic does not specify the exact UX for adding to the allow-list (e.g. whether it's a button on the blocked-content banner or a settings panel entry). This should be determined during design.
