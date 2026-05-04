## Parent Feature

#3.10 External-Image Confirmation

## What to build

Add a per-domain checkbox to the confirmation dialog: "Do not ask this again for [@domain]". This checkbox is disabled by default and becomes enabled only when the per-sender checkbox is checked (FR-13, design note N-3). When both are checked and the user confirms, the sender domain is persistently recorded in the image whitelist alongside the sender address entry.

When a message is opened and its sender domain matches a domain whitelist entry, images load automatically without the confirmation dialog. Domain matching uses the domain part of the sender address (after the `@`). Sender-level and domain-level entries are independent: removing one does not affect the other; either matching is sufficient to auto-load.

## Acceptance criteria

- [ ] The confirmation dialog offers a "Do not ask this again for [@domain]" checkbox showing the actual domain (FR-13)
- [ ] The per-domain checkbox is disabled until the per-sender checkbox is checked (AC-7, FR-13)
- [ ] Confirming with both checkboxes checked persists both sender and domain whitelist entries (FR-20, FR-21)
- [ ] Opening a message from a different address at a whitelisted domain loads images automatically (AC-6, FR-23)
- [ ] Domain matching uses the domain part after `@` (FR-24)
- [ ] Sender and domain whitelist entries are independent — removing one does not affect the other (FR-25)
- [ ] Cancelling the dialog persists nothing (FR-19)

## Blocked by

- Blocked by `4-per-sender-whitelist-in-dialog`

## User stories addressed

- US-10 (per-domain "do not ask again" option in dialog)
- US-16 (whitelisted domain auto-loads images)
- US-17 (sender and domain entries are independent)

## Notes

- **OQ-1 (Subdomain matching):** The epic notes uncertainty about whether whitelisting `@example.com` should also match `@sub.example.com`. The source application uses literal domain matching. This story implements literal matching; if parent-domain matching is desired, it should be specified via a design decision before implementation.
- **OQ-3 (Per-domain gating):** The epic questions whether requiring per-sender before per-domain is the best UX. This story implements the gated behavior per the epic's current specification (FR-13). If the gate is removed, this story's acceptance criteria should be updated.

## Type

AFK
