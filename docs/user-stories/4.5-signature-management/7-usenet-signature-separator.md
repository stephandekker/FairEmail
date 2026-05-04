## Parent Feature

#4.5 Signature Management

## What to build

Add a boolean setting to prepend the Usenet signature separator (`-- ` — two dashes followed by a space, on its own line) before the signature (FR-23). Defaults to disabled (FR-24). When enabled, the signature placement is automatically forced to "at the bottom" (FR-25), consistent with RFC 3676 convention (N-4). The placement setting control should reflect this override (e.g. greyed out or showing "at the bottom" when separator is active).

Covers epic sections: §6.4 (US-15, US-16), §7.6 (FR-23 – FR-25).

## Acceptance criteria

- [ ] A boolean "Usenet signature separator" setting exists, defaulting to disabled
- [ ] When enabled, `-- ` (two dashes + space) appears on its own line immediately before the signature in outgoing messages
- [ ] AC-8: Enabling the separator forces placement to "at the bottom" regardless of the placement setting
- [ ] The placement setting UI reflects the forced override when the separator is active
- [ ] Disabling the separator restores the user's previous placement choice

## Blocked by

- Blocked by `6-signature-placement-options`

## User stories addressed

- US-15
- US-16
