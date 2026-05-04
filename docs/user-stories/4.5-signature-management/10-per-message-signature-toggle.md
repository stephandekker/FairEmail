## Parent Feature

#4.5 Signature Management

## What to build

Add a toggle in the compose window that lets the user enable or disable the signature for the current draft, overriding the default behavior (FR-29). The initial state of the toggle is derived from the applicable insertion trigger setting and, if applicable, the reply-once logic (FR-30). Changing the toggle immediately adds or removes the signature from the draft (FR-31). The compose window also provides a way to preview the current identity's signature content (FR-32).

Covers epic sections: §6.6 (US-19, US-20), §7.8 (FR-29 – FR-32).

## Acceptance criteria

- [ ] A signature toggle (e.g. checkbox) is visible in the compose window
- [ ] The toggle's initial state reflects the applicable insertion trigger and reply-once logic (FR-30)
- [ ] AC-10: Unchecking the toggle removes the signature from the draft; re-checking re-inserts it
- [ ] Changes to the toggle take effect immediately in the draft (FR-31)
- [ ] The compose window provides a way to preview the current identity's signature content (FR-32)

## Blocked by

- Blocked by `5-insertion-trigger-settings`

## User stories addressed

- US-19
- US-20
