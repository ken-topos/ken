#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'USAGE'
Usage:
  scripts/scripted-pr-automerge.sh \
    --target <sha-or-branch> \
    --title <pr-title> \
    (--description <text> | --description-file <path>) \
    [--doc-only]

Creates a PR for the target branch/commit and performs the publisher merge
gate.

Behavior:
  * doc-only: merge immediately.
  * non-doc: wait for the latest CI workflow duration + 10%, then poll PR
    checks until they complete; merge after all checks pass.

The script returns after the merge command succeeds.
USAGE
}

die() {
  printf 'error: %s\n' "$*" >&2
  exit 1
}

need_cmd() {
  command -v "$1" >/dev/null 2>&1 || die "missing required command: $1"
}

target=""
title=""
description=""
description_file=""
doc_only=0

while [ "$#" -gt 0 ]; do
  case "$1" in
    --target)
      [ "$#" -ge 2 ] || die "--target requires a value"
      target="$2"
      shift 2
      ;;
    --title)
      [ "$#" -ge 2 ] || die "--title requires a value"
      title="$2"
      shift 2
      ;;
    --description)
      [ "$#" -ge 2 ] || die "--description requires a value"
      description="$2"
      shift 2
      ;;
    --description-file)
      [ "$#" -ge 2 ] || die "--description-file requires a value"
      description_file="$2"
      shift 2
      ;;
    --doc-only)
      doc_only=1
      shift
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      die "unknown argument: $1"
      ;;
  esac
done

[ -n "$target" ] || die "--target is required"
[ -n "$title" ] || die "--title is required"
if [ -n "$description" ] && [ -n "$description_file" ]; then
  die "use either --description or --description-file, not both"
fi
if [ -z "$description" ] && [ -z "$description_file" ]; then
  die "--description or --description-file is required"
fi

need_cmd gh
need_cmd git
need_cmd date
need_cmd jq

if ! gh auth status >/dev/null 2>&1; then
  if [ -x .devcontainer/mint-gh-token.sh ]; then
    export GH_TOKEN="$(.devcontainer/mint-gh-token.sh)"
    gh auth setup-git >/dev/null
  else
    die "gh is not authenticated and .devcontainer/mint-gh-token.sh is absent"
  fi
fi

tmpdir="$(mktemp -d)"
cleanup() {
  rm -rf "$tmpdir"
}
trap cleanup EXIT

body_file="$description_file"
if [ -n "$description" ]; then
  body_file="$tmpdir/pr-body.md"
  printf '%s\n' "$description" >"$body_file"
fi
[ -f "$body_file" ] || die "description file not found: $body_file"

git fetch origin >/dev/null

resolve_branch() {
  local ref="$1"

  if git show-ref --verify --quiet "refs/heads/$ref"; then
    printf '%s\n' "$ref"
    return 0
  fi

  if git show-ref --verify --quiet "refs/remotes/origin/$ref"; then
    git branch --track "$ref" "origin/$ref" >/dev/null 2>&1 || true
    printf '%s\n' "$ref"
    return 0
  fi

  if git rev-parse --verify --quiet "$ref^{commit}" >/dev/null; then
    local sha short local_matches remote_matches
    sha="$(git rev-parse "$ref^{commit}")"
    short="$(git rev-parse --short "$sha")"

    local_matches="$(git for-each-ref refs/heads/wp \
      --format='%(objectname) %(refname:short)' |
      awk -v sha="$sha" '$1 == sha { print $2 }')"
    remote_matches="$(git for-each-ref refs/remotes/origin/wp \
      --format='%(objectname) %(refname:short)' |
      awk -v sha="$sha" '$1 == sha { sub("^origin/", "", $2); print $2 }')"

    if [ "$(printf '%s\n' "$local_matches" | sed '/^$/d' | wc -l | tr -d ' ')" = "1" ]; then
      printf '%s\n' "$local_matches"
      return 0
    fi

    if [ "$(printf '%s\n' "$remote_matches" | sed '/^$/d' | wc -l | tr -d ' ')" = "1" ]; then
      local match="$remote_matches"
      if ! git show-ref --verify --quiet "refs/heads/$match"; then
        git branch --track "$match" "origin/$match" >/dev/null 2>&1 || true
      fi
      printf '%s\n' "$match"
      return 0
    fi

    local synthetic="wp/scripted-merge-$short"
    if git show-ref --verify --quiet "refs/heads/$synthetic"; then
      [ "$(git rev-parse "$synthetic")" = "$sha" ] ||
        die "synthetic branch $synthetic exists at a different commit"
    else
      git branch "$synthetic" "$sha"
    fi
    printf '%s\n' "$synthetic"
    return 0
  fi

  return 1
}

head_branch="$(resolve_branch "$target")" ||
  die "target does not resolve to a local branch, origin branch, or commit: $target"

head_sha="$(git rev-parse "$head_branch")"
git push --force-with-lease -u origin "refs/heads/$head_branch:refs/heads/$head_branch"

existing_pr="$(gh pr list --head "$head_branch" --state open --json number --jq '.[0].number // empty')"
if [ -n "$existing_pr" ]; then
  pr_number="$existing_pr"
else
  pr_url="$(gh pr create --base main --head "$head_branch" --title "$title" --body-file "$body_file")"
  pr_number="$(printf '%s\n' "$pr_url" | sed -n 's#.*/pull/\([0-9][0-9]*\).*#\1#p' | tail -1)"
fi
[ -n "$pr_number" ] || die "could not determine PR number"

printf 'PR #%s created or found for %s @ %s\n' "$pr_number" "$head_branch" "$head_sha"

merge_pr() {
  gh pr merge "$pr_number" \
    --admin \
    --squash \
    --match-head-commit "$head_sha" \
    --subject "$title" \
    --body-file "$body_file" \
    "$@"
}

if [ "$doc_only" -eq 1 ]; then
  # ── THE DOC-ONLY BLIND SPOT ────────────────────────────────────────────────
  # `--doc-only` merges with NO CI. That is the point of the flag, and it is
  # also a hole: a doc-only merge can redden `main`, and this path is
  # structurally incapable of noticing that it did.
  #
  # Measured, 2026-07-22: `a5d3a13b` ("tracker: DOC-W1 closed") touched
  # `docs/program/issues/DOC-W1.md`, which three `library/` chapters cite as a
  # currency source. It merged clean because it never ran the gate it broke.
  # `main` sat red for ~25 minutes and surfaced on the next `crates/` PR, where
  # it read as that PR's own failure — a shell-script change appearing to break
  # a Rust test shard.
  #
  # ★ The coupling is CITATION-DIRECTED, not path-directed. The doc track and
  #   the build track are concurrent on the premise that one touches `library/`
  #   and the other `crates/`. That is true of file paths and false of
  #   evidence: `library/manifest.toml` cites `crates/` and `docs/program/`
  #   files, so either side can invalidate the other's claim without sharing a
  #   single path.
  #
  # So run the one gate a doc-only merge can break, against the CANDIDATE, in a
  # throwaway worktree. ~4s, no cargo, no network. Narrowly scoped on purpose:
  # it asks only "is the doc currency claim still backed?", so a `main` that is
  # red for an unrelated reason does not block a doc-only publish — and the
  # Librarian's re-validation commit is precisely the publish that PASSES it,
  # so the gate unblocks itself rather than deadlocking.
  doc_gate_wt="$(mktemp -d -t ken-docgate-XXXXXX)"
  cleanup_doc_gate() {
    git worktree remove --force "$doc_gate_wt" >/dev/null 2>&1 || true
    rm -rf "$doc_gate_wt" >/dev/null 2>&1 || true
  }
  trap cleanup_doc_gate EXIT
  if ! git worktree add --detach "$doc_gate_wt" "$head_sha" >/dev/null 2>&1; then
    die "doc-only gate: could not create a worktree at $head_sha to check it"
  fi
  if ! ( cd "$doc_gate_wt" && ./scripts/gen-doc-status.sh --check ); then
    cleanup_doc_gate
    trap - EXIT
    die "doc-only gate: the library currency gate FAILS at $head_sha.

Merging this with --doc-only would land it on main WITHOUT CI and leave main
red for the next PR that runs the full suite -- which will look like that PR's
failure, not this one's.

Re-validate the cited sources and bump library/REVISION (the Librarian's
mandate), then publish. Do not bypass: gen-doc-status.sh refuses to auto-bump
because the bump IS the claim that someone re-validated."
  fi
  cleanup_doc_gate
  trap - EXIT
  printf 'Doc-only gate: library currency check passed at %s.\n' "$head_sha"

  merge_pr
  printf 'Doc-only PR #%s merge command succeeded.\n' "$pr_number"
  exit 0
fi

gh pr merge "$pr_number" --disable-auto >/dev/null 2>&1 || true

latest_run_json="$(gh run list --workflow CI --status completed --limit 1 \
  --json createdAt,updatedAt --jq '.[0] // empty')"

wait_seconds=60
if [ -n "$latest_run_json" ]; then
  created_at="$(printf '%s\n' "$latest_run_json" | jq -r '.createdAt // empty')"
  updated_at="$(printf '%s\n' "$latest_run_json" | jq -r '.updatedAt // empty')"
  if [ -n "$created_at" ] && [ -n "$updated_at" ]; then
    created_s="$(date -d "$created_at" +%s)"
    updated_s="$(date -d "$updated_at" +%s)"
    duration=$(( updated_s - created_s ))
    if [ "$duration" -gt 0 ]; then
      wait_seconds=$(( (duration * 110 + 99) / 100 ))
    fi
  fi
fi

printf 'Waiting %ss before polling PR #%s checks.\n' "$wait_seconds" "$pr_number"
sleep "$wait_seconds"

while :; do
  set +e
  checks_json="$(gh pr checks "$pr_number" --json name,bucket,state,link)"
  checks_status=$?
  set -e
  if [ "$checks_status" -ne 0 ] && [ "$checks_status" -ne 8 ]; then
    die "could not read checks for PR #$pr_number"
  fi

  pending_count="$(printf '%s\n' "$checks_json" |
    jq '[.[] | select(.bucket == "pending")] | length')"
  failing="$(printf '%s\n' "$checks_json" |
    jq '[.[] | select(.bucket == "fail" or .bucket == "cancel")]')"
  failing_count="$(printf '%s\n' "$failing" | jq 'length')"

  if [ "$failing_count" -gt 0 ]; then
    printf '%s\n' "$failing" | jq -r '.[] | "- \(.name): \(.bucket) \(.link)"' >&2
    die "PR #$pr_number has failing checks"
  fi

  if [ "$pending_count" -eq 0 ]; then
    merge_pr
    printf 'PR #%s checks passed and merge command succeeded.\n' "$pr_number"
    exit 0
  fi

  printf 'PR #%s checks still pending (%s); polling again in 15s.\n' "$pr_number" "$pending_count"
  sleep 15
done
