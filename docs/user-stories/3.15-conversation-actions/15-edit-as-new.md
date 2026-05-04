# Edit-as-New

## Parent Feature
#3.15 Conversation Actions

## User Story
As a drafter, when I use edit-as-new on an existing message, I want a fresh draft created with the original's recipients, subject, body, and attachments — but with no threading link and a new message identifier — so that I can re-use old messages as templates without creating false conversation links.

## Blocked by
`1-action-menu-infrastructure`

## Acceptance Criteria
- An "Edit as new" action is available in the action menu.
- The action creates a new draft with: original To/CC/BCC recipients, original subject (no prefix), original body as editable content (not quoted), and all original attachments (FR-35, AC-12).
- The new draft has a fresh message identifier and no threading headers (In-Reply-To, References) linking it to the original (FR-36, AC-12).
- The "From" identity is set to the identity that originally sent the message, if it matches a configured identity; otherwise the account's default identity (FR-37).
- No signature is automatically appended (FR-38, AC-18).
- The action works offline for locally-available messages (NFR-4, AC-22).

## Mapping to Epic
- US-22, US-23, US-24
- FR-35, FR-36, FR-37, FR-38
- AC-12, AC-18, AC-22
- Design Note N-7

## HITL / AFK
AFK — behavior is well-specified.

## Notes
- The body is presented as editable content, not quoted text. This is a key distinction from forward, where the body is quoted. The user gets full editing control over the content.
- N-7 explains why no signature is appended: the original body may already contain a signature, and appending another would corrupt the content.
