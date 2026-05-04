# User Story: Tier 1 — IMAP SPECIAL-USE Attribute Detection

## Parent Feature
#2.2 Special-Folder Auto-Detection

## Blocked by
- `1-role-taxonomy-and-folder-data-model`

## Description
As any user, when my mail server advertises folder roles through RFC 6154 SPECIAL-USE attributes, I want the application to use those attributes as the primary and most trusted signal for role assignment, so that detection is reliable even if folder names are unusual or ambiguous.

## Motivation
Server-advertised metadata is the most authoritative source of folder roles. Implementing this tier first gives us correct detection for the majority of modern IMAP servers, covering the happy path before we build fallback heuristics.

## Acceptance Criteria
- [ ] When the server returns SPECIAL-USE attributes on a LIST response, the application parses them and maps them to roles per the following table _(FR-4, FR-5)_:
  | Server attribute | Assigned role |
  |---|---|
  | `\All` or `\Archive` | Archive |
  | `\Drafts` | Drafts |
  | `\Trash` | Trash |
  | `\Junk` | Spam |
  | `\Sent` | Sent |
  | `\Important` | System |
  | `\Flagged` | System |
- [ ] Folders whose attributes include `\NoSelect` or `\NonExistent` are excluded from role assignment and are not presented as selectable. _(FR-6)_
- [ ] If the server advertises two folders with the same role attribute, only one is assigned the role; the other remains a User folder. _(FR-2, US-7)_
- [ ] Tier 1 detection runs to completion across all folders before any Tier 2 (name heuristic) processing begins. _(N-1)_
- [ ] Roles assigned by Tier 1 are persisted to the data model from Story 1.

## Sizing
Medium — IMAP LIST response parsing, attribute extraction, role mapping, persistence.

## HITL / AFK
AFK — straightforward protocol-level mapping with well-defined inputs and outputs.

## Notes
- The existing Android code handles this in `EntityFolder.getType()` (lines 532–550) and `Core.java` (lines 2893–2959). The desktop implementation parses the same RFC 6154 attributes but may use a different IMAP library.
- The epic does not specify tie-breaking when two folders share the same SPECIAL-USE attribute from the server. The Android code appears to take the first encountered. This ambiguity is flagged as OQ-4 in the epic.
