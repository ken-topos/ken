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
    --delete-branch \
    --match-head-commit "$head_sha" \
    --subject "$title" \
    --body-file "$body_file" \
    "$@"
}

if [ "$doc_only" -eq 1 ]; then
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
