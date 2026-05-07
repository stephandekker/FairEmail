# User Story: Username Format Derivation

## Parent Feature
#1.7 Pre-installed Provider Database

## Description
As a **user of a provider that uses a non-standard username format**, I want the application to derive the correct login username automatically from the provider profile and my email address, so that I do not have to guess whether to use my full email address, just the local part, or some other format.

This slice adds username-format handling to the pre-fill path:
- Each provider entry specifies the expected username format: full email address (default), local part only, or a custom template pattern (FR-18).
- When a provider is matched and settings are pre-filled, the application derives the correct username from the user's email address according to the provider's format and pre-fills it (FR-19).

## Acceptance Criteria
- [ ] A provider entry can specify the username format as one of: full email address (default), local part only, or a custom template.
- [ ] When no username format is specified, the full email address is used as the username (default behaviour).
- [ ] When a provider specifies "local part only", the username derived from `alice@example.com` is `alice` (AC-15).
- [ ] When a provider specifies a custom template, the username is derived by applying the template to the user's email address.
- [ ] The derived username is pre-filled into the account configuration alongside server settings, without requiring the user to modify it.

## Blocked by
`2-server-settings-prefill`

## HITL / AFK
**AFK** — Clear transformation rules with well-defined inputs and outputs.
