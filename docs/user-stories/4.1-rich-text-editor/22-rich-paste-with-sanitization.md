# Rich Paste with Sanitization

## Parent Feature
#4.1 Rich Text Editor

## User Story
As any user, when I paste content from an external application, I want the editor to preserve the source formatting by default while sanitizing dangerous markup, so that copy-paste from a web page or document works as expected and is safe.

## Blocked by
`1-wysiwyg-editor-surface`

## Acceptance Criteria
- Pasting content from the system clipboard in rich text mode preserves source formatting (bold, lists, links, etc.) by default.
- Source formatting is converted to the editor's supported style set (unsupported markup is best-effort converted, not silently dropped).
- All pasted content is sanitized to remove potentially dangerous markup: scripts, event handlers, iframes, object/embed elements, and external resource references.
- Pasting content containing `<script>` tags does not execute or insert the script; it is silently stripped.
- Pasting into plain-text mode always inserts as plain text regardless of clipboard content type.
- The user always sees what the recipient will get (sanitization at paste time, not send time).

## Mapping to Epic
- US-30, US-33
- FR-49, FR-52, FR-53
- AC-16 (rich paste portion), AC-18
- N-3, NFR-7

## HITL / AFK
AFK — sanitization rules are well-defined in the epic.

## Notes
- N-3 explains that sanitization happens at paste time (not send time) so the user always sees what the recipient will get.
- NFR-7 requires graceful degradation: if pasted markup contains elements the editor cannot represent (e.g. tables, complex CSS), make a best-effort conversion rather than dropping content.
- "Paste as plain text" and "paste as quote" are separate stories (`23-paste-as-plain-text`, `24-paste-as-quote`).
