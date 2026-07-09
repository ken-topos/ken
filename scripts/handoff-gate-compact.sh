#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'USAGE'
Usage:
  scripts/handoff-gate-compact.sh [--wait-seconds <seconds>] <agent>...

Resets each named agent worktree to origin/main, sends the Codex compaction
sequence to that agent's moot tmux session, then waits five minutes by default.

Agent names may be passed as either "language-leader" or "moot-language-leader".
The script fails before making changes if any named agent worktree or tmux
session cannot be resolved.
USAGE
}

die() {
  printf 'error: %s\n' "$*" >&2
  exit 1
}

need_cmd() {
  command -v "$1" >/dev/null 2>&1 || die "missing required command: $1"
}

wait_seconds=300
agents=()

while [ "$#" -gt 0 ]; do
  case "$1" in
    --wait-seconds)
      [ "$#" -ge 2 ] || die "--wait-seconds requires a value"
      wait_seconds="$2"
      shift 2
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    --)
      shift
      while [ "$#" -gt 0 ]; do
        agents+=("$1")
        shift
      done
      ;;
    -*)
      die "unknown argument: $1"
      ;;
    *)
      agents+=("$1")
      shift
      ;;
  esac
done

[ "${#agents[@]}" -gt 0 ] || die "at least one agent is required"
case "$wait_seconds" in
  ''|*[!0-9]*)
    die "--wait-seconds must be a non-negative integer"
    ;;
esac

need_cmd git
need_cmd tmux

repo_root="$(dirname "$(git rev-parse --git-common-dir)")"
origin_main="origin/main"

normalize_agent() {
  local agent="$1"
  agent="${agent#moot-}"
  printf '%s\n' "$agent"
}

worktree_for_agent() {
  local agent="$1"
  local direct="$repo_root/.worktrees/$agent"

  if [ -d "$direct/.git" ] || [ -f "$direct/.git" ]; then
    printf '%s\n' "$direct"
    return 0
  fi

  git worktree list --porcelain |
    awk -v agent="$agent" '
      /^worktree / {
        path = substr($0, 10)
        name = path
        sub(".*/", "", name)
        if (name == agent) {
          print path
          found = 1
          exit
        }
      }
      END { exit found ? 0 : 1 }
    '
}

reset_agent() {
  local agent="$1"
  local worktree="$2"
  local cur_branch ahead preserve_ref

  # SAFETY: `reset --hard origin/main` moves the current branch ref, which
  # orphans any commits it holds ahead of origin/main (completed-but-unmerged
  # work, not just uncommitted state). Preserve them under a `preserved/` ref
  # first so a handoff compaction can never silently destroy landed work.
  cur_branch="$(git -C "$worktree" rev-parse --abbrev-ref HEAD 2>/dev/null || printf 'HEAD')"
  ahead="$(git -C "$worktree" rev-list --count "$origin_main"..HEAD 2>/dev/null || printf '0')"
  if [ "${ahead:-0}" -gt 0 ]; then
    preserve_ref="preserved/${cur_branch//\//-}-$(git -C "$worktree" rev-parse --short HEAD)"
    git -C "$worktree" branch -f "$preserve_ref" HEAD >/dev/null 2>&1 || true
    printf '[%s] WARNING: %s had %s commit(s) ahead of %s — preserved at %s before reset\n' \
      "$agent" "$cur_branch" "$ahead" "$origin_main" "$preserve_ref"
  fi

  printf '[%s] resetting %s to %s\n' "$agent" "$worktree" "$origin_main"
  git -C "$worktree" reset --hard "$origin_main" >/dev/null
}

compact_agent() {
  local agent="$1"
  local target="moot-$agent"

  printf '[%s] sending compaction sequence to %s\n' "$agent" "$target"
  tmux send-keys -t "$target" Enter
  sleep 1
  tmux send-keys -t "$target" -l '/compact'
  sleep 1
  tmux send-keys -t "$target" Enter
}

# Capture the post-compaction evidence for one agent: the live context marker
# from its pane plus its worktree HEAD/branch. Grep never fails the script
# (guarded with || true); a missing marker is reported, not fatal.
verify_agent() {
  local agent="$1"
  local worktree="$2"
  local target="moot-$agent"
  local ctx head branch

  # Bottom-most match is the live status line. Handles both Claude Code
  # ("ctx 0%") and Codex ("0% context left"), plus an in-progress "Compacting".
  ctx="$(tmux capture-pane -t "$target" -p 2>/dev/null \
    | grep -oE 'ctx [0-9]+%|[0-9]+% context left|Compacting|Context compacted' \
    | tail -1 || true)"
  [ -n "$ctx" ] || ctx='(no ctx marker in visible pane — capture wide by hand)'

  head="$(git -C "$worktree" rev-parse --short HEAD 2>/dev/null || printf '?')"
  branch="$(git -C "$worktree" rev-parse --abbrev-ref HEAD 2>/dev/null || printf '?')"

  printf '[%s] %s | worktree %s (%s)\n' "$agent" "$ctx" "$head" "$branch"
}

wait_for_jobs() {
  local stage="$1"
  shift

  local failed=0
  local pid
  for pid in "$@"; do
    if ! wait "$pid"; then
      failed=1
    fi
  done

  [ "$failed" -eq 0 ] || die "$stage failed"
}

declare -A seen_agents=()
resolved_agents=()
resolved_worktrees=()

for raw_agent in "${agents[@]}"; do
  agent="$(normalize_agent "$raw_agent")"
  [ -n "$agent" ] || die "empty agent name"

  if [ -n "${seen_agents[$agent]:-}" ]; then
    continue
  fi
  seen_agents[$agent]=1

  worktree="$(worktree_for_agent "$agent")" ||
    die "no worktree found for agent: $agent"

  tmux has-session -t "moot-$agent" >/dev/null 2>&1 ||
    die "no tmux session found for agent: moot-$agent"

  resolved_agents+=("$agent")
  resolved_worktrees+=("$worktree")
done

git fetch origin >/dev/null
git rev-parse --verify --quiet "$origin_main^{commit}" >/dev/null ||
  die "could not resolve $origin_main"

printf 'Compacting %s agent(s): %s\n' \
  "${#resolved_agents[@]}" "${resolved_agents[*]}"

reset_pids=()
for i in "${!resolved_agents[@]}"; do
  reset_agent "${resolved_agents[$i]}" "${resolved_worktrees[$i]}" &
  reset_pids+=("$!")
done
wait_for_jobs "worktree reset" "${reset_pids[@]}"

compact_pids=()
for agent in "${resolved_agents[@]}"; do
  compact_agent "$agent" &
  compact_pids+=("$!")
done
wait_for_jobs "compaction send" "${compact_pids[@]}"

printf 'Compaction commands sent. Waiting %ss before returning.\n' \
  "$wait_seconds"
sleep "$wait_seconds"

printf 'Post-compaction verification (evidence):\n'
for i in "${!resolved_agents[@]}"; do
  verify_agent "${resolved_agents[$i]}" "${resolved_worktrees[$i]}"
done
printf 'Handoff-gate compaction wait complete.\n'
