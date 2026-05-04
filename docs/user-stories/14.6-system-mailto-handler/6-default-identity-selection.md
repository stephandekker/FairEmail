## Parent Feature

#14.6 System mailto: Handler

## What to build

When a compose window is opened via `mailto:` and the URI provides no account or identity hint, the application selects the user's primary account and its primary identity as the default From address. This follows the same rules as composing from the Unified Inbox (design note N-6). The user must be able to change the From identity within the compose window before sending (FR-17).

This slice wires up the identity-selection logic for the mailto: compose path specifically. It does not include smart matching based on recipient history (that is a separate slice).

Covers epic sections: FR-14, FR-17; AC-8, AC-9.

## Acceptance criteria

- [ ] When a `mailto:` URI provides no account hint, the From field defaults to the primary account's primary identity
- [ ] The identity selection follows the same rules as composing from the Unified Inbox
- [ ] The user can change the From identity in the compose window opened via `mailto:` before sending
- [ ] If the user has multiple accounts, the primary account is consistently selected as the default

## Blocked by

- Blocked by `3-basic-warm-start-compose`

## User stories addressed

- US-8 (default to primary account/identity)
- US-9 (ability to change From identity)

## Type

AFK
