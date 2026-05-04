# User Story: Inbox Guaranteed Detection

## Parent Feature
#2.2 Special-Folder Auto-Detection

## Blocked by
- `1-role-taxonomy-and-folder-data-model`
- `2-tier1-imap-special-use-detection`

## Description
As any user, I want the application to always identify an Inbox for every account — by matching the canonical "INBOX" name (case-insensitive) if present, or by synthesizing a default Inbox entry if no folder can be identified — so that incoming mail always has a home and account setup always succeeds.

## Motivation
The Inbox is the single non-optional folder. Without it, fundamental operations (receiving mail, unified inbox) have no target. This story guarantees every account has an Inbox regardless of server behaviour.

## Acceptance Criteria
- [ ] Any folder whose name matches "INBOX" (case-insensitive) is assigned the Inbox role, regardless of any other attributes. _(FR-7, AC-7)_
- [ ] The INBOX name match takes precedence even if the folder lacks SPECIAL-USE attributes. _(FR-7)_
- [ ] If no folder in the server's list resolves to Inbox (neither via Tier 1 metadata nor via the canonical name match), the application synthesizes or assumes a default Inbox entry so the account remains functional. _(FR-3, US-5, AC-8)_
- [ ] Each account has exactly one Inbox. If multiple folders claim to be INBOX, only one is assigned the role. _(FR-2)_
- [ ] The Inbox role assignment is persisted to the data model.

## Sizing
Small — one name check plus a fallback synthesis path.

## HITL / AFK
AFK — deterministic logic with clear rules.

## Notes
- IMAP RFC 3501 mandates that "INBOX" is a case-insensitive reserved name, so in practice nearly all servers will have it. The synthesis path is a safety net for unusual edge cases.
- The epic (N-6) explicitly states Inbox is always guaranteed. The synthesis approach (create a virtual folder entry vs. error out) is an implementation decision not prescribed by the epic.
