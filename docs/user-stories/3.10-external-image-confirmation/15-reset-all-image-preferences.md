## Parent Feature

#3.10 External-Image Confirmation

## What to build

Provide a mechanism (in the privacy settings screen or via a general "reset asked questions" action) to reset all image-related preferences back to factory defaults. This includes: the master block toggle (→ on), the confirmation toggle (→ on), the original-view auto-show toggle (→ off), and all per-sender and per-domain whitelist entries (→ cleared).

## Acceptance criteria

- [ ] A reset action is available in settings (FR-34)
- [ ] After reset, blocking is enabled, confirmation is enabled, original-view auto-show is disabled (AC-17)
- [ ] After reset, all per-sender and per-domain whitelist entries are cleared (AC-17)
- [ ] After reset, messages that previously auto-loaded images (due to whitelist) no longer do so
- [ ] The reset action requires confirmation to prevent accidental use

## Blocked by

- Blocked by `11-privacy-settings-screen`
- Blocked by `12-whitelist-management-in-settings`

## User stories addressed

- US-28 (reset all image-related choices to defaults)

## Type

AFK
