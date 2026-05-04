# Offline Provider Discovery During Account Setup

## Parent Feature
#10.2 No Third-Party Servers

## User Story
As any user, when I set up a new account using the quick-setup wizard, I want the application to first attempt to discover my provider's settings from a bundled, offline provider database, so that my email domain is not disclosed to any third-party autoconfiguration service by default.

## Blocked by
- `1-default-network-posture` (this is a specific manifestation of the default no-third-party-traffic posture)

## Acceptance Criteria
- The application ships with a bundled, offline provider database containing known server settings for common mail providers.
- During account setup, provider discovery first attempts the bundled database without any network call to a third party.
- For providers in the bundled database, setup completes without contacting any external autoconfiguration service.
- Manual server configuration is always available as an alternative to automated discovery, allowing the user to avoid all third-party contact during setup.

## Mapping to Epic
- US-11
- FR-14, FR-15, FR-17
- AC-4

## HITL / AFK
AFK — no human review needed beyond normal code review.

## Notes
- This story focuses on the privacy aspect of provider discovery (no third-party traffic). The bundled provider database itself is likely implemented as part of epic 1.7 (Pre-Installed Provider Database) and epic 1.3 (Quick Setup Wizard). This story ensures the privacy contract is met: bundled lookup first, no external calls by default.
- If the provider database stories from epics 1.3/1.7 already exist, this story may be partially satisfied by them. The key addition here is the explicit privacy guarantee and the ordering constraint (bundled first, external only with consent).
