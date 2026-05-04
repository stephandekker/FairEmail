## Parent Feature

#4.14 Attachments

## What to build

Provide a mechanism in the compose window to record an audio clip directly and have it attached to the current draft, enabling voice notes without leaving the application.

Covers epic sections: US-16, FR-4.

## Acceptance criteria

- [ ] The compose window offers an action to record an audio clip.
- [ ] The recorded audio is immediately attached to the current draft.
- [ ] The audio file has a correct MIME type and a reasonable default filename.

## Blocked by

- Blocked by `1-basic-file-attachment`

## User stories addressed

- US-16

## Notes

- OQ-5 from the epic asks whether the application should specify a preferred audio format (OGG, MP3, WAV) for maximum email-client compatibility, or accept whatever the system recorder produces. This needs a design decision before implementation.
- On a Linux desktop, the system audio recording mechanism may differ significantly from Android's. The epic deliberately does not prescribe the recording implementation.
