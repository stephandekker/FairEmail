## Parent Feature

#12.1 Local Contact Store

## What to build

When the user marks a sender as blocked/junk, record the address as a junk contact in the local store. When the user whitelists an address (marks as "no-junk"), record a no-junk contact that overrides any junk classification (FR-35 context, design note N-4).

Junk and no-junk contacts are stored as separate contact types, distinct from sent-to and received-from records for the same address. This ensures blocking does not interfere with existing correspondence records.

Provide a filtered view (or separate tab) of blocked/junk contacts, showing the count of blocked addresses per account (FR-25). The user can view and manage their block list from this view.

## Acceptance criteria

- [ ] Blocking a sender records the address as a junk contact (AC-17)
- [ ] Whitelisting (unblocking) an address records a no-junk contact (AC-17)
- [ ] Junk contacts are viewable as a separate filtered list or tab (FR-25)
- [ ] The blocked contact count per account is displayed (AC-18)
- [ ] Junk and no-junk records are independent of sent-to / received-from records for the same address (N-4)

## Blocked by

- Blocked by 4-contact-list-view-and-search (requires the contact list infrastructure)

## User stories addressed

- US-24 (record junk contact on block)
- US-25 (view blocked/junk contacts list)
- US-26 (whitelist / no-junk override)
