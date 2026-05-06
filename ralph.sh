#!/bin/bash
# ralph.sh — Continuously picks the top-priority unblocked user story and implements it
#             using a single Claude Code agent.
#
# Usage: bash ralph.sh [--ignore-local-changes]
#
#   --ignore-local-changes  Skip the uncommitted-changes guard and start anyway.
#
# User stories live in docs/user-stories/<epic-slug>/<N>-<title>.md
#   e.g. docs/user-stories/10.2-no-third-party-servers/2-remote-content-blocking.md
# Each epic-slug directory is named "<major>.<minor>-<short-title>" and contains the
# stories belonging to that epic, named "<N>-<title>.md" where N is the per-epic story
# number (no zero padding).
#
# A story is "unblocked" when every story listed in its "## Blocked by" section has
# already been moved to docs/user-stories/done/<epic-slug>/. Blockers are referenced as
# backtick-wrapped filename stems (e.g. `1-default-network-posture`) and resolved within
# the same epic directory.
# Stories without "## Acceptance Criteria" (e.g. PRDs) are skipped.
# Stories are picked in epic-then-story numeric order: 1.1 before 1.10 before 2.1, and
# within each epic 1- before 2- before 10-.
# ONE story is fully completed (including a git commit) before the next is started.
# When a story is finished, its .md file is moved to
# docs/user-stories/done/<epic-slug>/<N>-<title>.md, mirroring the epic layout.
#
# Max iterations: 2x the number of open stories at startup, to prevent runaway loops if
# the agent creates new stories mid-run.

set -euo pipefail

IGNORE_LOCAL_CHANGES=false
for arg in "$@"; do
  case "$arg" in
    --ignore-local-changes)
      IGNORE_LOCAL_CHANGES=true
      ;;
    -h|--help)
      sed -n '2,26p' "$0"
      exit 0
      ;;
    *)
      echo "Unknown argument: $arg" >&2
      echo "Usage: bash ralph.sh [--ignore-local-changes]" >&2
      exit 2
      ;;
  esac
done

STORIES_DIR="docs/user-stories"
DONE_DIR="${STORIES_DIR}/done"

# Resolve script location and chdir into it so the script works from any cwd
# (cron defaults to $HOME, which breaks every relative path and git command below).
RALPH_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$RALPH_DIR"

# Cron strips PATH down to /usr/bin:/bin. Prepend the dirs we actually need
# (claude is in ~/.local/bin; cargo/rustc are in ~/.cargo/bin) so claude and
# the toolchain commands the prompt invokes are findable.
export PATH="${HOME}/.local/bin:${HOME}/.cargo/bin:${PATH}"

# Fail fast if claude isn't reachable — otherwise the loop exits with bash
# status 127 ("command not found") much later, looking like an unexplained stop.
if ! command -v claude >/dev/null 2>&1; then
  echo "ralph.sh: 'claude' not found on PATH" >&2
  echo "  HOME=${HOME:-<unset>}" >&2
  echo "  PATH=${PATH}" >&2
  exit 1
fi

LOCK_FILE="${RALPH_DIR}/.ralph.lock"
TIMEOUT_FILE="${RALPH_DIR}/ralph-timeout.md"

log() { echo "[$(date '+%Y-%m-%d %H:%M:%S')] $*"; }

write_timeout_marker() {
  local reset_epoch="$1"
  local note="${2:-}"
  {
    echo "# Ralph Usage Limit Timeout"
    echo ""
    echo "Limit hit at:  $(date '+%Y-%m-%d %H:%M:%S %z')"
    echo "Reset at:      $(date -d "@${reset_epoch}" '+%Y-%m-%d %H:%M:%S %z')"
    [ -n "$note" ] && echo "Note:          ${note}"
    echo ""
    echo "reset_epoch=${reset_epoch}"
  } > "$TIMEOUT_FILE"
}

# List open story files under each epic dir, sorted by epic.major, epic.minor, story-num.
# Excludes anything under DONE_DIR.
list_open_stories() {
  find "$STORIES_DIR" -mindepth 2 -maxdepth 2 -type f -name '*.md' \
    -not -path "$DONE_DIR/*" \
  | awk -F/ '{
      epic = $(NF-1); file = $NF
      if (match(epic, /^([0-9]+)\.([0-9]+)/)) {
        split(substr(epic, RSTART, RLENGTH), e, ".")
        major = e[1] + 0; minor = e[2] + 0
      } else { major = 0; minor = 0 }
      story = (match(file, /^[0-9]+/)) ? substr(file, RSTART, RLENGTH) + 0 : 0
      printf "%05d.%05d.%05d\t%s\n", major, minor, story, $0
    }' \
  | sort \
  | cut -f2-
}

# Story id used in logs and commit messages: "<epic-slug>/<story-stem>".
story_id_from_path() {
  local epic stem
  epic=$(basename "$(dirname "$1")")
  stem=$(basename "$1" .md)
  echo "${epic}/${stem}"
}

# Human title: prefer the first H1 heading in the file; fall back to the filename.
story_title_from_path() {
  local h1
  h1=$(grep -m1 -E '^# [^#]' "$1" 2>/dev/null | sed -E 's/^# +//')
  if [ -n "$h1" ]; then
    echo "$h1"
  else
    basename "$1" .md | sed -E 's/^[0-9]+-//; s/-/ /g'
  fi
}

pick_unblocked_story() {
  local open_files
  open_files=$(list_open_stories)

  if [ -z "$open_files" ]; then
    echo "NO_STORIES"
    return
  fi

  while IFS= read -r f; do
    [ -z "$f" ] && continue

    # Skip files without acceptance criteria (e.g. PRDs). Casing varies in the corpus.
    if ! grep -qiE '^## Acceptance Criteria' "$f"; then
      continue
    fi

    local epic_dir
    epic_dir=$(dirname "$f")

    # Pull just the "## Blocked by" section — ignore mentions elsewhere in the file.
    local deps_section
    deps_section=$(awk '/^## Blocked by/{flag=1; next} /^## /{flag=0} flag' "$f")

    # Blockers are backtick-wrapped filename stems, e.g. `1-default-network-posture`.
    local blockers
    blockers=$(echo "$deps_section" | grep -oE '`[0-9]+-[A-Za-z0-9_-]+`' | tr -d '`' | sort -u || true)

    local all_clear=true
    for b in $blockers; do
      # Blocker is unresolved if its file is still in the open epic dir
      # (i.e. has not yet been moved into DONE_DIR/<epic-slug>/).
      if [ -f "${epic_dir}/${b}.md" ]; then
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

# Exit cleanly if another ralph instance is already running.
# fd 200 stays open for the lifetime of this shell; the kernel releases the
# lock automatically when the script exits (even on crash).
exec 200>"$LOCK_FILE"
if ! flock -n 200; then
  log "Another ralph instance is already running (lock: ${LOCK_FILE}). Exiting."
  exit 0
fi

# Bail out cleanly on Ctrl+C / SIGTERM. Without this, claude may swallow SIGINT
# and exit 0, causing the safety-net `git mv` below to fire as if the story
# had been completed successfully.
on_interrupt() {
  echo
  log "Interrupt received — exiting without moving the current story."
  exit 130
}
trap on_interrupt INT TERM

# Exit cleanly if a previous run hit the usage limit and the reset hasn't passed yet.
if [ -f "$TIMEOUT_FILE" ]; then
  RESET_EPOCH=$(grep -oE 'reset_epoch=[0-9]+' "$TIMEOUT_FILE" | head -1 | cut -d= -f2 || true)
  NOW_EPOCH=$(date +%s)
  if [ -n "${RESET_EPOCH:-}" ] && [ "$NOW_EPOCH" -lt "$RESET_EPOCH" ]; then
    log "Usage limit reset has not passed yet (resumes at $(date -d "@${RESET_EPOCH}" '+%Y-%m-%d %H:%M:%S')). Exiting."
    exit 0
  fi
  log "Usage limit reset has passed (or marker is malformed); removing ${TIMEOUT_FILE}."
  rm -f "$TIMEOUT_FILE"
fi

if [ "$IGNORE_LOCAL_CHANGES" = "true" ]; then
  log "WARNING: --ignore-local-changes set; skipping clean-tree check."
elif ! git diff --quiet || ! git diff --cached --quiet; then
  log "ERROR: Uncommitted local changes detected. Please commit or stash them before running ralph."
  log "       (Or pass --ignore-local-changes to override.)"
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
  if grep -qiE '^## Acceptance Criteria' "$f"; then
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
  STORY_EPIC_DIR=$(basename "$(dirname "$STORY_FILE")")
  STORY_DONE_DIR="${DONE_DIR}/${STORY_EPIC_DIR}"

  # Exposed for the pre-commit hook (if installed) so it can reference Ralph context.
  export RALPH_STORY_ID="$STORY_ID"
  export RALPH_STORY_TITLE="$STORY_TITLE"
  export RALPH_STORY_FILE="$STORY_FILE"
  export RALPH_STORY_DONE_DIR="$STORY_DONE_DIR"

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
that includes all changed files. Also \`git mv\` the story spec from
\`${STORY_FILE}\` into \`${STORY_DONE_DIR}/\` (creating that directory if it does
not exist) as part of the same commit so the ralph loop can see it is finished.
The commit message must be exactly:

RALPH: ${STORY_ID} - ${STORY_TITLE}
PROMPT
)"

  CLAUDE_LOG=$(mktemp -t ralph-claude.XXXXXX)
  CLAUDE_EXIT=0
  claude --dangerously-skip-permissions --no-session-persistence -p "$PROMPT" 2>&1 \
    | tee "$CLAUDE_LOG" || CLAUDE_EXIT=$?

  if [ "$CLAUDE_EXIT" -ne 0 ]; then
    # Claude Code emits "Claude AI usage limit reached|<unix_epoch>" when the cap is hit.
    PARSED_RESET=$(grep -oE 'limit reached\|[0-9]+' "$CLAUDE_LOG" | grep -oE '[0-9]+$' | head -1 || true)

    if [ -n "${PARSED_RESET:-}" ]; then
      write_timeout_marker "$PARSED_RESET" "parsed from claude output"
      log "Usage limit hit. Wrote ${TIMEOUT_FILE}; will resume after $(date -d "@${PARSED_RESET}" '+%Y-%m-%d %H:%M:%S')."
      rm -f "$CLAUDE_LOG"
      exit 0
    fi

    if grep -qiE 'usage limit|rate.?limit|too many requests' "$CLAUDE_LOG"; then
      FALLBACK_RESET=$(( $(date +%s) + 5 * 3600 ))
      write_timeout_marker "$FALLBACK_RESET" "fallback: limit detected but no reset epoch parsed"
      log "Usage limit hit but reset epoch not parseable. Estimating 5h cooldown."
      rm -f "$CLAUDE_LOG"
      exit 0
    fi

    log "claude exited with status ${CLAUDE_EXIT} (no usage-limit signal in output). Stopping."
    rm -f "$CLAUDE_LOG"
    exit 1
  fi

  rm -f "$CLAUDE_LOG"

  # Safety net: if Claude forgot to move the spec, do it ourselves so the loop
  # doesn't pick the same story again next iteration.
  if [ -f "$STORY_FILE" ]; then
    log ""
    log "--- Story file still in place; moving ${STORY_FILE} → ${STORY_DONE_DIR}/ ---"
    mkdir -p "$STORY_DONE_DIR"
    git mv "$STORY_FILE" "$STORY_DONE_DIR/"
    git commit -m "RALPH: mark ${STORY_ID} done"
  fi

  ITERATIONS_DONE=$(( ITERATIONS_DONE + 1 ))
  log "<<< Done with ${STORY_ID}. Continuing loop..."
done

log ""
log "======================================"
log "  ralph.sh finished."
log "======================================"
