# Field Validation Rules

## Parent Feature

#1.4 Manual Server Configuration

## What to build

Add client-side validation rules to the inbound account configuration form. The hostname and username fields are required (non-empty) when synchronization is enabled. The password is required unless a client certificate is selected or "allow insecure" is enabled (those features come in later stories; for now password is always required). Leading/trailing whitespace is trimmed from hostname and username before use. If the password contains leading/trailing whitespace or control characters, a non-blocking warning is shown. The port field accepts only numeric input (max 5 digits).

Validation errors must prevent save and display clear inline feedback.

Covers epic sections: FR-17 through FR-22.

## Acceptance criteria

- [ ] Hostname is required; saving with an empty hostname shows an error
- [ ] Username is required; saving with an empty username shows an error
- [ ] Password is required (for now; later stories relax this when cert or insecure mode is active)
- [ ] Leading and trailing whitespace is trimmed from hostname and username before use
- [ ] Password with leading/trailing whitespace or control characters shows a non-blocking warning (password is still accepted)
- [ ] Port field accepts only numeric input, maximum 5 digits
- [ ] Validation errors display clear inline feedback and prevent save

## Blocked by

- Blocked by `1-create-imap-account`

## User stories addressed

- US-1 (implicit — valid configuration required to create account)

## Notes

FR-18 says username is not required when "allow insecure connections" is enabled. FR-19 says password is not required when insecure is enabled or a client certificate is selected. Those relaxations should be wired up when the respective features (stories 9 and 11) are implemented. This story enforces the strict defaults.
