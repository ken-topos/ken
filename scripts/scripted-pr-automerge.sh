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

# ── PART 2 (SRC-ATTEST): FRESH MERGE-RESULT AUTHORIZATION ────────────────────
#
# What this delivers, stated exactly — @architect ruling dec_50fdjy68gm01j.
# Read the boundary before trusting the guarantee.
#
#   1. UNCONDITIONAL. Old CI is never authorization. EVERY publish -- doc-only
#      and normal alike -- reconstructs the candidate on a freshly fetched
#      origin/main and runs origin/main's trusted checker immediately before
#      merge. This is what closes #885's stale-CI-authorization defect.
#
#   2. CONDITIONAL IDENTITY. "The tree we checked is the tree GitHub landed" is
#      true only within ADR-0003's exclusive-publisher model, and only because
#      all sanctioned merge attempts share ONE enforced critical section. That
#      is the lock below. Without it the claim has no support at all.
#
#   3. ⛔ RESIDUAL BOUNDARY -- NOT CLOSED, AND WE DO NOT CLAIM IT IS.
#      `gh pr merge --match-head-commit` pins the PR HEAD. GitHub exposes NO
#      base-SHA compare-and-swap. So an OUT-OF-BAND writer -- anything merging
#      outside this script -- can still move `main` inside the final API
#      round-trip, and the landed tree will not be the checked tree.
#      Step 4 narrows that window from the CI-poll duration (minutes) to one
#      round-trip. It does not eliminate it.
#      ⇒ "Closes #885's stale-CI authorization defect" is TRUE.
#        "Eliminates every final-round-trip race" is FALSE. Do not write it.
#
#   4. DETECTOR, NOT ROLLBACK. After the merge, compare the landed tree OID
#      against the synthetic checked tree OID and re-run the checker on what
#      actually landed. A mismatch or a red result is a loud publisher failure
#      that FREEZES further publication for diagnosis.
#      ⛔ NEVER auto-revert `main`. An automatic revert of a merge someone else
#        may already have built on is worse than the defect it responds to.
#
# ★ Why the dependency is enforced rather than documented: the previous version
#   of this gate was correct *because* the publisher happened to be serialized,
#   and nothing recorded that. That is the F13 finding, and it recurred one
#   layer up when the SRC-ATTEST frame asserted the identity claim outright. A
#   load-bearing precondition that lives only in prose is not a precondition --
#   it is a hope. So: acquire a real lock, or fail closed.

publisher_state_dir() {
  git rev-parse --path-format=absolute --git-common-dir 2>/dev/null ||
    die "publisher gate: cannot resolve the shared Git directory"
}

# ⚠ The lock MUST live in the COMMON git dir, not a per-worktree path. This
#   repository is checked out as ~70 linked worktrees that share one object
#   store; a per-worktree lock file would be a different file for every agent
#   and would therefore never contend -- a lock that always succeeds, which is
#   indistinguishable from no lock at all until the day it matters.
acquire_merge_lock() {
  need_cmd flock
  local common_dir
  common_dir="$(publisher_state_dir)"
  exec 8>"$common_dir/ken-publisher-merge.lock"
  flock -n 8 || die "publisher gate: another merge critical section is active.

Only one publish may hold the fetch -> check -> merge -> verify window at a
time; that mutual exclusion is the ONLY reason the checked tree is the landed
tree. Wait for the other publish to finish and re-run. Do not bypass."
}

freeze_marker_path() {
  printf '%s/ken-publisher-FROZEN\n' "$(publisher_state_dir)"
}

refuse_if_frozen() {
  local marker
  marker="$(freeze_marker_path)"
  if [ -f "$marker" ]; then
    die "publisher gate: PUBLICATION IS FROZEN -- a previous publish failed its
post-merge verification and further publishing is blocked pending diagnosis.

$(cat "$marker")

Diagnose the landed state on origin/main first. Clear the freeze deliberately,
by hand, once you understand what happened:
  rm $marker"
  fi
}

# ⛔ FAILURE TO PERSIST THE FREEZE MUST NOT BE SWALLOWED (@librarian QA).
#    This ended `|| true`. With the marker path unwritable, the function
#    returned 0, no marker existed, and the NEXT publish proceeded normally --
#    while every caller's message said publication was frozen. The artifact
#    promises persistent state; a write that did not happen cannot be reported
#    as success, least of all by the function whose entire job is that write.
#    ⇒ verify the marker EXISTS and is non-empty, and say so loudly if not.
#      There is nothing to fall back to: if the freeze cannot be persisted, the
#      only true statement is that this invocation failed and subsequent
#      publication is NOT frozen. Say that instead of claiming it is.
freeze_publication() {
  local marker
  marker="$(freeze_marker_path)"

  # ⛔ GUARD THE WRITE EXPLICITLY -- @architect. The previous version ran the
  #    redirect as a bare command under the script's `set -e`. A failed redirect
  #    therefore aborted the shell AT THE PRINTF: before this function's own
  #    `-s` check, before its diagnosis, and before the CALLER's die(). The
  #    operator saw an exit code and nothing else, on the one path whose entire
  #    job is to say something.
  #
  #    ⚠ And the probe missed it by `if`-wrapping the call, which puts the body
  #    in a condition context and SUPPRESSES `set -e` inside it -- so the probe
  #    exercised a different execution mode from production and reported the
  #    diagnosis firing when in production it could not. **A probe must call the
  #    function the way the caller does; the calling context is part of the
  #    behaviour under test.**
  #
  #    With the write guarded here, this function behaves identically whether or
  #    not a caller wraps it -- which is the property that makes it testable at
  #    all.
  if ! printf '%s\n' "$1" >"$marker" 2>/dev/null || [ ! -s "$marker" ]; then
    printf '%s\n' "
⛔⛔ PUBLICATION FREEZE COULD NOT BE PERSISTED at: $marker

The condition that triggered the freeze STILL HAPPENED, and the freeze that was
supposed to block the next publish DOES NOT EXIST. Subsequent publishes will
NOT be stopped. Do not read any later 'frozen' message as protection.

  reason: $1

Freeze publication BY HAND before anything else runs." >&2
    return 1
  fi
  return 0
}

gate_wt=""
checked_base=""
checked_tree_oid=""

release_gate_worktree() {
  if [ -n "$gate_wt" ]; then
    git worktree remove --force "$gate_wt" >/dev/null 2>&1 || true
    rm -rf "$gate_wt" >/dev/null 2>&1 || true
    gate_wt=""
  fi
}

# F12: CHAIN the pre-existing EXIT trap, never clobber it. Bash EXIT traps are
# single-slot; an earlier version overwrote `trap cleanup EXIT` and leaked
# $tmpdir on every run.
cleanup_gate() {
  release_gate_worktree
  cleanup
}
trap cleanup_gate EXIT

# Build the exact squash result on current origin/main and run origin/main's
# checker against it. Sets $checked_base and $checked_tree_oid.
build_and_check_merge_result() {
  # F13: guard the fetch. A silently stale origin/main makes the gate evaluate
  # one base while GitHub squashes onto another -- the F10 split, one layer
  # down, with no diff to review.
  git fetch origin main --quiet ||
    die "publisher gate: CANNOT EVALUATE -- could not refresh origin/main.

The gate must compare against the base the merge will actually land on."

  checked_base="$(git rev-parse origin/main)"

  gate_wt="$(mktemp -d -t ken-pubgate-XXXXXX)"
  git worktree add --detach "$gate_wt" "$checked_base" >/dev/null 2>&1 ||
    die "publisher gate: could not create a worktree at origin/main"

  # ⛔ Give the SCRATCH WORKTREE its own identity, once, rather than passing
  #    `-c user.*` to the operations we happen to know need it.
  #
  #    Measured: `git merge --squash` needs a committer identity too, but ONLY
  #    when the merge is not a fast-forward -- i.e. exactly when origin/main has
  #    advanced past the candidate's base, which is the case this gate exists
  #    for. A first fix that covered only `git commit` passed the fast-forward
  #    probe and failed the advanced-base one. Enumerating the calls observed to
  #    fail is how that happened; this sets it for every git operation in the
  #    worktree instead, so the class cannot recur as new operations are added.
  #
  #    This repository configures user.email PER-REPO, not globally, so a gate
  #    that inherits ambient identity is silently environment-sensitive.
  git -C "$gate_wt" config user.email 'publisher-gate@ken-topos.local' >/dev/null 2>&1 || true
  git -C "$gate_wt" config user.name  'ken publisher gate'             >/dev/null 2>&1 || true

  # ⛔ `git merge --squash` STAGES without COMMITTING, so HEAD would still be
  #    origin/main and a checker that compares a recorded revision against HEAD
  #    -- a commit -- would not see the candidate's content at all. Caught once
  #    by this gate's own three-outcome falsification: the red probe returned
  #    PERMIT. Without the commit the whole mechanism silently degrades into
  #    "is origin/main currently green?", which is NOT what it claims.
  #    `--no-verify` because repo hooks regenerate tracked files, which would
  #    contaminate the very tree under test.
  # ⛔ SEPARATE the two failures. An earlier version ran merge and commit in one
  #    `&&` chain under a single diagnosis naming only the merge. A commit that
  #    failed for its OWN reasons -- no configured author identity being the
  #    live one, since this repo sets user.email per-repo and not globally --
  #    then reported "the candidate needs rebasing onto current origin/main",
  #    which is FALSE and sends the ring to rebase a branch that is fine.
  #    Measured, not imagined: it is how the row 9-11 probe harness first failed,
  #    and the misdiagnosis was convincing enough to survive two readings.
  ( cd "$gate_wt" && git merge --squash "$head_sha" >/dev/null 2>&1 ) ||
    die "publisher gate: CANNOT EVALUATE -- $head_sha does not merge cleanly onto origin/main.

This is NOT a currency-gate failure and re-running any generator will not help.
The candidate needs rebasing onto current origin/main."

  ( cd "$gate_wt" &&
      git commit --no-verify -q -m "publisher gate: merge-result probe" >/dev/null 2>&1 ) ||
    die "publisher gate: CANNOT EVALUATE -- the merge-result probe COMMIT failed.

$head_sha merges onto origin/main cleanly; the failure is in committing the
staged result inside the scratch worktree. This is an environment fault in the
publisher, NOT a defect in the candidate -- do NOT rebase it. Check that the
scratch worktree is writable and that git can create a commit there."

  # Capture the tree BEFORE the F11 overwrite below, so $checked_tree_oid is the
  # true merge result and is comparable with what GitHub lands.
  checked_tree_oid="$(git -C "$gate_wt" rev-parse 'HEAD^{tree}')"

  # F11: the candidate does not get to supply the checker that clears it.
  # Otherwise a PR editing the checker can be published, skip CI, and be gated
  # by the very code it introduces. The cost -- the candidate's own gate changes
  # go untested here -- is the correct trade; CI covers those.
  git show "$checked_base:scripts/gen-doc-status.sh" >"$gate_wt/scripts/gen-doc-status.sh" ||
    die "publisher gate: could not read scripts/gen-doc-status.sh from origin/main"
  chmod +x "$gate_wt/scripts/gen-doc-status.sh"

  ( cd "$gate_wt" && ./scripts/gen-doc-status.sh --check ) ||
    die "publisher gate: the library currency gate FAILS on the MERGE RESULT of
$head_sha onto origin/main.

Merging would leave main red for the next PR that runs the full suite -- which
will look like that PR's failure, not this one's.

Re-validate the cited sources and refresh the attestation ledger (the
Librarian's mandate), then publish. Do not bypass: the check path is read-only
and refuses to repair a mismatch, because the attestation IS the claim that
someone re-validated."
}

# Step 4: re-read origin/main after the check. If the base moved while we were
# evaluating, the result we just cleared is not the result that would land, so
# reconstruct against the new base.
fresh_result_gate() {
  local attempt=0
  while :; do
    build_and_check_merge_result

    git fetch origin main --quiet ||
      die "publisher gate: CANNOT EVALUATE -- could not re-read origin/main before merging."

    if [ "$(git rev-parse origin/main)" = "$checked_base" ]; then
      printf 'Publisher gate: currency check passed on the merge result of %s onto %s.\n' \
        "$head_sha" "$(git rev-parse --short "$checked_base")"
      return 0
    fi

    attempt=$((attempt + 1))
    if [ "$attempt" -ge 3 ]; then
      die "publisher gate: origin/main advanced during 3 consecutive evaluations.

The base is moving faster than the gate can evaluate it. Something else is
publishing concurrently -- which also means the lock above is not covering it.
Investigate before retrying."
    fi
    printf 'Publisher gate: origin/main advanced during evaluation; reconstructing (attempt %s).\n' \
      "$attempt"
    release_gate_worktree
  done
}

# Clause 4: the detector. Runs AFTER the merge, still holding the lock.
# ⛔ TERMINAL REPORTING MUST BE CONDITIONAL ON PERSISTENCE -- @architect.
#    Every caller used to run `freeze_publication ... || true` and then die()
#    with text asserting "Publication is now FROZEN ... clear the freeze by
#    hand." When the marker could not be written, the SAME output said both
#    "Subsequent publishes will NOT be stopped" AND "Publication is now FROZEN".
#    The second is false, and it is the one an operator acts on.
#
#    ⚠ Probe 12c could not see this because it substituted `echo
#    REACHED_DIE_POINT` for the real caller's terminal diagnosis -- a probe that
#    replaces the thing under test cannot observe the thing under test. Fourth
#    instance today of a probe passing by supplying the condition it was meant
#    to detect.
#
#    ⇒ ONE exit point for every freeze alarm, and the protection claim is made
#      only when the protection actually exists.
freeze_and_alarm() {
  local marker_reason="$1" alarm_body="$2"
  if freeze_publication "$marker_reason"; then
    die "$alarm_body

Publication is now FROZEN. Diagnose before publishing again, then clear it
deliberately, by hand."
  fi
  die "$alarm_body

⛔ AND THE FREEZE DID NOT PERSIST. Publication is **NOT** frozen and the next
publish will proceed unblocked -- see the diagnosis above. There is nothing
protecting main right now. Freeze by hand before anything else runs."
}

verify_landed_tree() {
  git fetch origin main --quiet || {
    freeze_and_alarm \
      "Could not fetch origin/main after merging PR #$pr_number ($head_sha). Landed state UNVERIFIED." \
      "publisher gate: merged PR #$pr_number but could NOT verify the landed tree."
  }

  local landed_tree
  landed_tree="$(git rev-parse 'origin/main^{tree}')"

  if [ "$landed_tree" != "$checked_tree_oid" ]; then
    freeze_and_alarm \
      "PR #$pr_number ($head_sha) landed tree $landed_tree but the checked tree was $checked_tree_oid. origin/main moved inside the final round-trip -- an out-of-band writer, or a second publish path outside the lock." \
      "PUBLISHER ALARM: PR #$pr_number merged, but the LANDED TREE IS NOT THE
CHECKED TREE.

  checked: $checked_tree_oid
  landed:  $landed_tree

This is the residual boundary in clause 3 -- something moved origin/main inside
the final API round-trip. The merge HAS happened and is NOT being reverted:
an automatic revert of a commit others may already have built on is worse than
the defect."
  fi

  # Redundant BY CONSTRUCTION when the OIDs match -- identical tree, identical
  # checker, identical result. It is here deliberately anyway: it is the check
  # on the OID comparison ITSELF. If the comparison logic above is ever wrong,
  # this is what still catches a red main.
  release_gate_worktree
  gate_wt="$(mktemp -d -t ken-pubverify-XXXXXX)"

  # ⛔ FAILS OPEN -- @librarian QA. This was one condition:
  #      if worktree_add && ! checker; then alarm; fi
  #    A FAILED `worktree add` makes the whole condition false, so the alarm is
  #    SKIPPED and control falls through to the success message -- claiming the
  #    checker is green on a checker THAT NEVER RAN. Proved by wrapping only
  #    `git worktree add`: verify_landed_tree returned 0 and printed the green
  #    sentence, after a merge.
  #
  #    This is the SAME fail-open default the runtime ring hit today in the
  #    visibility walk: a step that cannot reach an answer returning the
  #    permissive one. "Cannot determine" is a THIRD outcome and it must fail.
  if ! git worktree add --detach "$gate_wt" origin/main >/dev/null 2>&1; then
    freeze_and_alarm \
      "PR #$pr_number ($head_sha) merged, but the post-merge verification worktree could not be created. The landed state was never checked." \
      "PUBLISHER ALARM: PR #$pr_number MERGED and the landed state is UNVERIFIED.

Could not create the verification worktree at origin/main, so the currency
checker never ran. This is NOT evidence that main is green -- it is the absence
of evidence either way, after a merge that has already happened.
Not reverting."
  fi

  if ! ( cd "$gate_wt" && ./scripts/gen-doc-status.sh --check ); then
    freeze_and_alarm \
      "PR #$pr_number ($head_sha) landed and the tree OID matched, but origin/main's own currency checker is RED on the landed tree." \
      "PUBLISHER ALARM: PR #$pr_number landed with the expected tree, but the
currency checker is RED on origin/main.
Not reverting."
  fi
  release_gate_worktree

  printf 'Post-merge verification: landed tree %s matches the checked tree, and the currency checker is green on origin/main.\n' \
    "$(git rev-parse --short 'origin/main^{tree}')"
}

refuse_if_frozen

if [ "$doc_only" -eq 1 ]; then
  # `--doc-only` merges with NO CI. That is the point of the flag, and it is
  # also why this path needs the guard MOST: a doc-only merge can redden `main`,
  # and without the guard it is structurally incapable of noticing that it did.
  #
  # Measured, 2026-07-22: `a5d3a13b` ("tracker: DOC-W1 closed") touched
  # `docs/program/issues/DOC-W1.md`, which three `library/` chapters cite as a
  # currency source. It merged clean because it never ran the gate it broke.
  # `main` sat red ~25 minutes and surfaced on the next `crates/` PR, where it
  # read as that PR's own failure.
  #
  # ★ The coupling is CITATION-DIRECTED, not path-directed. The doc and build
  #   tracks are concurrent on the premise that one touches `library/` and the
  #   other `crates/`. True of file paths, FALSE of evidence:
  #   `library/manifest.toml` cites `crates/` and `docs/program/` files, so
  #   either side can invalidate the other's claim without sharing a path.
  acquire_merge_lock
  # ⛔ RE-CHECK THE FREEZE HERE, not only at startup -- @librarian QA.
  #    The startup `refuse_if_frozen` runs BEFORE the lock and, on the normal
  #    path, before a minutes-long wait for CI. Another publisher's alarm can
  #    freeze publication inside that window: this invocation passed the startup
  #    check, waits, acquires the now-released lock, and merges into a state
  #    someone else has already declared unsafe. Proved to the merge boundary.
  #    The freeze is only meaningful if it is read INSIDE the lock, immediately
  #    before evaluating and merging.
  refuse_if_frozen
  fresh_result_gate
  merge_pr
  printf 'Doc-only PR #%s merge command succeeded.\n' "$pr_number"
  verify_landed_tree
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
    # ⛔ GREEN CI IS NOT AUTHORIZATION. This is the whole point of SRC-ATTEST
    #   Part 2, and the normal path needs it MORE than `--doc-only` does, not
    #   less: this path just spent minutes polling, and `main` can advance many
    #   times inside that window. The checks that passed attest to a merge
    #   result computed when the run STARTED.
    #
    #   #885, measured: a PR's green check formed against a base with zero
    #   citations; `main` then gained those citations; the PR merged on the old
    #   green and left `main` red. Nothing in the PR changed — the base did.
    #
    #   So re-derive the merge result on a freshly fetched origin/main and run
    #   origin/main's checker on it, under the lock, immediately before merging.
    acquire_merge_lock
    # ⛔ RE-CHECK THE FREEZE HERE, not only at startup -- @librarian QA.
    #    The startup `refuse_if_frozen` runs BEFORE the lock and, on the normal
    #    path, before a minutes-long wait for CI. Another publisher's alarm can
    #    freeze publication inside that window: this invocation passed the startup
    #    check, waits, acquires the now-released lock, and merges into a state
    #    someone else has already declared unsafe. Proved to the merge boundary.
    #    The freeze is only meaningful if it is read INSIDE the lock, immediately
    #    before evaluating and merging.
    refuse_if_frozen
    fresh_result_gate
    merge_pr
    printf 'PR #%s checks passed and merge command succeeded.\n' "$pr_number"
    verify_landed_tree
    exit 0
  fi

  printf 'PR #%s checks still pending (%s); polling again in 15s.\n' "$pr_number" "$pending_count"
  sleep 15
done
