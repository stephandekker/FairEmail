## Parent Feature

#3.10 External-Image Confirmation

## What to build

When a message containing remote images (URLs requiring a network fetch, typically `http:` or `https:`) is opened, those images must be blocked — no network request is made. Embedded images (content-ID attachments and data-URI images) must continue to display normally. A clear visual indicator must appear on the message view whenever remote images have been blocked, so the user knows images are available.

This is the foundational slice for the entire epic: it establishes the default-blocked state (FR-1, FR-2, FR-3) and the visual indicator (US-3). The blocking preference must be persisted so that it survives restarts.

## Acceptance criteria

- [ ] A freshly installed application displays no remote images in any message (AC-1 partial)
- [ ] No network request for remote image content is initiated for any message opened with default settings (NFR-1)
- [ ] Embedded images (content-ID and data-URI) render normally in blocked state (AC-1 partial, FR-2)
- [ ] A visual indicator is shown on messages that have blocked remote images (US-3)
- [ ] The image-blocking preference is persisted and defaults to "block" (FR-3)
- [ ] Messages with no remote images do not show the blocked-images indicator

## Blocked by

None — can start immediately

## User stories addressed

- US-1 (block remote images by default)
- US-2 (embedded images still display)
- US-3 (visual indicator for blocked images)

## Type

AFK
