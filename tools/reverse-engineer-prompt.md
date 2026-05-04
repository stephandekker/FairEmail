# Reverse-engineer feature {{FEATURE_ID}} — {{FEATURE_NAME}}

You are running inside a non-interactive automation loop. Your only task this run is to produce a single epic document for one specific feature, then exit.

## Feature

- **ID**: {{FEATURE_ID}}
- **Name**: {{FEATURE_NAME}}
- **Description from the high-level feature list** (`{{REPO_ROOT}}/reverse-engineering/high-level-features.md`):

> {{FEATURE_DESCRIPTION}}

## Codebase

`{{REPO_ROOT}}/` — FairEmail Android source tree. The code is the ultimate source of truth. Where online or in-repo documentation conflicts, prefer the most recent. The in-repo `FAQ.md` and `CHANGELOG.md` are useful supplements.

## Method

Invoke the `reverse-engineer-epic` skill. Treat the feature as if it were a Linux desktop email application, consistent with the framing already used in `{{REPO_ROOT}}/reverse-engineering/high-level-features.md` — translate Android-specific concepts to their desktop equivalents and omit those that have no clear desktop counterpart.

The reference example for tone, structure, and depth is:

  {{REPO_ROOT}}/docs/epics/unified-inbox.md

Use the same section structure (Background & Purpose, Goals, Non-Goals, Glossary, Personas, User Stories, Functional Requirements, Non-Functional Requirements, Acceptance Criteria, Open Questions, Design Notes & Rationale).

## Output

Write the epic to exactly:

  {{OUTPUT_PATH}}

Do not write to any other path. Do not modify any other file. Do not modify code, configuration, build files, or anything outside `{{OUTPUT_PATH}}`. If you discover a bug or inconsistency, mention it in the **Open Questions** section of the epic — do not attempt to fix it.

## Hard constraints

- **NO implementation details** in the document: no class names, no database table or column names, no method names, no SQL, no layout filenames, no Java/Kotlin/Android API names, no library names, no specific frameworks. Capture *behavior, intent and contracts* only.
- **NO code changes**, anywhere in the repository.
- **NO interactive prompting**. Do not ask the user any clarifying questions. Make the best inference from the code and document any genuine ambiguities under **Open Questions**.
- **NO scope creep**: this run produces exactly one epic file for the feature named above.

When `{{OUTPUT_PATH}}` has been written, you are done. Exit cleanly.
