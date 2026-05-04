# Chip Encryption-Status Indicator

## Parent Feature
#4.11 Recipient Autocomplete

## User Story
As any user, I want each recipient chip to indicate whether a PGP key, an S/MIME certificate, or both are available for that address, so that I can see my encryption options before sending.

## Blocked by
- `4-chip-rendering`

## Acceptance Criteria
- Each chip displays an encryption-status indicator showing whether a PGP key is available, an S/MIME certificate is available, both are available, or neither is available for that recipient's email address.
- The indicator is visually distinct for each state (PGP-only, S/MIME-only, both, none).
- The indicator updates if the encryption key state changes (e.g. a key is imported while composing).
- The indicator has an appropriate screen-reader label describing the encryption status.

## HITL/AFK Classification
**HITL** — the indicator iconography and visual design need human review to ensure clarity and consistency with the application's design language.

## Notes
- This story depends on the application's encryption subsystem being able to look up key/certificate availability by email address. If that subsystem is not yet available, this story should be deferred or stubbed.
- N-6 in the epic explains the rationale: giving the user immediate feedback about encryption options reduces accidental unencrypted sends.
