# Link Confirmation Dialog with URL Display

## Parent Feature
#10.2 No Third-Party Servers

## User Story
As any user, when I click a link in an email, I want the application to show me a confirmation dialog with the actual destination URL before opening it, so that I am not silently redirected through tracking services and I can verify the destination.

## Blocked by
- `1-default-network-posture` (the no-proxy guarantee must be established first)

## Acceptance Criteria
- Clicking any link in an email displays a confirmation dialog before opening it.
- The dialog displays the actual destination URL, clearly showing the domain.
- The dialog warns the user if the displayed link text differs from the actual URL target (e.g. link text says "bank.com" but URL points to "phishing.example.com").
- The link is not opened through any proxy, redirect, or intermediary server operated by the developer or a third party.
- The user can cancel and not open the link.

## Mapping to Epic
- US-8, US-9
- FR-10, FR-11, FR-12
- AC-3

## HITL / AFK
AFK — no human review needed beyond normal code review.

## Notes
- URL tracking parameter stripping is a separate story (story 6) that adds an optional layer on top of this confirmation dialog.
