# Incidental Disclosure Minimization (User-Agent, Timezone, Locale)

## Parent Feature
#10.2 No Third-Party Servers

## User Story
As a privacy-conscious user, I want the option to use a generic user-agent string when fetching remote content and to suppress timezone and locale information in outgoing message headers, so that my email client's identity and my location are not unnecessarily disclosed.

## Blocked by
- `2-remote-content-blocking` (the generic user-agent option applies when remote content is loaded)

## Acceptance Criteria
- The application offers a setting to use a generic user-agent string when fetching remote content (for cases where the user has opted to load remote images), so that the email client's identity is not disclosed to remote servers.
- The application offers a setting to suppress or genericize timezone and locale information in outgoing message headers, to reduce fingerprinting surface.
- Both settings are optional — they enhance privacy but are not required for the core no-third-party guarantee.

## Mapping to Epic
- FR-43, FR-44

## HITL / AFK
AFK — no human review needed beyond normal code review.

## Notes
- These are hardening measures that reduce incidental information leakage even when the user has opted to load remote content or send messages. They do not affect the default no-third-party posture (which blocks remote content entirely) but improve privacy for users who selectively enable remote content.
- The epic does not specify default values for these settings. A reasonable default would be to enable the generic user-agent by default (since it has no downside) and leave timezone suppression as opt-in (since some recipients may need timezone information).
