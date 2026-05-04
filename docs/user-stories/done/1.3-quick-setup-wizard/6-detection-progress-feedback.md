## Parent Feature

#1.3 Quick Setup Wizard

## What to build

Real-time progress feedback during provider detection and connectivity checking, so the user knows the application is working and not frozen (FR-14).

**Behavior:**
- While the wizard is detecting the provider and checking connectivity, it displays progress messages indicating which strategy is currently being attempted.
- Example messages: "Looking up DNS records...", "Checking autoconfig...", "Scanning ports on [host]...", "Connecting to IMAP server...", "Authenticating..." (FR-14).
- The UI must never appear frozen during detection — progress updates should be frequent enough to convey activity.

This slice wires the detection and connectivity pipeline (from other slices) into a progress display in the wizard UI. It covers the progress UI component and the callback/event mechanism that detection strategies use to report their current state.

## Acceptance criteria

- [ ] Real-time progress messages are shown during detection (AC-16)
- [ ] The UI does not appear frozen at any point during detection (AC-16)
- [ ] Progress messages indicate which detection strategy is being attempted (FR-14)
- [ ] Progress messages update as the wizard moves through detection strategies
- [ ] Progress indicators are accessible to screen readers (NFR-8)

## Blocked by

- Blocked by 1-wizard-ui-with-input-validation
- Blocked by 2-bundled-provider-database

## User stories addressed

- US-10 (real-time progress during provider detection)
