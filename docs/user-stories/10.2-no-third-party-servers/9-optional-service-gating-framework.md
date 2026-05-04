# Optional Service Opt-In Gating Framework

## Parent Feature
#10.2 No Third-Party Servers

## User Story
As any user, I want every feature that contacts a third-party server to be governed by an independent opt-in setting that defaults to disabled, with a clear description of what data is sent and to whom, so that I can make informed choices and control exactly which third parties receive my data.

## Blocked by
- `1-default-network-posture` (the framework enforces the default posture for all optional services)

## Acceptance Criteria
- Every feature that contacts a third-party server has its own independent opt-in setting that defaults to disabled.
- Each opt-in setting is accompanied by a visible description in the settings interface that names the third-party service, describes what data is transmitted, and links to or references the service's privacy policy.
- Enabling an optional service requires an affirmative user action (e.g. toggling a switch, confirming a dialog). Services are never enabled implicitly as a side effect of another action.
- When enabling a service for the first time, a one-time confirmation tells the user what data will be shared and with whom.
- Disabling a previously enabled service takes effect immediately: the application ceases contacting that service and retains no authorization state that could cause it to resume without re-enablement.
- The settings interface provides a single, consolidated location where the user can review the state of all optional-service toggles.

## Mapping to Epic
- US-13, US-14, US-15, US-16
- FR-18, FR-19, FR-20, FR-21, FR-22
- AC-7, AC-8, AC-12, AC-15

## HITL / AFK
AFK — no human review needed beyond normal code review.

## Notes
- This story establishes the generic framework/pattern for gating optional services. Individual service implementations (stories 10–13) use this framework to gate their specific third-party calls.
- The "consolidated location" (FR-22) could be a dedicated "Privacy" or "Optional Services" section in settings, or a panel that aggregates toggles from various settings sections. The exact design is not prescribed by the epic.
- G5 calls for the architecture to make it "structurally difficult" for new features to silently introduce third-party network calls without opt-in gating. This story should establish patterns/conventions that make the opt-in gate the path of least resistance for developers adding new external calls.
