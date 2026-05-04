## Parent Feature

#1.3 Quick Setup Wizard

## What to build

A clearly labeled button on the wizard's main screen that allows the user to switch to full manual setup at any time. This is the escape hatch for users whose providers cannot be auto-detected or who prefer to enter server details directly.

**Behavior:**
- The button is visible on the wizard's main screen at all times (FR-35).
- Switching to manual setup carries over any information the user has already entered — name, email, and password — so the user does not have to re-type it (FR-36).

This slice requires that the manual setup screen (epic 1.4) exists or that a stub/placeholder is available to navigate to. The slice itself is responsible for the button, the navigation, and the data hand-off.

## Acceptance criteria

- [ ] A clearly labeled "Manual setup" button is visible on the wizard screen at all times (FR-35)
- [ ] Clicking the button navigates to the manual setup screen (AC-12)
- [ ] Name, email, and password are pre-filled in the manual setup screen (AC-12, FR-36)
- [ ] The button is keyboard-accessible and screen-reader-labeled (NFR-8)

## Blocked by

- Blocked by 1-wizard-ui-with-input-validation

## User stories addressed

- US-26 (switch to manual setup at any time)

## Notes

- This slice depends on epic 1.4 (Manual Server Configuration) providing a screen to navigate to. If epic 1.4 is not yet implemented, a stub or placeholder screen may be needed. This cross-epic dependency should be coordinated during implementation planning.
