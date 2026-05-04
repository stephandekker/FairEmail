## Parent Feature

#14.6 System mailto: Handler

## What to build

When smart identity matching is enabled and the application has prior correspondence history with the To recipient from a `mailto:` URI, the application pre-selects the identity that previously communicated with that recipient instead of defaulting to the primary identity. If the URI contains a recognized account-hint parameter, the application honors that hint if the referenced account exists and is composable (FR-15).

This builds on the default identity selection (slice 6) by adding context-aware matching as an enhancement layer.

Covers epic sections: FR-15, FR-16; US-10.

## Acceptance criteria

- [ ] When smart identity matching is enabled and there is prior correspondence with the To recipient, the compose window pre-selects the identity that previously communicated with that recipient
- [ ] When smart identity matching is disabled, the default primary-identity behavior (slice 6) is used
- [ ] When the URI contains a recognized account-hint parameter and the referenced account exists, that account's identity is pre-selected
- [ ] When the URI contains an account hint but the referenced account does not exist, the application falls back to the primary identity without error
- [ ] The user can still override the pre-selected identity before sending

## Blocked by

- Blocked by `6-default-identity-selection`

## User stories addressed

- US-10 (infer appropriate account from context or hint)

## Type

AFK

## Notes

- Open question OQ-2 in the epic asks whether the desktop application should support the Android app's custom `aid` parameter or use a different mechanism (e.g. identity-email-based hint). This is unresolved. Implementers should decide on the hint mechanism and document it. If uncertain, start with identity-email matching from correspondence history only, and defer the custom header parameter to a follow-up decision.
