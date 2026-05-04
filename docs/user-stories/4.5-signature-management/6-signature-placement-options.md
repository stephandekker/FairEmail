## Parent Feature

#4.5 Signature Management

## What to build

Expose a signature placement setting with three options: "above the text", "below the text" (default), and "at the bottom" (after quoted material). The compose window positions the signature according to this setting when composing new messages, replies, and forwards. When the user has enabled "write reply below quote" and is replying, placement adapts so the signature appears after the user's reply position (FR-22).

This is a global setting (N-5), not per-identity.

Covers epic sections: §6.3 (US-12, US-13, US-14), §7.5 (FR-20 – FR-22).

## Acceptance criteria

- [ ] A placement setting is available with three options: "above the text", "below the text", "at the bottom"
- [ ] Default placement is "below the text" (FR-21)
- [ ] AC-5: With "below the text", signature appears after user's text and before quoted material in a reply
- [ ] AC-6: With "at the bottom", signature appears after the quoted material in a reply
- [ ] AC-7: With "above the text", signature appears at the top of the message body
- [ ] When "write reply below quote" is active, placement adapts to be consistent with bottom-posting conventions (FR-22)

## Blocked by

- Blocked by `5-insertion-trigger-settings`

## User stories addressed

- US-12
- US-13
- US-14

## Notes

- OQ-3 from the epic notes that the interaction between "write below quote" and signature placement has subtle edge cases (reply vs. forward, forwarded-then-replied messages). The implementation should document its behavior for these edge cases clearly and consider whether the UI needs explanatory text for bottom-posters.
- OQ-6 asks whether placement should be per-identity rather than global. This slice implements the global setting per the epic; per-identity placement could be a future enhancement.
