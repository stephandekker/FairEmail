#!/usr/bin/env bash
# =============================================================================
# Ralph loop: iterate the epics in docs/epics/ and break each into a set of
# tracer-bullet user-story files under docs/user-stories/<id>-<slug>/.
#
# Each iteration:
#   1. Find the next epic in docs/epics/ that does NOT already have a populated
#      output directory at docs/user-stories/<id>-<slug>/ (containing >= 1 .md).
#   2. Invoke `claude -p` headlessly with a constrained prompt that runs the
#      epic-to-user-stories skill against that single epic.
#   3. Move on to the next epic, until --max iterations have been reached
#      or there are no more pending epics.
#
# Priority handling (MoSCoW for v1 of the Linux Desktop application):
#   Pending epics are processed in priority order: all [MUST] items first,
#   then all [SHOULD] items, then all [COULD] items. [WONT] (and untagged
#   epics that resolve to WONT) are skipped entirely. Within each priority
#   bucket, epics are sorted by numeric feature ID (1.1, 1.2, ..., 2.1, 10.1).
#
# Source of priority: each epic file carries a header line of the form
#   > **MoSCoW priority for v1 (Linux desktop):** `[MUST]` — ...
# (added previously by the reverse-engineer loop / a one-shot tagging pass).
# Epics missing that line are treated as priority "" and ranked alongside
# [COULD] entries.
#
# This loop NEVER implements code. The prompt explicitly forbids code changes
# and confines writes to the per-epic output directory.
#
# Usage:
#   tools/epic-to-user-stories-loop.sh --max 5
#   tools/epic-to-user-stories-loop.sh --max 1 --dry-run
#   tools/epic-to-user-stories-loop.sh --max 5 --feature 3.2
#   tools/epic-to-user-stories-loop.sh --max 10 --max-turns 60
#   tools/epic-to-user-stories-loop.sh --list
#
# Flags:
#   --max N         (required) Process at most N epics this run.
#   --dry-run       Show what would be processed; do not invoke claude.
#   --feature X.Y   Restrict to a single epic (still bounded by --max).
#                   A WONT-priority epic is still skipped under --feature.
#   --max-turns N   Cap claude turns per epic. Default: 60.
#   --list          List all epics and their done/pending status, then exit.
#   --help, -h      Show this help.
# =============================================================================

set -euo pipefail

MAX=""
DRY_RUN=0
FEATURE_FILTER=""
MAX_TURNS=60
LIST_ONLY=0

while [[ $# -gt 0 ]]; do
  case "$1" in
    --max)        MAX="$2"; shift 2 ;;
    --dry-run)    DRY_RUN=1; shift ;;
    --feature)    FEATURE_FILTER="$2"; shift 2 ;;
    --max-turns)  MAX_TURNS="$2"; shift 2 ;;
    --list)       LIST_ONLY=1; shift ;;
    --help|-h)    sed -n '2,/^# ===/p' "$0" | sed 's/^# \{0,1\}//'; exit 0 ;;
    *)            echo "Unknown option: $1" >&2; exit 2 ;;
  esac
done

if [[ "$LIST_ONLY" -eq 0 && -z "$MAX" ]]; then
  echo "Error: --max is required (or use --list)" >&2
  exit 2
fi

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
EPICS_DIR="$REPO_ROOT/docs/epics"
STORIES_DIR="$REPO_ROOT/docs/user-stories"
PROMPT_TEMPLATE="$REPO_ROOT/tools/epic-to-user-stories-prompt.md"
LOG_DIR="$STORIES_DIR/.loop-logs"

[[ -d "$EPICS_DIR" ]]       || { echo "Missing: $EPICS_DIR" >&2; exit 1; }
[[ -f "$PROMPT_TEMPLATE" ]] || { echo "Missing: $PROMPT_TEMPLATE" >&2; exit 1; }
mkdir -p "$STORIES_DIR" "$LOG_DIR"

# ---------------------------------------------------------------------------
# Parse docs/epics/*.md into "ID|NAME|SLUG|PRIORITY|FILENAME" lines.
#
# - ID and SLUG come from the filename, which by convention is "<id>-<slug>.md"
#   (e.g. "1.1-unlimited-accounts.md" → id "1.1", slug "unlimited-accounts").
# - NAME comes from the first "# Epic: ..." heading inside the file; it falls
#   back to the slug (humanised) when no heading is found.
# - PRIORITY comes from the first occurrence of `[MUST|SHOULD|COULD|WONT]`
#   inside the file's header (everything before the first "## " section).
#   Untagged epics yield PRIORITY="".
#
# Output is sorted numerically by feature ID (1.1 < 1.2 < ... < 1.9 < 1.10
# < 2.1 < ... < 10.1), so the listing matches the high-level-features order.
# ---------------------------------------------------------------------------
parse_epics() {
  python3 - "$EPICS_DIR" <<'PY'
import os, re, sys
epics_dir = sys.argv[1]
fname_re  = re.compile(r'^(\d+)\.(\d+)-(.+)\.md$')
title_re  = re.compile(r'^# Epic:\s*(.+?)\s*$', re.MULTILINE)
prio_re   = re.compile(r'`\[(MUST|SHOULD|COULD|WONT)\]`')

rows = []
for fname in os.listdir(epics_dir):
    m = fname_re.match(fname)
    if not m:
        continue
    major, minor, slug = m.group(1), m.group(2), m.group(3)
    fid = f"{major}.{minor}"
    path = os.path.join(epics_dir, fname)
    text = open(path).read()
    # Confine priority search to the file header (before first "## " section)
    header = text.split("\n## ", 1)[0]
    pm = prio_re.search(header)
    priority = pm.group(1) if pm else ""
    tm = title_re.search(text)
    name = tm.group(1).strip() if tm else slug.replace('-', ' ').title()
    name = name.replace('|', '/')
    slug = slug.replace('|', '/')
    rows.append((int(major), int(minor), fid, name, slug, priority, fname))

rows.sort(key=lambda r: (r[0], r[1]))
for _, _, fid, name, slug, priority, fname in rows:
    print(f"{fid}|{name}|{slug}|{priority}|{fname}")
PY
}

ts() { date +'%Y-%m-%d %H:%M:%S'; }

# Format a number of seconds as HHh MMm SSs (or shorter if <1h / <1m).
fmt_duration() {
  local s="$1" h m
  if (( s < 60 )); then printf '%ds' "$s"
  elif (( s < 3600 )); then m=$(( s / 60 )); printf '%dm %02ds' "$m" "$(( s - m*60 ))"
  else h=$(( s / 3600 )); m=$(( (s - h*3600) / 60 )); printf '%dh %02dm %02ds' "$h" "$m" "$(( s - h*3600 - m*60 ))"
  fi
}

# An epic is "done" if its output directory exists and contains at least one
# .md file (excluding hidden files).
already_done() {
  local id="$1" slug="$2"
  local dir="$STORIES_DIR/$id-$slug"
  [[ -d "$dir" ]] || return 1
  compgen -G "$dir/*.md" > /dev/null
}

# Render the per-epic prompt by substituting the template placeholders.
render_prompt() {
  local id="$1" name="$2" priority="$3" epic_path="$4" out_dir="$5"
  python3 - "$PROMPT_TEMPLATE" "$id" "$name" "$priority" "$epic_path" "$out_dir" "$REPO_ROOT" <<'PY'
import sys
tpl, eid, ename, eprio, epath, odir, root = sys.argv[1:]
text = open(tpl).read()
for k, v in (("{{EPIC_ID}}", eid),
             ("{{EPIC_NAME}}", ename),
             ("{{EPIC_PRIORITY}}", eprio or "?"),
             ("{{EPIC_PATH}}", epath),
             ("{{OUTPUT_DIR}}", odir),
             ("{{REPO_ROOT}}", root)):
    text = text.replace(k, v)
sys.stdout.write(text)
PY
}

# ---------------------------------------------------------------------------
# Mode: --list. Print done/pending status and exit.
# Status column shows: [done], [pending], or [skipped] (for [WONT]).
# Priority column shows the MoSCoW tag or [-] for untagged epics.
# ---------------------------------------------------------------------------
if [[ "$LIST_ONLY" -eq 1 ]]; then
  done_count=0
  pending_count=0
  skipped_count=0
  pending_must=0; pending_should=0; pending_could=0
  while IFS='|' read -r id name slug priority fname; do
    if [[ -n "$priority" ]]; then
      prio_label="[$priority]"
    else
      prio_label="[-]"
    fi
    if already_done "$id" "$slug"; then
      printf '  [done]    %-9s %-7s %s\n' "$prio_label" "$id" "$name"
      done_count=$((done_count+1))
    elif [[ "$priority" == "WONT" ]]; then
      printf '  [skipped] %-9s %-7s %s\n' "$prio_label" "$id" "$name"
      skipped_count=$((skipped_count+1))
    else
      printf '  [pending] %-9s %-7s %s\n' "$prio_label" "$id" "$name"
      pending_count=$((pending_count+1))
      case "$priority" in
        MUST)   pending_must=$((pending_must+1)) ;;
        SHOULD) pending_should=$((pending_should+1)) ;;
        *)      pending_could=$((pending_could+1)) ;;
      esac
    fi
  done < <(parse_epics)
  echo ""
  echo "Total: $((done_count+pending_count+skipped_count))   Done: $done_count   Pending: $pending_count   Skipped (WONT): $skipped_count"
  echo "Pending breakdown — MUST: $pending_must   SHOULD: $pending_should   COULD: $pending_could"
  exit 0
fi

# ---------------------------------------------------------------------------
# Pre-scan: collect pending epics, bucket by priority, then concatenate
# in MUST → SHOULD → COULD order. WONT entries are dropped entirely;
# untagged entries fall through to the COULD bucket.
# ---------------------------------------------------------------------------
must_bucket=()
should_bucket=()
could_bucket=()
skipped_wont=0

while IFS='|' read -r id name slug priority fname; do
  if [[ -n "$FEATURE_FILTER" && "$id" != "$FEATURE_FILTER" ]]; then continue; fi
  if already_done "$id" "$slug"; then continue; fi
  if [[ "$priority" == "WONT" ]]; then
    skipped_wont=$((skipped_wont+1))
    continue
  fi
  case "$priority" in
    MUST)   must_bucket+=("$id|$name|$slug|$priority|$fname") ;;
    SHOULD) should_bucket+=("$id|$name|$slug|$priority|$fname") ;;
    *)      could_bucket+=("$id|$name|$slug|${priority:-COULD}|$fname") ;;
  esac
done < <(parse_epics)

pending=()
[[ ${#must_bucket[@]}   -gt 0 ]] && pending+=("${must_bucket[@]}")
[[ ${#should_bucket[@]} -gt 0 ]] && pending+=("${should_bucket[@]}")
[[ ${#could_bucket[@]}  -gt 0 ]] && pending+=("${could_bucket[@]}")

total_pending=${#pending[@]}
run_start_ts=$(date +%s)
echo "Pending epics: $total_pending  (MUST: ${#must_bucket[@]}  SHOULD: ${#should_bucket[@]}  COULD: ${#could_bucket[@]}, WONT skipped: $skipped_wont)"
echo "Will process up to: $MAX  (--max-turns per epic: $MAX_TURNS, dry-run: $DRY_RUN)"
echo "Processing order: all [MUST] first, then [SHOULD], then [COULD]."
echo "Run started     : $(ts)"
echo ""

if [[ $total_pending -eq 0 ]]; then
  echo "Nothing to do."
  exit 0
fi

# ---------------------------------------------------------------------------
# Capture pre-state for safety check.
# ---------------------------------------------------------------------------
pre_status=""
if git -C "$REPO_ROOT" rev-parse --is-inside-work-tree >/dev/null 2>&1; then
  pre_status="$(git -C "$REPO_ROOT" status --porcelain)"
fi

# ---------------------------------------------------------------------------
# Loop.
# ---------------------------------------------------------------------------
processed=0
succeeded=0
failed=0

for entry in "${pending[@]}"; do
  if [[ $processed -ge $MAX ]]; then break; fi
  processed=$((processed+1))

  IFS='|' read -r id name slug priority fname <<< "$entry"
  epic_path="$EPICS_DIR/$fname"
  out_dir="$STORIES_DIR/$id-$slug"
  log="$LOG_DIR/$id-$slug.log"
  prio_label="[${priority:-?}]"

  echo "[$processed/$MAX]  $id  $prio_label  —  $name"
  echo "         epic   : $epic_path"
  echo "         output : $out_dir/"
  echo "         log    : $log"
  echo "         started: $(ts)"

  prompt="$(render_prompt "$id" "$name" "$priority" "$epic_path" "$out_dir")"

  if [[ $DRY_RUN -eq 1 ]]; then
    echo "         (dry-run; prompt preview below)"
    printf '%s\n' "$prompt" | sed 's/^/           │ /'
    echo ""
    continue
  fi

  mkdir -p "$out_dir"
  start_ts=$(date +%s)
  if claude -p "$prompt" \
        --max-turns "$MAX_TURNS" \
        --dangerously-skip-permissions \
        > "$log" 2>&1; then
    elapsed=$(( $(date +%s) - start_ts ))
    story_count=$(find "$out_dir" -maxdepth 1 -type f -name '*.md' | wc -l)
    if (( story_count > 0 )); then
      total_bytes=$(find "$out_dir" -maxdepth 1 -type f -name '*.md' -printf '%s\n' | awk '{s+=$1} END {print s+0}')
      echo "         ✓ ok ($story_count stories, $total_bytes bytes, took $(fmt_duration "$elapsed"))   finished: $(ts)"
      succeeded=$((succeeded+1))
    else
      echo "         ✗ claude exited 0 but no story files were created in $out_dir (see log)   finished: $(ts)"
      failed=$((failed+1))
    fi
  else
    elapsed=$(( $(date +%s) - start_ts ))
    echo "         ✗ claude failed after $(fmt_duration "$elapsed") (see log)   finished: $(ts)"
    failed=$((failed+1))
  fi
  echo ""
done

# ---------------------------------------------------------------------------
# Safety check: warn if claude touched anything outside docs/user-stories/.
# ---------------------------------------------------------------------------
if [[ -n "$pre_status" || $DRY_RUN -eq 0 ]]; then
  if git -C "$REPO_ROOT" rev-parse --is-inside-work-tree >/dev/null 2>&1; then
    post_status="$(git -C "$REPO_ROOT" status --porcelain)"
    unexpected="$(diff <(printf '%s\n' "$pre_status") <(printf '%s\n' "$post_status") \
                  | grep -E '^>' \
                  | grep -vE '(^> \?\?|^> .M|^> M.) +(docs/user-stories/)' \
                  || true)"
    if [[ -n "$unexpected" ]]; then
      echo "WARNING: changes detected outside docs/user-stories/:"
      printf '%s\n' "$unexpected" | sed 's/^/  /'
      echo "  Review with: git -C $REPO_ROOT status"
    fi
  fi
fi

# ---------------------------------------------------------------------------
# Summary.
# ---------------------------------------------------------------------------
remaining=$(( total_pending - processed ))
run_end_ts=$(date +%s)
total_elapsed=$(( run_end_ts - run_start_ts ))
echo "==============================================================="
echo "Run complete.  Processed: $processed   Succeeded: $succeeded   Failed: $failed"
echo "Pending after this run: $remaining"
echo "Run started   : $(date -d "@$run_start_ts" +'%Y-%m-%d %H:%M:%S')"
echo "Run finished  : $(ts)"
echo "Total runtime : $(fmt_duration "$total_elapsed")"
if (( succeeded > 0 )); then
  avg=$(( total_elapsed / succeeded ))
  echo "Avg per epic  : $(fmt_duration "$avg")  (succeeded only)"
fi
echo "Re-run with: tools/epic-to-user-stories-loop.sh --max <N>"
