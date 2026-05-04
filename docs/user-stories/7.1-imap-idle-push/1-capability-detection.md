# IMAP IDLE Capability Detection and Persistence

## Parent Feature
#7.1 IMAP IDLE (Push)

## User Story
As a set-and-forget user, when the application connects to my IMAP server, I want it to automatically detect whether the server supports IDLE (and NOTIFY), and persist that information, so that the application can make correct push-vs-poll decisions without me knowing or caring about protocol details.

## Blocked by
*(none — this is the foundational slice)*

## Acceptance Criteria
- On each new connection to an IMAP server, the application queries the server's CAPABILITY response and determines whether IDLE (RFC 2177) is advertised.
- The detected IDLE capability is recorded persistently per account, surviving application restarts.
- The NOTIFY capability (RFC 5465) is also detected and recorded alongside other capabilities, even though it is not exploited yet.
- If the server does not advertise IDLE, the application does not attempt to issue IDLE commands on that connection.
- Capability detection occurs before any push/poll decision is made for the account's folders.
- The persisted capability is updated on every new connection (server upgrades/downgrades are reflected).

## Mapping to Epic
- FR-1, FR-2, FR-3, FR-4
- US-1 (prerequisite), US-3 (prerequisite)
- AC-1 (prerequisite), AC-2 (prerequisite)

## HITL / AFK
AFK — no human review needed beyond normal code review.

## Notes
- This story only covers detection and persistence of capabilities. The actual IDLE session establishment is story 2, and poll fallback is story 3.
- The existing FairEmail codebase checks `iservice.hasCapability("IDLE")` after connect and stores it as `capIdle`. The new implementation should persist this to the account record so it survives across connections and can inform pre-connection UI decisions.
