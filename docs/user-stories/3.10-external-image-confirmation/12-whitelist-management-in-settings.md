## Parent Feature

#3.10 External-Image Confirmation

## What to build

Add whitelist management capabilities to the privacy settings screen:

1. A way to review existing per-sender and per-domain whitelist entries (US-18).
2. The ability to clear all whitelist entries (US-18).
3. Automatic clearing of all per-sender and per-domain whitelist entries when the user disables the confirmation toggle (FR-28, design note N-6).

Sender-level and domain-level entries must be independently removable (US-17).

## Acceptance criteria

- [ ] The privacy settings screen allows reviewing per-sender and per-domain whitelist entries (US-18)
- [ ] Individual whitelist entries can be removed; removing a sender entry does not affect domain entries and vice versa (US-17, FR-25)
- [ ] All whitelist entries can be cleared at once (US-18)
- [ ] Disabling the confirmation toggle in settings clears all whitelist entries (AC-10, FR-28)
- [ ] After clearing, previously whitelisted senders/domains no longer auto-load images

## Blocked by

- Blocked by `5-sender-whitelist-auto-load`
- Blocked by `6-per-domain-whitelist-in-dialog`
- Blocked by `11-privacy-settings-screen`

## User stories addressed

- US-17 (sender and domain entries are independent)
- US-18 (review and clear whitelist entries)
- US-22 (disabling confirmation clears whitelist)

## Notes

- **OQ-2 (Whitelist review UI):** The epic notes that the source application does not offer a UI to view or selectively remove individual entries — only bulk clear. This story includes individual removal as specified by US-17/US-18, but the exact UI design (list view, search, etc.) is a design decision that may warrant review.

## Type

AFK
