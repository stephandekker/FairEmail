# Identity and Reputation Service Opt-In Gates (Avatars, Favicons, Blocklists, Geolocation)

## Parent Feature
#10.2 No Third-Party Servers

## User Story
As a power user, I want avatar/contact-photo fetching, favicon/brand-indicator fetching, spam blocklist lookups, and IP geolocation lookups to each be independently toggleable and disabled by default, so that sender information and IP addresses from my emails are not disclosed to third parties without my consent.

## Blocked by
- `9-optional-service-gating-framework` (these services use the generic gating framework)

## Acceptance Criteria
- Avatar/contact-photo services (fetching sender profile images using a hash of the sender's email address) are disabled by default with opt-in disclosure.
- Favicon/brand-indicator services (fetching website icons or brand logos from sender domains) are disabled by default with opt-in disclosure.
- Spam blocklist lookups (querying third-party DNS-based blocklists for sender reputation) are disabled by default with opt-in disclosure.
- IP geolocation lookups (resolving IP addresses from message headers to geographic locations) are disabled by default; activated only by an explicit user action per lookup, not automatically.
- Each service has its own independent toggle.
- Each toggle appears in the consolidated optional-services view.
- Disabling any of these services takes effect immediately.

## Mapping to Epic
- US-14
- FR-26, FR-27, FR-28, FR-35

## HITL / AFK
AFK — no human review needed beyond normal code review.

## Notes
- OQ-6 asks whether favicon/BIMI fetching from the sender's domain should be treated as "third-party server" contact. The epic treats it as opt-in, which is the conservative interpretation. This story follows that interpretation.
- OQ-3 asks about DNS-based blocklist queries. These use DNS rather than HTTPS but still contact third-party infrastructure. The epic treats them as opt-in (FR-28), and this story follows that.
- Geolocation is per-lookup opt-in (FR-35), not a persistent toggle. The framework should support both persistent toggles and per-action consent patterns.
