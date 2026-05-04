# First-Run Discoverability Hint

## Parent Feature

#3.1 Unified Inbox

## What to build

On first run, display a brief on-screen hint telling the user that they can curate the unified stream by acting on folders in the folder list (US-29, NFR-5). The hint must be dismissible and must not return for that user once dismissed. This is purely a UI overlay / tooltip — it does not change any behavior.

## Acceptance criteria

- [ ] On first launch (or first account setup), a hint is displayed explaining how to curate the Unified Inbox.
- [ ] The hint references the folder-list context action for toggling membership.
- [ ] The hint is dismissible (e.g. "Got it" button or click-away).
- [ ] Once dismissed, the hint does not reappear for that user.
- [ ] The hint is accessible (keyboard-dismissible, screen-reader compatible).

## Blocked by

- Blocked by `5-toggle-membership-context-action`

## User stories addressed

- US-29 (first-run hint about curating the unified stream)
