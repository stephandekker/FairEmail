# Communication Service Opt-In Gates (Translation, AI/LLM, Spell-Check)

## Parent Feature
#10.2 No Third-Party Servers

## User Story
As a power user, I want cloud-based translation, AI/LLM summarization, and spell-checking/grammar services to each be independently toggleable and disabled by default, so that my message text is never sent to a third party without my explicit consent.

## Blocked by
- `9-optional-service-gating-framework` (these services use the generic gating framework)

## Acceptance Criteria
- Cloud translation services are disabled by default. Enabling requires opt-in with disclosure. Translation is activated only by an explicit user action on a specific message, not automatically.
- AI/LLM services are disabled by default. Enabling requires the user to configure an endpoint and/or API key. Both cloud AI providers and user-configured local model endpoints are supported.
- Cloud spell-checking/grammar services are disabled by default. Enabling requires opt-in with disclosure of what text is transmitted.
- Each service has its own independent toggle — enabling one does not enable the others.
- Each toggle appears in the consolidated optional-services view with a description of what data is sent and to whom.
- Disabling any of these services takes effect immediately.

## Mapping to Epic
- US-14
- FR-24, FR-25, FR-29

## HITL / AFK
AFK — no human review needed beyond normal code review.

## Notes
- These three services are grouped because they all transmit user-authored or received text to external services. Each still has its own independent toggle per FR-18.
- FR-25 distinguishes between cloud AI providers and user-configured local model endpoints. A local model endpoint does not contact a "third-party server" in the epic's sense, but the gating framework should still require the user to explicitly configure it.
