## Parent Feature

#12.1 Local Contact Store

## What to build

Implement automatic periodic purge of stale contacts. The purge removes contacts that satisfy all of the following conditions (FR-30):

- Last-contacted is older than a user-configurable threshold (default: one month)
- Times-contacted is below a user-configurable threshold (default: one)
- State is not "favorite"
- Type is sent-to or received-from (not junk or no-junk)

The purge runs as part of the application's regular maintenance cycle (FR-31). The user may also trigger maintenance manually. The age and frequency thresholds are configurable in settings (FR-32).

## Acceptance criteria

- [ ] Contacts meeting all purge conditions are automatically removed after the purge interval elapses (AC-14)
- [ ] Favorite contacts are never removed by automatic purge, regardless of age or frequency (AC-15, FR-30c)
- [ ] Junk and no-junk contacts are exempt from purge (FR-30d)
- [ ] The purge age threshold is configurable (default: one month) (FR-32)
- [ ] The purge frequency threshold is configurable (default: one contact) (FR-32)
- [ ] The purge runs as part of the regular maintenance cycle, not requiring user action (FR-31)

## Blocked by

- Blocked by 1-contact-record-and-learn-on-send (requires the contact store)
- Blocked by 6-favorite-and-ignored-states (purge must respect favorite exemption)

## User stories addressed

- US-28 (automatic purge of stale, rarely-used contacts)
- US-29 (favorites exempt from purge)
- US-30 (junk contacts exempt from purge)

## Notes

Open question OQ-6 asks whether the default purge age threshold should be longer for a desktop application (e.g. three months instead of one month). This story implements the one-month default as specified in FR-30, but the team may want to adjust for desktop usage patterns.
