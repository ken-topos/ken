#!/usr/bin/env bash
# Publisher merge-gate probes -- SRC-ATTEST Part 2, acceptance rows 9, 10, 11.
#
# WHY THIS FILE EXISTS
#
# Rows 9-11 are the probes nobody writes, for the same reason the frame's row 5
# is: each requires MANUFACTURING a condition orthogonal to the change. A base
# that advances mid-evaluation, a landed tree that is not the checked tree, and
# a checker that is green during the gate and red afterwards do not occur on any
# happy path, so a happy-path suite reports green while proving none of them.
#
# ⛔ THIS HARNESS SOURCES THE REAL FUNCTION DEFINITIONS out of
#    scripts/scripted-pr-automerge.sh. It does not carry a copy. A copied gate
#    drifts from the shipped gate silently, and then the suite proves a fiction
#    -- which is exactly how scripts/pane-busy.sh came to have an arm-check
#    suite that asserted its own defect as the specification.
#
# The synthetic repository stands in for origin, and a stand-in currency checker
# stands in for gen-doc-status.sh. That substitution is deliberate and bounded:
# rows 9-11 are about the GATE'S CONTROL FLOW under adverse base/tree movement.
# Whether the real checker computes currency correctly is rows 1-7, which run
# against the real checker in crates/ken-cli/tests/library_documentation_gates.rs.
#
# Usage: scripts/publisher-gate-probes.sh [path/to/scripted-pr-automerge.sh]

set -uo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PUBLISHER="${1:-$REPO_ROOT/scripts/scripted-pr-automerge.sh}"
[ -r "$PUBLISHER" ] || { echo "cannot read publisher script: $PUBLISHER" >&2; exit 2; }

pass=0; fail=0
ok()   { pass=$((pass+1)); printf '  ✅ %s\n' "$1"; }
bad()  { fail=$((fail+1)); printf '  ❌ %s\n' "$1"; }
head_() { printf '\n=== %s\n' "$1"; }

# ---------------------------------------------------------------------------
# Extract the gate's function definitions from the real script.
#
# The slice runs from the first gate function to the line where the script's
# top-level flow begins. If either anchor ever moves, extraction yields the
# wrong slice -- so the extraction is itself asserted below before anything
# uses it. A harness that silently sources nothing passes every negative check.
# ---------------------------------------------------------------------------
GATE_LIB="$(mktemp -t ken-gatelib-XXXXXX.sh)"
sed -n '/^publisher_state_dir() {/,/^refuse_if_frozen$/p' "$PUBLISHER" \
  | sed '$d' > "$GATE_LIB"

head_ "Extraction integrity (the harness's own plumbing)"
for fn in publisher_state_dir acquire_merge_lock freeze_marker_path refuse_if_frozen \
          freeze_publication release_gate_worktree build_and_check_merge_result \
          fresh_result_gate verify_landed_tree; do
  if grep -q "^$fn() {" "$GATE_LIB"; then ok "extracted $fn"; else bad "MISSING $fn -- extraction anchors have drifted; every probe below is vacuous"; fi
done
[ "$fail" -eq 0 ] || { echo; echo "Extraction failed; refusing to report on probes that would be vacuous."; exit 1; }

# ---------------------------------------------------------------------------
# Sandbox
# ---------------------------------------------------------------------------
SANDBOX="$(mktemp -d -t ken-gateprobe-XXXXXX)"
cleanup_all() {
  # Remove any worktrees the gate registered before dropping the tree.
  if [ -d "$SANDBOX/clone" ]; then
    git -C "$SANDBOX/clone" worktree prune >/dev/null 2>&1 || true
  fi
  rm -rf "$SANDBOX" "$GATE_LIB" >/dev/null 2>&1 || true
}
trap cleanup_all EXIT

# The stand-in currency checker. Green unless a control file says otherwise, and
# it can advance origin/main from inside the evaluation window -- which is how
# probe 9 manufactures the race without stubbing the gate itself.
write_checker() {
  local dest="$1"
  mkdir -p "$(dirname "$dest")"
  cat >"$dest" <<'CHK'
#!/usr/bin/env bash
# stand-in for gen-doc-status.sh --check
CTL="${KEN_PROBE_CTL:?probe control dir not set}"
if [ -f "$CTL/advance_every" ] || { [ -f "$CTL/advance_once" ] && [ ! -f "$CTL/advanced" ]; }; then
  touch "$CTL/advanced"
  n=$(cat "$CTL/advance_n" 2>/dev/null || echo 0); n=$((n+1)); echo "$n" >"$CTL/advance_n"
  ( cd "$CTL/advancer" \
    && date +%s%N >> drift.txt \
    && git add -A >/dev/null 2>&1 \
    && git -c user.email=p@x -c user.name=p commit -q -m "out-of-band advance $n" >/dev/null 2>&1 \
    && git push -q origin HEAD:main >/dev/null 2>&1 )
fi
[ -f "$CTL/red" ] && { echo "stand-in checker: RED"; exit 1; }
exit 0
CHK
  chmod +x "$dest"
}

build_sandbox() {
  rm -rf "$SANDBOX/origin.git" "$SANDBOX/clone" "$SANDBOX/ctl" "$SANDBOX/advancer"
  mkdir -p "$SANDBOX/ctl"
  git init -q --bare "$SANDBOX/origin.git"

  git init -q "$SANDBOX/seed"
  ( cd "$SANDBOX/seed"
    git symbolic-ref HEAD refs/heads/main
    echo base > content.txt
    write_checker scripts/gen-doc-status.sh
    git add -A && git -c user.email=p@x -c user.name=p commit -q -m base
    git remote add origin "$SANDBOX/origin.git" && git push -q origin main ) >/dev/null 2>&1
  # Without this the bare repo's HEAD still names master, every clone comes up
  # on an unborn branch, and the "candidate" becomes an unrelated ROOT commit --
  # which the gate then correctly refuses as unmergeable, making every probe
  # below fail for a reason that has nothing to do with what it tests.
  git -C "$SANDBOX/origin.git" symbolic-ref HEAD refs/heads/main

  git clone -q "$SANDBOX/origin.git" "$SANDBOX/clone"
  git clone -q "$SANDBOX/origin.git" "$SANDBOX/advancer"
  ln -sfn "$SANDBOX/advancer" "$SANDBOX/ctl/advancer"

  # The candidate branch: one commit changing content, pushed to the sandbox origin.
  ( cd "$SANDBOX/clone"
    git checkout -q -B candidate origin/main
    echo candidate > content.txt
    git add -A && git -c user.email=p@x -c user.name=p commit -q -m "candidate change"
    git push -q origin candidate
    git checkout -q main ) >/dev/null 2>&1
  rm -rf "$SANDBOX/seed"
}

# Simulate what GitHub's squash-merge lands: the merge result committed onto main.
land_squash() {
  ( cd "$SANDBOX/advancer"
    git fetch -q origin
    git checkout -q -B main origin/main
    git merge -q --squash origin/candidate >/dev/null 2>&1
    git -c user.email=p@x -c user.name=p commit -q --no-verify -m "squash: candidate (#$1)"
    git push -q origin main ) >/dev/null 2>&1
}

# Run a gate function inside the sandbox clone, with the real definitions sourced
# and only the script's ambient helpers stubbed.
run_gate() {
  local snippet="$1"
  ( cd "$SANDBOX/clone"
    export KEN_PROBE_CTL="$SANDBOX/ctl"
    # shellcheck disable=SC1090
    cat >"$SANDBOX/run.sh" <<RUNNER
set -uo pipefail
die() { printf '%s\n' "\$*" >&2; exit 1; }
need_cmd() { command -v "\$1" >/dev/null 2>&1 || die "missing \$1"; }
cleanup() { :; }
head_sha="\$(git rev-parse origin/candidate)"
pr_number=9001
source "$GATE_LIB"
$snippet
RUNNER
    bash "$SANDBOX/run.sh" 2>&1 )
}

state_dir() { git -C "$SANDBOX/clone" rev-parse --path-format=absolute --git-common-dir; }
marker() { printf '%s/ken-publisher-FROZEN\n' "$(state_dir)"; }

# ---------------------------------------------------------------------------
# ⛔ Containment guard. publisher_state_dir() resolves the git common dir of the
#    CWD. If a probe ever ran with the real repository as its cwd, probe 11
#    would write a freeze marker into the REAL publisher state and block the
#    fleet's merges. Assert containment before manufacturing any freeze.
# ---------------------------------------------------------------------------
build_sandbox
head_ "Containment guard"
sd="$(state_dir)"
case "$sd" in
  "$SANDBOX"/*) ok "publisher state dir resolves inside the sandbox ($sd)" ;;
  *) bad "STATE DIR ESCAPED THE SANDBOX: $sd -- refusing to run freeze probes"; echo; exit 1 ;;
esac
[ -f "$(marker)" ] && { bad "sandbox started already frozen"; exit 1; }
ok "no freeze marker present at start (negative control for probe 11)"

# ---------------------------------------------------------------------------
# PROBE 9a -- a base advance during evaluation forces RECONSTRUCTION.
# The advance is manufactured from inside build_and_check_merge_result, by the
# checker it invokes, so the gate is not stubbed and the race is real.
# ---------------------------------------------------------------------------
head_ "Probe 9a -- base advance during evaluation forces reconstruction"
build_sandbox
touch "$SANDBOX/ctl/advance_once"
before_9a="$(git -C "$SANDBOX/clone" rev-parse origin/main)"
out="$(run_gate 'fresh_result_gate; echo "GATE_RC=$?"; echo "CHECKED_BASE=$checked_base"')"
rc9a=$?
if grep -q 'reconstructing (attempt 1)' <<<"$out"; then ok "gate reported reconstruction"; else bad "no reconstruction reported"; printf '%s\n' "$out" | sed 's/^/     | /'; fi
if grep -q 'GATE_RC=0' <<<"$out"; then ok "gate then passed on the NEW base"; else bad "gate did not recover after reconstruction"; fi
newbase="$(git -C "$SANDBOX/clone" rev-parse origin/main 2>/dev/null; git -C "$SANDBOX/advancer" rev-parse origin/main 2>/dev/null | head -1)"
if grep -q "CHECKED_BASE=$(git -C "$SANDBOX/advancer" ls-remote origin main | cut -f1)" <<<"$out"; then
  ok "the base it finally checked is the ADVANCED base, not the stale one"
else
  bad "final checked_base is not the advanced base -- the reconstruction did not re-anchor"
  printf '%s\n' "$out" | grep CHECKED_BASE | sed 's/^/     | /'
fi
[ "$before_9a" != "$(git -C "$SANDBOX/advancer" ls-remote origin main | cut -f1)" ] \
  && ok "positive control: origin/main genuinely moved during the window" \
  || bad "positive control FAILED: origin/main never moved, so 9a proved nothing"

# ---------------------------------------------------------------------------
# PROBE 9b -- a base that keeps advancing forces ABORT, not an endless loop.
# ---------------------------------------------------------------------------
head_ "Probe 9b -- a base advancing on every evaluation ABORTS after 3"
build_sandbox
touch "$SANDBOX/ctl/advance_every"
out="$(run_gate 'fresh_result_gate; echo "GATE_RC=$?"')"
if grep -q 'advanced during 3 consecutive evaluations' <<<"$out"; then ok "gate aborted with the bounded-retry diagnosis"; else bad "gate did not abort on a continuously advancing base"; printf '%s\n' "$out" | sed 's/^/     | /'; fi
if grep -q 'GATE_RC=0' <<<"$out"; then bad "gate reported SUCCESS on a base it never pinned"; else ok "gate did not report success"; fi
n="$(cat "$SANDBOX/ctl/advance_n" 2>/dev/null || echo 0)"
[ "$n" -ge 3 ] && ok "positive control: $n evaluations actually occurred (loop ran, not short-circuited)" \
               || bad "positive control FAILED: only $n evaluations -- the abort may be for another reason"

# ---------------------------------------------------------------------------
# PROBE 10 -- on the normal path the LANDED tree equals the CHECKED tree.
# ---------------------------------------------------------------------------
head_ "Probe 10 -- landed tree OID == checked tree OID on the normal path"
build_sandbox
out="$(run_gate '
  fresh_result_gate
  echo "CHECKED_TREE=$checked_tree_oid"
  ( cd '"$SANDBOX"'/advancer && git fetch -q origin && git checkout -q -B main origin/main \
      && git merge -q --squash origin/candidate >/dev/null 2>&1 \
      && git -c user.email=p@x -c user.name=p commit -q --no-verify -m "squash: candidate (#9001)" \
      && git push -q origin main ) >/dev/null 2>&1
  verify_landed_tree; echo "VERIFY_RC=$?"')"
if grep -q 'VERIFY_RC=0' <<<"$out"; then ok "post-merge verification PASSED on a faithful squash"; else bad "verification failed on the normal path"; printf '%s\n' "$out" | sed 's/^/     | /'; fi
if grep -q 'matches the checked tree' <<<"$out"; then ok "gate reported the tree match explicitly"; else bad "no match report"; fi
ct="$(grep -o 'CHECKED_TREE=[0-9a-f]*' <<<"$out" | cut -d= -f2)"
lt="$(git -C "$SANDBOX/advancer" rev-parse 'origin/main^{tree}' 2>/dev/null)"
if [ -n "$ct" ] && [ "$ct" = "$lt" ]; then ok "independently confirmed: checked $ct == landed $lt"; else bad "OIDs differ independently: checked=$ct landed=$lt"; fi
[ -f "$(marker)" ] && bad "NEGATIVE CONTROL FAILED: the happy path wrote a freeze marker" || ok "negative control: happy path left publication UNFROZEN"

# ---------------------------------------------------------------------------
# PROBE 11a -- a planted landed-tree MISMATCH alarms, freezes, does NOT revert.
# ---------------------------------------------------------------------------
head_ "Probe 11a -- planted landed-tree mismatch: alarm + freeze + NO revert"
build_sandbox
out="$(run_gate '
  fresh_result_gate
  echo "CHECKED_TREE=$checked_tree_oid"
  ( cd '"$SANDBOX"'/advancer && git fetch -q origin && git checkout -q -B main origin/main \
      && git merge -q --squash origin/candidate >/dev/null 2>&1 \
      && echo "out-of-band writer" > intruder.txt && git add -A \
      && git -c user.email=p@x -c user.name=p commit -q --no-verify -m "squash + intruder" \
      && git push -q origin main ) >/dev/null 2>&1
  echo "LANDED_BEFORE=$( (cd '"$SANDBOX"'/advancer && git ls-remote origin main | cut -f1) )"
  verify_landed_tree; echo "VERIFY_RC=$?"')"
if grep -q 'LANDED TREE IS NOT THE' <<<"$out"; then ok "alarmed on the mismatch"; else bad "no alarm on a planted mismatch"; printf '%s\n' "$out" | sed 's/^/     | /'; fi
if grep -q 'VERIFY_RC=0' <<<"$out"; then bad "reported success despite a mismatch"; else ok "did not report success"; fi
if [ -f "$(marker)" ]; then ok "publication FROZEN (marker written)"; else bad "no freeze marker written"; fi
lb="$(grep -o 'LANDED_BEFORE=[0-9a-f]*' <<<"$out" | cut -d= -f2)"
la="$(cd "$SANDBOX/advancer" && git ls-remote origin main | cut -f1)"
if [ -n "$lb" ] && [ "$lb" = "$la" ]; then ok "NO REVERT: origin/main unchanged at ${la:0:8} after the alarm"; else bad "origin/main MOVED after the alarm ($lb -> $la) -- something reverted"; fi
# The freeze must actually bite the next publish.
out2="$(run_gate 'refuse_if_frozen; echo "REFUSE_RC=$?"')"
if grep -q 'PUBLICATION IS FROZEN' <<<"$out2"; then ok "the freeze BITES: the next publish refuses to start"; else bad "freeze marker written but the next publish proceeded"; fi
rm -f "$(marker)"

# ---------------------------------------------------------------------------
# PROBE 11b -- a planted RED checker on the landed tree alarms, freezes, does
# NOT revert. The checker is green during the gate and red afterwards, which is
# the only way to exercise the second arm: with a matching tree OID that arm is
# redundant by construction, so it can only be reached if the OID comparison is
# itself wrong. This plants exactly that world.
# ---------------------------------------------------------------------------
head_ "Probe 11b -- planted red checker on the landed tree: alarm + freeze + NO revert"
build_sandbox
out="$(run_gate '
  fresh_result_gate
  ( cd '"$SANDBOX"'/advancer && git fetch -q origin && git checkout -q -B main origin/main \
      && git merge -q --squash origin/candidate >/dev/null 2>&1 \
      && git -c user.email=p@x -c user.name=p commit -q --no-verify -m "squash: candidate" \
      && git push -q origin main ) >/dev/null 2>&1
  touch "$KEN_PROBE_CTL/red"
  echo "LANDED_BEFORE=$( (cd '"$SANDBOX"'/advancer && git ls-remote origin main | cut -f1) )"
  verify_landed_tree; echo "VERIFY_RC=$?"')"
if grep -q 'currency checker is RED on origin/main' <<<"$out"; then ok "alarmed on the red landed tree"; else bad "no alarm on a planted red checker"; printf '%s\n' "$out" | sed 's/^/     | /'; fi
if grep -q 'VERIFY_RC=0' <<<"$out"; then bad "reported success despite a red landed tree"; else ok "did not report success"; fi
if [ -f "$(marker)" ]; then ok "publication FROZEN (marker written)"; else bad "no freeze marker written"; fi
lb="$(grep -o 'LANDED_BEFORE=[0-9a-f]*' <<<"$out" | cut -d= -f2)"
la="$(cd "$SANDBOX/advancer" && git ls-remote origin main | cut -f1)"
if [ -n "$lb" ] && [ "$lb" = "$la" ]; then ok "NO REVERT: origin/main unchanged at ${la:0:8} after the alarm"; else bad "origin/main MOVED after the alarm ($lb -> $la)"; fi
rm -f "$(marker)"

printf '\n=== %s passed, %s failed\n' "$pass" "$fail"
[ "$fail" -eq 0 ]
