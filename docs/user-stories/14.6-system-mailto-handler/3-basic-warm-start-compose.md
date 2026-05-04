## Parent Feature

#14.6 System mailto: Handler

## What to build

Wire the URI parser (slice 1) to the compose window so that when the application is already running and receives a `mailto:` URI from the operating system, it opens a compose window with the To field pre-populated from the URI. This is the minimum end-to-end tracer bullet: system delivers URI -> application receives it -> parser extracts recipient -> compose window opens with To filled in.

The compose window opened via `mailto:` must behave identically to one opened from within the application (FR-13). At this stage only the To field is wired; full field pre-population is handled in the next slice.

Covers epic sections: FR-10, FR-11 (To only), FR-12, FR-13; AC-2, AC-3.

## Acceptance criteria

- [ ] Setting the application as the default `mailto:` handler and clicking `mailto:user@example.com` in a browser opens the application's compose window with `user@example.com` in the To field
- [ ] The compose window appears within one second when the application is already running (NFR-1)
- [ ] The To field is editable by the user before sending
- [ ] The compose window behaves identically to one opened from within the application (signature, drafts, attachments, etc.)

## Blocked by

- Blocked by `1-mailto-uri-parser`
- Blocked by `2-desktop-entry-registration`

## User stories addressed

- US-4 (click mailto: link, compose opens with recipient)

## Type

AFK
