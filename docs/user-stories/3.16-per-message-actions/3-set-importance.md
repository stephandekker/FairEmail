## Parent Feature

#3.16 Per-Message Actions

## What to build

Allow users to set a message's importance to high, normal, or low. Display a visual indicator (up-arrow for high, down-arrow for low, none for normal) in the message list and message view. Recognise sender-set priority from standard headers (`Priority`, `X-Priority`, `X-MSMail-Priority`) on receipt. Maintain local importance separately from sender importance (Design Note N-5); local setting governs display and sort. Support sorting by importance. Optionally propagate local importance to the server as IMAP keywords `$HighImportance` / `$LowImportance` (FR-24 through FR-29).

## Acceptance criteria

- [ ] User can set importance to high, normal, or low on any message
- [ ] High importance shows an up-arrow icon; low shows a down-arrow icon (AC-8)
- [ ] Sorting by importance places high above normal above low (AC-8)
- [ ] Incoming messages with `X-Priority: 1` display as high importance automatically
- [ ] Locally set importance overrides sender priority for display purposes
- [ ] When the "propagate to server" option is enabled, setting high importance adds the `$HighImportance` IMAP keyword
- [ ] Importance indicators persist across restarts

## Blocked by

None — can start immediately.

## User stories addressed

- US-21 (set importance to high/normal/low)
- US-22 (visual indicator in message list)
- US-23 (sort by importance)
- US-24 (optionally propagate to server as keywords)
- US-25 (recognise sender-set priority headers)
