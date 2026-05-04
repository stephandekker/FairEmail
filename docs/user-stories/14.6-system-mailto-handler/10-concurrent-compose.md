## Parent Feature

#14.6 System mailto: Handler

## What to build

When multiple `mailto:` URIs are received in rapid succession (e.g. the user clicks several links quickly), each URI must open its own separate compose window or tab. No compose request should be lost or silently dropped. This requires the URI reception and compose-window creation path to handle concurrent/queued invocations correctly.

Covers epic sections: NFR-3; AC-14.

## Acceptance criteria

- [ ] Two `mailto:` links clicked in rapid succession each produce their own compose window; neither is lost
- [ ] Three or more `mailto:` URIs received in quick succession each produce their own compose window
- [ ] Each compose window is independently populated with the correct fields from its respective URI
- [ ] The application does not crash or hang when processing concurrent mailto: requests
- [ ] Works in both warm-start and cold-start scenarios (if the first URI triggers a cold start, a second URI arriving during startup is not lost)

## Blocked by

- Blocked by `5-cold-start-compose`

## User stories addressed

- US-4, US-5 (implicitly -- each concurrent invocation must work correctly)

## Type

AFK

## Notes

- NFR-3 does not specify an upper bound on concurrent requests. The implementation should handle a reasonable number (e.g. 5-10) without degradation. Whether there should be a practical limit is an implementation decision.
- The cold-start + rapid-succession case (final acceptance criterion) may be architecturally tricky. If the first URI triggers app launch and a second arrives before initialization completes, both must eventually produce compose windows. If this proves too complex for a single slice, it could be deferred to a follow-up, but ideally it is addressed here.
