# Block Remote Content in Messages by Default

## Parent Feature
#10.2 No Third-Party Servers

## User Story
As any user, when I open an email containing remote images, fonts, CSS, iframes, scripts, or other externally hosted resources, I want the application to block all such content by default and show me a clear indicator with a one-action control to load it for the current message, so that the sender cannot track whether or when I opened the message.

## Blocked by
- `1-default-network-posture` (the default posture must be established first)

## Acceptance Criteria
- Opening an email containing remote images does not fetch those images until the user explicitly authorizes it via an on-screen control.
- All remotely hosted content types are blocked: images, fonts, CSS, iframes, scripts, objects, media.
- When remote content is blocked, a visible, non-modal indicator is displayed to the user.
- A one-action control (e.g. a button or banner action) is available to load remote content for the current message.
- When the user authorizes remote content, the application fetches it directly from the hosting server — not through any intermediary proxy operated by the developer or a third party.
- The blocking applies by default without the user needing to configure anything.

## Mapping to Epic
- US-5
- FR-5, FR-6, FR-9
- AC-2

## HITL / AFK
AFK — no human review needed beyond normal code review.

## Notes
- This story covers the per-message "load now" action only. Per-sender and per-domain allow-lists are a separate story (story 3). Tracking pixel detection is a separate story (story 4).
- The non-modal indicator is important: it should inform the user that content was blocked without interrupting the reading flow.
