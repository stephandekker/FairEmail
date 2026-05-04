# User Story: Provider-Scoped Dictionary Entries

## Parent Feature
#2.2 Special-Folder Auto-Detection

## Blocked by
- `4-tier2-name-heuristic-detection`

## Description
As a user of a provider with non-standard folder naming, I want the name-heuristic dictionary to include entries scoped to specific server hostnames, so that provider-specific folder names are correctly detected without causing false positives on other servers.

## Motivation
Some folder names are ambiguous across providers — a name that means "Sent" on one provider might be unrelated on another. Host-scoped entries let the dictionary be precise without being overly conservative.

## Acceptance Criteria
- [ ] Dictionary entries may optionally be scoped to a specific server hostname. _(FR-12)_
- [ ] A host-scoped entry is only evaluated when the account's server matches the specified host.
- [ ] Non-scoped (global) entries continue to apply to all servers.
- [ ] Host-scoped entries do not produce false positives on other servers. _(FR-12, N-4)_
- [ ] The scoping mechanism is transparent to the rest of the matching engine — it filters candidates before scoring, not after.

## Sizing
Small — an extension to the dictionary data structure and a filter in the matching pipeline.

## HITL / AFK
AFK — mechanical extension of the dictionary mechanism.

## Notes
- The Android code's `TypeScore` class (EntityFolder.java lines 559–575) includes a `host` field for this purpose. Examples include entries for mailo.com, laposte.net, and Verizon.
- This story could be folded into Story 4 if the team prefers fewer, larger stories. It is separated here because it is an independently testable behaviour on top of the base heuristic.
