## Parent Feature

#4.5 Signature Management

## What to build

When the user changes the selected identity in the compose window, the signature in the draft is replaced with the newly selected identity's signature, or removed if the new identity has no signature (FR-33). The signature toggle state is preserved across identity changes — if the user has manually disabled the signature, switching identities does not re-enable it (FR-34).

Covers epic sections: §6.6 (US-21), §7.9 (FR-33, FR-34).

## Acceptance criteria

- [ ] AC-2: Switching the identity in the compose window replaces the signature in the draft with the new identity's signature
- [ ] If the new identity has no signature, the signature block is removed from the draft
- [ ] AC-11: Manually disabling the signature toggle, then switching identities, does not re-enable the signature
- [ ] If the toggle is enabled, the new identity's signature is inserted at the correct placement position

## Blocked by

- Blocked by `10-per-message-signature-toggle`

## User stories addressed

- US-21
