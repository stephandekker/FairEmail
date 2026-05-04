## Parent Feature

#12.1 Local Contact Store

## What to build

Add user preferences to independently enable or disable contact learning from sent messages and from received messages (FR-12). Each direction can be toggled independently.

When learning from sent messages is disabled, sending an email does not create or update sent-to contacts. When learning from received messages is disabled, receiving an email does not create or update received-from contacts.

The epic's design note N-5 specifies conservative defaults: learning from sent mail is enabled by default, learning from received mail is disabled by default.

All contact data remains strictly local — this is enforced architecturally, not as a toggle (FR-12, US-32, NFR-4).

## Acceptance criteria

- [ ] A user preference exists to disable contact learning from sent messages (FR-12, US-31)
- [ ] A user preference exists to disable contact learning from received messages (FR-12, US-31)
- [ ] With sent learning disabled, sending an email does not create or update sent-to contacts
- [ ] With received learning disabled, receiving an email does not create or update received-from contacts
- [ ] Default: sent learning enabled, received learning disabled (N-5)
- [ ] Preferences persist across application restart

## Blocked by

- Blocked by 1-contact-record-and-learn-on-send (requires sent learning path)
- Blocked by 2-learn-on-receive (requires received learning path)

## User stories addressed

- US-31 (independent enable/disable for sent vs. received learning)
- US-32 (all contact data stays local)

## Notes

Open question OQ-8 asks whether the desktop app should enable both directions by default for a richer out-of-box experience, rather than the conservative default from the source app. This story implements the conservative default (N-5) but flags the question for team decision.
