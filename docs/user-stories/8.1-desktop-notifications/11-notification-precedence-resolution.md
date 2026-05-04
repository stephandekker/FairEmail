## Parent Feature

#8.1 Desktop Notifications

## What to build

Implement the full notification precedence chain so that when a new message arrives, the notification subsystem resolves the effective notification setting by checking, in order from highest to lowest priority: (1) rule-based override, (2) per-sender setting, (3) per-folder setting, (4) per-account setting, (5) global default. The most specific applicable setting wins. This story wires together the per-account, per-folder, and per-sender settings from the previous stories into a single, predictable resolution path. The rule-based override slot is an interface contract for the Rules & Automation feature group (FR-44) — this story defines the hook point but does not implement rule evaluation itself.

Covers epic sections: §7.7 (FR-29), §7.11 (FR-44), design note N-3.

## Acceptance criteria

- [ ] The precedence order is: rule-based override > per-sender > per-folder > per-account > global default (US-24)
- [ ] When multiple levels conflict, the highest-precedence applicable setting wins
- [ ] A rule-based override interface/hook point exists, even if no rules are defined yet
- [ ] Filter rules (from the Rules & Automation feature group) can suppress or silence notifications for matching messages via this hook
- [ ] The resolution logic is testable in isolation (given a message and configuration, predict whether it will notify)

## Blocked by

- Blocked by `10-per-sender-notification-overrides`

## User stories addressed

- US-24 (clear and predictable precedence order for notification settings)

## Notes

- FR-44 references filter rules from a separate feature group. This story provides the integration point (the rule-based override slot at the top of the precedence chain) but does not implement the rule engine. If the rule engine is not yet available, the slot is a no-op that can be wired up later.
