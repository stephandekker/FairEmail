#!/bin/bash
# ralph.sh — Continuously picks the top-priority unblocked user story and implements it
#             using a single Claude Code agent.
#
# Usage: bash ralph.sh
#
# User stories live in docs/epics/user-stories and are grouped by EPIC number and title
# A story is "unblocked" when every US-NN listed in its "## Dependencies" section has
# already been moved to docs/epics/user-stories/done/.
# Stories without "## Acceptance criteria" (e.g. PRDs) are skipped.
# Lowest US-NN among unblocked stories is picked first.
# ONE story is fully completed (including a git commit) before the next is started.
# When a story is finished, its .md file is moved to docs/epics/user-stories/done/ under the correct epic subdirectory.
#
# Max iterations: 2x the number of open stories at startup. This prevents agents that
# create new stories mid-loop (e.g. the UX reviewer) from running ralph indefinitely.

set -euo pipefail

STORIES_DIR="docs/epics/user-stories"
DONE_DIR="${STORIES_DIR}/done"

log() { echo "[$(date '+%Y-%m-%d %H:%M:%S')] $*"; }

# List open story files (top-level *.md only — excludes bugs/, tasks/, done/).
list_open_stories() {
  find "$STORIES_DIR" -maxdepth 1 -type f -name '*.md' | sort
}

# Extract US-NN id from a path like "docs/.../US-01_Project foundation.md".
story_id_from_path() {
  basename "$1" | grep -oE '^US-[0-9]+'
}

# Extract human title from a filename like "US-01_Project foundation.md" → "Project foundation".
story_title_from_path() {
  basename "$1" .md | sed -E 's/^US-[0-9]+_//'
}

pick_unblocked_story() {
  local open_files
  open_files=$(list_open_stories)

  if [ -z "$open_files" ]; then
    echo "NO_STORIES"
    return
  fi

  # Build a space-padded list of currently-open story ids for blocker lookup.
  local open_ids=" "
  while IFS= read -r f; do
    [ -z "$f" ] && continue
    local id
    id=$(story_id_from_path "$f" || true)
    [ -n "$id" ] && open_ids+="$id "
  done <<< "$open_files"

  while IFS= read -r f; do
    [ -z "$f" ] && continue

    # Skip files without acceptance criteria (e.g. PRDs).
    if ! grep -q "^## Acceptance criteria" "$f"; then
      continue
    fi

    # Pull just the "## Dependencies" section — ignore US-NN mentions elsewhere
    # in the file (Story narrative, Out of scope, etc.).
    local deps_section
    deps_section=$(awk '/^## Dependencies/{flag=1; next} /^## /{flag=0} flag' "$f")

    local blockers
    blockers=$(echo "$deps_section" | grep -oE 'US-[0-9]+' | sort -u || true)

    local all_clear=true
    for b in $blockers; do
      if [[ "$open_ids" == *" $b "* ]]; then
        all_clear=false
        break
      fi
    done

    if [ "$all_clear" = "true" ]; then
      echo "$f"
      return
    fi
  done <<< "$open_files"

  echo "NO_UNBLOCKED"
}

log "======================================"
log "  ralph.sh — autonomous story loop"
log "======================================"

if ! git diff --quiet || ! git diff --cached --quiet; then
  log "ERROR: Uncommitted local changes detected. Please commit or stash them before running ralph."
  exit 1
fi

# Install the pre-commit hook if a source is present and out of date.
HOOK_TARGET=".git/hooks/pre-commit"
HOOK_SOURCE="hooks/pre-commit"
if [ -f "$HOOK_SOURCE" ]; then
  if [ ! -f "$HOOK_TARGET" ] || ! diff -q "$HOOK_SOURCE" "$HOOK_TARGET" > /dev/null 2>&1; then
    cp "$HOOK_SOURCE" "$HOOK_TARGET"
    chmod +x "$HOOK_TARGET"
    log "Installed/updated pre-commit hook from ${HOOK_SOURCE}"
  fi
fi

mkdir -p "$DONE_DIR"

# Count stories with acceptance criteria for the iteration cap.
INITIAL_STORY_COUNT=0
while IFS= read -r f; do
  [ -z "$f" ] && continue
  if grep -q "^## Acceptance criteria" "$f"; then
    INITIAL_STORY_COUNT=$(( INITIAL_STORY_COUNT + 1 ))
  fi
done <<< "$(list_open_stories)"

MAX_ITERATIONS=$(( INITIAL_STORY_COUNT * 2 ))
ITERATIONS_DONE=0
log "Open stories at startup: ${INITIAL_STORY_COUNT} — max iterations set to ${MAX_ITERATIONS}"

while true; do
  if [ "$MAX_ITERATIONS" -gt 0 ] && [ "$ITERATIONS_DONE" -ge "$MAX_ITERATIONS" ]; then
    log "Reached max iterations (${MAX_ITERATIONS}). Stopping to prevent runaway loop."
    break
  fi

  log ""
  log "--- Scanning for next unblocked story (iteration $((ITERATIONS_DONE + 1))/${MAX_ITERATIONS}) ---"

  STORY_FILE=$(pick_unblocked_story)

  if [ "$STORY_FILE" = "NO_STORIES" ]; then
    log "All stories are done. Nothing left to do — exiting."
    break
  fi

  if [ "$STORY_FILE" = "NO_UNBLOCKED" ]; then
    log "All remaining stories are blocked by open dependencies. Cannot proceed — exiting."
    break
  fi

  STORY_ID=$(story_id_from_path "$STORY_FILE")
  STORY_TITLE=$(story_title_from_path "$STORY_FILE")
  STORY_BODY=$(cat "$STORY_FILE")

  # Exposed for the pre-commit hook (if installed) so it can reference Ralph context.
  export RALPH_STORY_ID="$STORY_ID"
  export RALPH_STORY_TITLE="$STORY_TITLE"
  export RALPH_STORY_FILE="$STORY_FILE"

  log ""
  log ">>> Implementing ${STORY_ID}: ${STORY_TITLE}"
  log "    Source: ${STORY_FILE}"
  log ""

  PROMPT="$(cat <<PROMPT
Implement user story ${STORY_ID} for this repository.

The full story spec lives at: ${STORY_FILE}

## Story: ${STORY_TITLE}

${STORY_BODY}

## Implementation Instructions
- Read CLAUDE.md first to understand the project structure, tech stack, and coding standards.
- Implement everything required by the acceptance criteria — no more, no less.
- Respect the project layout in CLAUDE.md: keep business logic in \`src/core/\` (UI-free, unit-testable),
  put I/O and persistence in \`src/services/\`, and keep \`src/ui/\` for widgets and templates.
- Do NOT change or remove existing tests. If tests need updating due to new behaviour, add new ones.
- After implementation, run the following and fix any failures before finishing:
  - \`cargo fmt\`
  - \`cargo clippy --all-targets -- -D warnings\`
  - \`cargo test\`

## IMPORTANT: Only one story at a time
Do not start or plan any other stories. Complete this story fully before stopping.

## Commit
Once all acceptance criteria are met and tests pass, create a single git commit
that includes all changed files. Also \`git mv\` the story spec
(\`${STORY_FILE}\`) into \`${DONE_DIR}/\` as part of the same commit so the
ralph loop can see it is finished. The commit message must be exactly:

RALPH: ${STORY_ID} - ${STORY_TITLE}
PROMPT
)"

  if ! claude --dangerously-skip-permissions -p "$PROMPT"; then
    log "claude exited with non-zero status — possibly hit usage limit. Stopping."
    exit 1
  fi

  # Safety net: if Claude forgot to move the spec, do it ourselves so the loop
  # doesn't pick the same story again next iteration.
  if [ -f "$STORY_FILE" ]; then
    log ""
    log "--- Story file still in place; moving ${STORY_FILE} → ${DONE_DIR}/ ---"
    git mv "$STORY_FILE" "$DONE_DIR/"
    git commit -m "RALPH: mark ${STORY_ID} done"
  fi

  ITERATIONS_DONE=$(( ITERATIONS_DONE + 1 ))
  log "<<< Done with ${STORY_ID}. Continuing loop..."
done

log ""
log "======================================"
log "  ralph.sh finished."
log "======================================"
