## Parent Feature

#1.3 Quick Setup Wizard

## What to build

When auto-detection fails entirely (no provider settings could be determined from any strategy), the wizard displays an informative message and offers a clear, one-click path to manual setup (FR-23).

**Behavior:**
- Display a message explaining that the provider could not be auto-detected.
- Offer a one-click path to manual setup (FR-23), which carries over entered data (name, email, password) per slice 4's behavior.
- The message is non-technical and does not expose raw error details by default (FR-25).
- A link to general support/FAQ is offered (FR-24).

This slice connects the detection pipeline's "no results" outcome to a user-facing fallback screen.

## Acceptance criteria

- [ ] When all detection strategies fail, the wizard shows a clear message that the provider could not be detected (FR-23)
- [ ] A one-click path to manual setup is offered (FR-23)
- [ ] The path to manual setup carries over name, email, and password (FR-36)
- [ ] The message is non-technical (FR-25)
- [ ] A general support/FAQ link is offered (FR-24)

## Blocked by

- Blocked by 4-manual-setup-escape-hatch
- Blocked by 6-detection-progress-feedback

## User stories addressed

- US-18 (informative message and clear path to manual setup on detection failure)
