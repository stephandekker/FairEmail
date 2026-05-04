# Password Propagation to Identities

## Parent Feature

#1.4 Manual Server Configuration

## What to build

When saving an existing account with a changed password, offer the user the option to propagate the new password to all identities (SMTP configurations) associated with that account. If the user accepts, the SMTP password on each associated identity is updated to match the new inbound password.

Covers epic sections: FR-44.

## Acceptance criteria

- [ ] When saving an existing account with a changed password, the application offers to propagate the new password to associated identities
- [ ] If the user accepts, the SMTP password on all associated identities is updated
- [ ] If the user declines, identity passwords remain unchanged
- [ ] The propagation prompt is only shown when the password has actually changed

## Blocked by

- Blocked by `22-edit-existing-account`
- Blocked by `16-smtp-identity-config`

## User stories addressed

- US-35 (propagate password changes to identities)

## Notes

Open question OQ-1 from the epic: should propagation apply only to identities whose SMTP password previously matched the old account password, or to all identities unconditionally? The source application propagates unconditionally, which may be surprising if the user intentionally has a different SMTP password on some identities. This design decision should be reviewed.
