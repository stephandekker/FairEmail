# Form Neutralization

## Parent Feature
#3.6 Safe HTML View

## Blocked by
2-core-sanitization-pipeline

## Description
Extend the sanitization pipeline to remove or render inert all HTML form elements, so that no data can be submitted from within a rendered email message. `<form>`, `<textarea>`, `<button>`, and `<select>` elements are removed entirely. `<input>` elements are retained for visual display only (showing checked state for checkboxes/radios) but stripped of all submission-related attributes.

## Motivation
Form elements in email are a phishing vector — they can trick users into submitting credentials or data to attacker-controlled servers. Removing actionability while preserving visual state (e.g. checked checkboxes in receipts) balances safety with readability.

## Acceptance Criteria
- [ ] `<form>`, `<textarea>`, `<button>`, and `<select>` elements are completely removed from the output.
- [ ] `<input>` elements of type checkbox/radio are retained with their visual state (`checked` attribute) but are non-interactive.
- [ ] No `name`, `value`, `action`, `method`, or `formaction` attribute survives on any element.
- [ ] Retained input elements cannot submit data (no surrounding form context, no submission attributes).
- [ ] A test message containing a phishing form renders with the form removed and no submission possible.
- [ ] A test message with styled checkboxes (e.g. a survey receipt) shows the check state visually but elements are inert.

## HITL/AFK Classification
AFK — automated tests with form-containing HTML inputs.

## Notes
- FR-9 through FR-11 govern this story.
