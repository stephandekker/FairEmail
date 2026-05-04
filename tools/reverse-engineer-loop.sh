#!/usr/bin/env bash
# =============================================================================
# Ralph loop: iterate the high-level feature list and reverse-engineer each
# entry into a solution-agnostic epic document under docs/epics/.
#
# Each iteration:
#   1. Find the next feature in reverse-engineering/high-level-features.md
#      that does NOT already have a corresponding file in docs/epics/.
#   2. Invoke `claude -p` headlessly with a constrained prompt that runs the
#      reverse-engineer-epic skill against that single feature.
#   3. Move on to the next feature, until --max iterations have been reached
#      or there are no more pending features.
#
# Priority handling (MoSCoW for v1 of the Linux Desktop application):
#   Pending features are processed in priority order, all [MUST] items first,
#   then all [SHOULD] items, then all [COULD] items. [WONT] items are skipped
#   entirely — they are out of scope for v1 and no epic is generated.
#   Within each priority bucket, the original document order is preserved.
#
# This loop NEVER implements code. The prompt explicitly forbids code changes
# and confines writes to the per-feature epic file.
#
# Usage:
#   tools/reverse-engineer-loop.sh --max 5
#   tools/reverse-engineer-loop.sh --max 1 --dry-run
#   tools/reverse-engineer-loop.sh --max 5 --feature 3.2
#   tools/reverse-engineer-loop.sh --max 10 --max-turns 50
#
# Flags:
#   --max N         (required) Process at most N features this run.
#   --dry-run       Show what would be processed; do not invoke claude.
#   --feature X.Y   Restrict to a single feature ID (still bounded by --max).
#                   Note: a [WONT] feature is still skipped under --feature.
#   --max-turns N   Cap claude turns per feature. Default: 40.
#   --list          List all features and their done/pending status, then exit.
#   --help, -h      Show this help.
# =============================================================================

set -euo pipefail

MAX=""
DRY_RUN=0
FEATURE_FILTER=""
MAX_TURNS=40
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
FEATURES_FILE="$REPO_ROOT/reverse-engineering/high-level-features.md"
EPICS_DIR="$REPO_ROOT/docs/epics"
PROMPT_TEMPLATE="$REPO_ROOT/tools/reverse-engineer-prompt.md"
LOG_DIR="$EPICS_DIR/.loop-logs"

[[ -f "$FEATURES_FILE" ]]   || { echo "Missing: $FEATURES_FILE" >&2; exit 1; }
[[ -f "$PROMPT_TEMPLATE" ]] || { echo "Missing: $PROMPT_TEMPLATE" >&2; exit 1; }
mkdir -p "$EPICS_DIR" "$LOG_DIR"

# ---------------------------------------------------------------------------
# Parse the high-level feature list into "ID|NAME|DESCRIPTION|PRIORITY" lines.
#
# Priority is one of MUST / SHOULD / COULD / WONT, extracted from an inline
# tag of the form `[MUST]` (in backticks) that appears at the start of the
# description, optionally separated from the description prose by whitespace.
# A feature that has no priority tag is reported with PRIORITY="" — the loop
# treats untagged entries as the lowest non-skipped tier ([COULD]).
# ---------------------------------------------------------------------------
parse_features() {
  python3 - "$FEATURES_FILE" <<'PY'
import re, sys
text = open(sys.argv[1]).read()
# Tolerate items with or without a trailing description on the same line.
# Use [^\S\n] for "horizontal whitespace" so we never cross a newline.
pat = re.compile(r'^- \*\*(\d+\.\d+)[^\S\n]+(.+?)\*\*[^\S\n]*(.*)$', re.MULTILINE)
prio_pat = re.compile(r'^`\[(MUST|SHOULD|COULD|WONT)\]`\s*(.*)$', re.DOTALL)
for m in pat.finditer(text):
    fid = m.group(1)
    name = m.group(2).strip().rstrip('.').rstrip(':').strip()
    desc = m.group(3).strip()
    priority = ""
    pm = prio_pat.match(desc)
    if pm:
        priority = pm.group(1)
        desc = pm.group(2).strip()
    # Pipe is the field separator; sanitize defensively.
    name = name.replace('|', '/')
    desc = desc.replace('|', '/')
    print(f"{fid}|{name}|{desc}|{priority}")
PY
}

# Numeric rank for a priority tag. Lower rank runs first.
# WONT returns 99 and is filtered out by the caller, never enqueued.
priority_rank() {
  case "$1" in
    MUST)   echo 0 ;;
    SHOULD) echo 1 ;;
    COULD)  echo 2 ;;
    WONT)   echo 99 ;;
    *)      echo 2 ;;  # untagged → treat as [COULD]
  esac
}

slugify() {
  printf '%s' "$1" \
    | tr '[:upper:]' '[:lower:]' \
    | sed -E 's/[^a-z0-9]+/-/g; s/^-+|-+$//g'
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

# An epic is "done" if a file exists under docs/epics/ either as
# "<id>-<slug>.md" or as "<slug>.md" (legacy / unprefixed).
already_done() {
  local id="$1" slug="$2"
  [[ -e "$EPICS_DIR/$id-$slug.md" || -e "$EPICS_DIR/$slug.md" ]]
}

# Render the per-feature prompt by substituting the template placeholders.
render_prompt() {
  local id="$1" name="$2" desc="$3" out="$4"
  python3 - "$PROMPT_TEMPLATE" "$id" "$name" "$desc" "$out" "$REPO_ROOT" <<'PY'
import sys
tpl, fid, fname, fdesc, fout, root = sys.argv[1:]
text = open(tpl).read()
for k, v in (("{{FEATURE_ID}}", fid),
             ("{{FEATURE_NAME}}", fname),
             ("{{FEATURE_DESCRIPTION}}", fdesc),
             ("{{OUTPUT_PATH}}", fout),
             ("{{REPO_ROOT}}", root)):
    text = text.replace(k, v)
sys.stdout.write(text)
PY
}

# ---------------------------------------------------------------------------
# Mode: --list. Print done/pending status and exit.
# Status column shows: [done], [pending], or [skipped] (for [WONT]).
# Priority column shows the MoSCoW tag ([MUST] / [SHOULD] / [COULD] / [WONT])
# or [-] for untagged entries.
# ---------------------------------------------------------------------------
if [[ "$LIST_ONLY" -eq 1 ]]; then
  done_count=0
  pending_count=0
  skipped_count=0
  pending_must=0; pending_should=0; pending_could=0
  while IFS='|' read -r id name desc priority; do
    slug="$(slugify "$name")"
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
  done < <(parse_features)
  echo ""
  echo "Total: $((done_count+pending_count+skipped_count))   Done: $done_count   Pending: $pending_count   Skipped (WONT): $skipped_count"
  echo "Pending breakdown — MUST: $pending_must   SHOULD: $pending_should   COULD: $pending_could"
  exit 0
fi

# ---------------------------------------------------------------------------
# Pre-scan: collect pending features, bucket by priority, then concatenate
# in MUST → SHOULD → COULD order. WONT entries (and any equivalent rank>=99)
# are dropped entirely. Untagged entries fall through to the COULD bucket.
# ---------------------------------------------------------------------------
must_bucket=()
should_bucket=()
could_bucket=()
skipped_wont=0

while IFS='|' read -r id name desc priority; do
  if [[ -n "$FEATURE_FILTER" && "$id" != "$FEATURE_FILTER" ]]; then continue; fi
  slug="$(slugify "$name")"
  if already_done "$id" "$slug"; then continue; fi
  if [[ "$priority" == "WONT" ]]; then
    skipped_wont=$((skipped_wont+1))
    continue
  fi
  case "$priority" in
    MUST)   must_bucket+=("$id|$name|$desc|$priority") ;;
    SHOULD) should_bucket+=("$id|$name|$desc|$priority") ;;
    *)      could_bucket+=("$id|$name|$desc|${priority:-COULD}") ;;
  esac
done < <(parse_features)

pending=()
[[ ${#must_bucket[@]}   -gt 0 ]] && pending+=("${must_bucket[@]}")
[[ ${#should_bucket[@]} -gt 0 ]] && pending+=("${should_bucket[@]}")
[[ ${#could_bucket[@]}  -gt 0 ]] && pending+=("${could_bucket[@]}")

total_pending=${#pending[@]}
run_start_ts=$(date +%s)
echo "Pending features: $total_pending  (MUST: ${#must_bucket[@]}  SHOULD: ${#should_bucket[@]}  COULD: ${#could_bucket[@]}, WONT skipped: $skipped_wont)"
echo "Will process up to: $MAX  (--max-turns per feature: $MAX_TURNS, dry-run: $DRY_RUN)"
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

  IFS='|' read -r id name desc priority <<< "$entry"
  slug="$(slugify "$name")"
  out="$EPICS_DIR/$id-$slug.md"
  log="$LOG_DIR/$id-$slug.log"
  prio_label="[${priority:-?}]"

  echo "[$processed/$MAX]  $id  $prio_label  —  $name"
  echo "         output : $out"
  echo "         log    : $log"
  echo "         started: $(ts)"

  prompt="$(render_prompt "$id" "$name" "$desc" "$out")"

  if [[ $DRY_RUN -eq 1 ]]; then
    echo "         (dry-run; prompt preview below)"
    printf '%s\n' "$prompt" | sed 's/^/           │ /'
    echo ""
    continue
  fi

  start_ts=$(date +%s)
  if claude -p "$prompt" \
        --max-turns "$MAX_TURNS" \
        --dangerously-skip-permissions \
        > "$log" 2>&1; then
    elapsed=$(( $(date +%s) - start_ts ))
    if [[ -e "$out" ]]; then
      echo "         ✓ ok ($(stat -c%s "$out") bytes, took $(fmt_duration "$elapsed"))   finished: $(ts)"
      succeeded=$((succeeded+1))
    else
      echo "         ✗ claude exited 0 but $out was not created (see log)   finished: $(ts)"
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
# Safety check: warn if claude touched anything outside docs/epics/.
# ---------------------------------------------------------------------------
if [[ -n "$pre_status" || $DRY_RUN -eq 0 ]]; then
  if git -C "$REPO_ROOT" rev-parse --is-inside-work-tree >/dev/null 2>&1; then
    post_status="$(git -C "$REPO_ROOT" status --porcelain)"
    unexpected="$(diff <(printf '%s\n' "$pre_status") <(printf '%s\n' "$post_status") \
                  | grep -E '^>' \
                  | grep -vE '(^> \?\?|^> .M|^> M.) +(docs/epics/|reverse-engineering/)' \
                  || true)"
    if [[ -n "$unexpected" ]]; then
      echo "WARNING: changes detected outside docs/epics/ and reverse-engineering/:"
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
echo "Re-run with: tools/reverse-engineer-loop.sh --max <N>"
