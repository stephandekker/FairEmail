## Parent Feature

#14.6 System mailto: Handler

## What to build

The application declares itself as a candidate handler for `mailto:` URIs in its system-integration metadata so that the operating system can discover it and offer it as a choice (or default) when the user activates a `mailto:` link. The registration must persist across application restarts and upgrades without requiring user re-intervention.

This slice covers the passive declaration side only (FR-1 through FR-3). It does not cover actually receiving and processing a URI -- that is wired up in subsequent slices. The application should not forcibly claim default status; it declares capability and lets the user or system choose (per design note N-1).

Covers epic sections: FR-1, FR-2, FR-3; AC-1, AC-15.

## Acceptance criteria

- [ ] The application appears in the operating system's default-application chooser for `mailto:` URIs after installation
- [ ] The user can set the application as the default `mailto:` handler through system settings
- [ ] The registration persists across application restarts -- the user does not need to re-register
- [ ] The registration persists across application upgrades
- [ ] The application does not forcibly override another application's existing default handler status

## Blocked by

None - can start immediately

## User stories addressed

- US-1 (declare as candidate handler)
- US-2 (settable as default through system settings)
- US-3 (registration persists across upgrades and restarts)

## Type

AFK

## Notes

- Open question OQ-4 (default vs. opt-in registration) and OQ-5 (XDG compliance specifics) are unresolved in the epic. This story implements passive declaration per design note N-1. Whether to actively call `xdg-settings set` on first run is a UX decision that should be resolved before or during implementation; if unresolved, default to passive-only.
