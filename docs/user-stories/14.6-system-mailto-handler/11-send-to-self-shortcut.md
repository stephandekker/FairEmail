## Parent Feature

#14.6 System mailto: Handler

## What to build

A quick-action or shortcut within the application that composes a new message with the user's own primary email address pre-filled in the To field, bypassing the need to type or paste the address. This uses the same compose flow as a standard `mailto:` invocation (FR-23) but with a hardcoded recipient (design note N-5). The From identity is the user's primary identity.

This is a convenience feature layered on top of the mailto: compose infrastructure for the common "email myself a quick note" pattern.

Covers epic sections: FR-22, FR-23; AC-13.

## Acceptance criteria

- [ ] The application provides a discoverable quick-action or shortcut to "send to self"
- [ ] Activating the shortcut opens a compose window with the user's own primary email address in the To field
- [ ] The From identity is the user's primary identity
- [ ] The compose window follows the same flow as a standard `mailto:` invocation (signature, drafts, attachments, identity switching)
- [ ] The shortcut works when accounts are configured; when no account is configured, it shows the same error as a regular mailto: invocation (slice 9)

## Blocked by

- Blocked by `6-default-identity-selection`
- Blocked by `9-no-account-error-handling`

## User stories addressed

- US-17 (send-to-self shortcut)

## Type

AFK
