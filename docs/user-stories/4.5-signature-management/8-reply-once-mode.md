## Parent Feature

#4.5 Signature Management

## What to build

Add a boolean "reply-once" setting (FR-15). When enabled alongside "signature on reply", the signature is added to a reply only if no outbound message has previously been sent in the same conversation thread (FR-16, FR-17). Reply-once applies only to reply/reply-all, not to new messages or forwards (FR-18). The setting is disabled (greyed out) when "signature on reply" is off (FR-19).

Detection of "already replied" is based on the existence of any outbound message in the conversation thread (N-3).

Covers epic sections: §6.2 (US-10, US-11), §7.4 (FR-15 – FR-19).

## Acceptance criteria

- [ ] A "reply-once" boolean setting exists in send settings
- [ ] The setting is greyed out / disabled when "signature on reply" is off (FR-19)
- [ ] AC-4: With reply-once enabled, the first reply in a thread includes the signature; a second reply in the same thread does not
- [ ] Reply-once does not affect new messages or forwards (FR-18)
- [ ] The check for "already replied" is based on outbound messages in the thread

## Blocked by

- Blocked by `5-insertion-trigger-settings`

## User stories addressed

- US-10
- US-11
