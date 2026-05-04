# Break epic {{EPIC_ID}} — {{EPIC_NAME}} into user stories

You are running inside a non-interactive automation loop. Your only task this run is to break one specific epic into user-story files inside one specific output directory, then exit.

## Epic

- **ID**: {{EPIC_ID}}
- **Name**: {{EPIC_NAME}}
- **MoSCoW priority for v1 (Linux desktop)**: `[{{EPIC_PRIORITY}}]`
- **Source epic file**: `{{EPIC_PATH}}`

Read the epic file in full before drafting stories. The epic is the authoritative description of *what* must be built; do not contradict it.

## Codebase

`{{REPO_ROOT}}/` — FairEmail Android source tree, being reframed as a Linux desktop email application. The code is reference context for grounding story granularity (e.g. recognising what is one cohesive change vs. several). It is *not* the source of truth for behaviour — the epic is. Where the code and the epic disagree, the epic wins, and the disagreement should be flagged inside the story body.

## Method

Invoke the `epic-to-user-stories` skill against the epic identified above.

**Override Step 4 (Quiz the user) of the skill.** This run is non-interactive — there is no user to quiz. Instead:

- Use your best judgement on granularity, dependency ordering, and HITL/AFK classification.
- If a decision is genuinely uncertain (e.g. whether a slice should be split further, or whether a step needs human review), record the uncertainty inside the relevant story under a `## Notes` section. Do **not** ask the user.
- Prefer **many thin tracer-bullet slices over few thick ones**, as the skill prescribes. Each slice must cut end-to-end through every layer touched by that behaviour, not be a horizontal slice of one layer.

Apply the skill's `<user-story-template>` for the body of each story. For the `## Parent Feature` section, use:

```
#{{EPIC_ID}} {{EPIC_NAME}}
```

For the `## Blocked by` section, reference other stories in this same epic by their filename (without the `.md` extension) — e.g. `Blocked by 1-add-account-record`. There are no GitHub issue numbers in this loop.

## Output

Write each user story as a separate `.md` file inside exactly:

  {{OUTPUT_DIR}}/

Use the filename convention `<N>-<short-slug>.md`, where `<N>` is a 1-based sequence number reflecting the recommended build order (blockers first), and `<short-slug>` is a short kebab-case description of the slice. Examples:

- `1-create-account-record.md`
- `2-test-connection-before-save.md`
- `3-edit-existing-account.md`

Do not write to any other path. Do not modify the parent epic file (`{{EPIC_PATH}}`). Do not modify code, configuration, build files, the high-level feature list, or anything outside `{{OUTPUT_DIR}}/`. If you discover a bug or inconsistency in the epic, flag it inside the relevant story under `## Notes` — do not edit the epic.

The output directory has already been created for you. You only need to write story files into it.

## Hard constraints

- **NO interactive prompting.** Do not ask the user any clarifying questions. Make the best inference and document genuine ambiguities in a per-story `## Notes` section.
- **NO code changes**, anywhere in the repository.
- **NO modification of the parent epic** at `{{EPIC_PATH}}`.
- **NO scope creep**: this run produces user-story files for exactly one epic, the one named above. Do not start on a sibling epic, do not edit other epics' stories.
- **NO implementation details** in story bodies that the parent epic deliberately omits — keep stories at the same behaviour/contract level the epic uses (no class names, no SQL, no specific framework APIs).
- **At least one story file must be produced.** If the epic genuinely cannot be split into more than one slice, write one story that covers the whole epic — but in nearly all cases there will be several.

When all story files for this epic have been written into `{{OUTPUT_DIR}}/`, you are done. Exit cleanly.
