## Parent Feature

#4.5 Signature Management

## What to build

Add three independent boolean settings controlling whether a signature is inserted for: (a) new messages, (b) replies and reply-all, (c) forwards. All three default to enabled (FR-13). For resend operations, signatures are not inserted (FR-14). These are global send settings, not per-identity (N-5).

With this slice, the compose window respects the trigger settings — if "signature on reply" is disabled, replying to a message does not insert the signature, even though the identity has one.

Covers epic sections: §6.2 (US-8, US-9), §7.3 (FR-12 – FR-14).

## Acceptance criteria

- [ ] Three independent boolean toggles exist in send settings: "signature on new", "signature on reply", "signature on forward"
- [ ] All three default to enabled (FR-13)
- [ ] Composing a new message inserts/omits the signature according to the "signature on new" toggle
- [ ] Replying inserts/omits the signature according to the "signature on reply" toggle
- [ ] Forwarding inserts/omits the signature according to the "signature on forward" toggle
- [ ] Resend operations do not insert a signature regardless of settings (FR-14)
- [ ] AC-3: With "signature on new" enabled and "signature on reply" disabled, a new message includes the signature and a reply does not

## Blocked by

- Blocked by `1-basic-signature-storage-and-insertion`

## User stories addressed

- US-8
- US-9
