# Distribution Channel Privacy Parity

## Parent Feature
#10.2 No Third-Party Servers

## User Story
As any user, regardless of which distribution channel I installed the application from, I want the same default privacy posture — no third-party traffic until I opt in — so that choosing a different install source does not silently weaken my privacy.

## Blocked by
- `9-optional-service-gating-framework` (channel parity depends on the gating framework being in place)

## Acceptance Criteria
- All distribution channels ship with the same default privacy posture: no third-party network traffic until the user opts in.
- A distribution channel may restrict the set of available optional services (e.g. a freedom-respecting build may omit proprietary cloud AI endpoints entirely), but no channel expands the set of services enabled by default.
- A freedom-respecting distribution build does not offer proprietary cloud AI endpoints. Its default privacy posture is identical to other builds.
- If a distribution channel includes platform-mandated services (e.g. vendor billing infrastructure), those services are disabled by default within the application's own settings, even if the platform itself may activate them at the OS level.

## Mapping to Epic
- US-22, US-23
- FR-38, FR-39, FR-40
- AC-9

## HITL / AFK
HITL — the definition of which services are included/excluded per channel may require product decision review.

## Notes
- N-4 clarifies that distribution-channel variation is "subtractive, not additive": builds may remove optional services but never add services enabled by default.
- OQ-4 asks about platform-mandated telemetry that the application cannot fully disable. The epic suggests the privacy documentation should disclaim these platform-level behaviors. This is addressed in story 17.
- On Linux desktop, the primary distribution channels are likely: community/GitHub build, Flatpak, Snap, distro package repositories. The privacy invariant must hold across all of these.
