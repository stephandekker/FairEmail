## Parent Feature

#12.1 Local Contact Store

## What to build

Enable contact group management. Each contact may belong to at most one group, represented as a string label (FR-26).

The user can:
- Assign or change a contact's group label via the contact edit interface (already available from story 5).
- View a list of all defined groups with the count of contacts in each (FR-27).
- Filter the contact list to show only contacts belonging to a selected group (FR-28).

## Acceptance criteria

- [ ] A contact can be assigned to a group via the edit interface
- [ ] A groups overview shows all defined groups with contact counts (FR-27, US-23)
- [ ] The contact list can be filtered by a single group (FR-28, US-22)
- [ ] Changing a contact's group persists across application restart
- [ ] Removing a group assignment (setting to empty) is supported

## Blocked by

- Blocked by 5-edit-and-delete-contacts (group assignment uses the contact edit interface)

## User stories addressed

- US-21 (assign group label to a contact)
- US-22 (filter contacts by group)
- US-23 (view all groups with counts)

## Notes

Open question OQ-1 (group cardinality) asks whether the desktop app should support multiple groups per contact (tagging) vs. the single-group model from the source application. The epic specifies single-group (FR-26). This story implements single-group. If the team decides to support multiple groups, this story would need revision.
