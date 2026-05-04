# Data Locality — All Data Stored Locally

## Parent Feature
#10.2 No Third-Party Servers

## User Story
As any user, I want all my message content, metadata, attachments, contact data, search indexes, drafts, and application settings stored exclusively on my local device (and on my mail server via standard sync), so that no application data is transmitted to any server operated by the developer or a third party in the default configuration.

## Blocked by
- `1-default-network-posture` (data locality is a consequence of the default posture)

## Acceptance Criteria
- All message content, metadata, attachments, contact data, search indexes, drafts, and application settings are stored exclusively on the user's local device and (via standard mail-protocol sync) on the user's mail server.
- No application data is stored on or transmitted to any server operated by the application developer or a third party in the default configuration.
- If the application offers a cloud backup feature, it is opt-in, user-directed (the user chooses the destination), and encrypted before transmission.

## Mapping to Epic
- FR-41, FR-42
- AC-11

## HITL / AFK
AFK — no human review needed beyond normal code review.

## Notes
- This story overlaps significantly with story 1 (default network posture) but focuses specifically on data storage rather than network traffic. The key addition is FR-42's requirement for any future cloud backup feature to be opt-in, user-directed, and encrypted.
- On Linux desktop, local data storage is the natural default. This story ensures that no feature inadvertently syncs data to a cloud service.
