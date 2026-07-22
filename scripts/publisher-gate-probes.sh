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

# ⛔ FIRST: is the slice WELL-FORMED? An earlier version asked only "are the
#    nine functions I know about present?", which asserts each function's
#    OPENING LINE and never its body. Truncation removes TAILS, so that check
#    was structurally blind to the one drift it existed to catch: a slice cut
#    mid-function passed all nine name assertions, failed `bash -n`, and -- the
#    runner having no `set -e` and no guard on `source` -- still exited 0 with
#    every negative assertion passing vacuously.
#
#    That is the exact failure this file's own header warns about. It was
#    written down at the top and implemented forty lines below it.
#
#    `bash -n` subsumes the whole class: it catches ANY truncation regardless of
#    where it lands, and it needs no one to maintain a list.
if bash -n "$GATE_LIB" 2>/dev/null; then
  ok "the extracted slice PARSES (bash -n) -- not truncated mid-function"
else
  bad "the extracted slice does NOT parse -- truncated or malformed; every probe below would be vacuous"
fi

# ⛔ SECOND: assert SET EQUALITY against the set DERIVED from the publisher's
#    gate region -- not containment against a hand-kept list. Containment proves
#    "my names are present"; it cannot notice a TENTH function, so anything
#    added to the gate is invisible to it forever. Measured: `cleanup_gate` was
#    in the slice and unasserted, green, live on the branch.
#    Adding it to a list is the enumeration move. Deriving the list is the fix.
expected_fns="$(sed -n 's/^\([a-z_][a-z_0-9]*\)() {$/\1/p' "$GATE_LIB" | sort -u)"
declared_fns="$( ( set +u; unset -f $(compgen -A function) 2>/dev/null
                   cleanup() { :; }; die() { :; }   # the slice installs an EXIT trap calling these
                   # shellcheck disable=SC1090
                   source "$GATE_LIB" >/dev/null 2>&1
                   compgen -A function ) | sort -u )"
if [ -z "$expected_fns" ]; then
  bad "derived ZERO function names from the slice -- extraction anchors have drifted entirely"
else
  ok "derived $(printf '%s\n' "$expected_fns" | wc -l | tr -d ' ') gate function names from the publisher"
fi
missing="$(comm -23 <(printf '%s\n' "$expected_fns") <(printf '%s\n' "$declared_fns"))"
if [ -z "$missing" ]; then
  ok "every function in the gate region actually DEFINES on source (set equality, not containment)"
else
  bad "in the slice but not defined after sourcing: $(printf '%s' "$missing" | tr '\n' ' ')"
fi

# ⛔ THIRD: every gate function must be ACCOUNTED FOR -- driven by a named probe,
#    or excluded with a stated reason. Asserted as SET EQUALITY against the
#    derived set, so a function added to the gate cannot slip in silently.
#
#    ⚠ This is a DECLARED map, not a measurement. An earlier version grepped the
#    probe file for each function name, which is a PROXY: `freeze_publication`
#    is genuinely exercised (probe 11 asserts the marker it writes) but is never
#    named, so the proxy called it uncovered -- while a mere mention in a comment
#    would have satisfied it. A name-grep measures neither direction correctly.
#    A declared map at least states a claim someone can check.
COVERAGE="
acquire_merge_lock|probe 8 (cross-worktree mutual exclusion)
build_and_check_merge_result|probes 9a/9b/10/11 (every probe runs it)
fresh_result_gate|probes 9a (reconstruct) and 9b (abort)
verify_landed_tree|probes 10, 11a, 11b
refuse_if_frozen|probe 11a (the freeze must BITE the next publish)
freeze_publication|probes 11a/11b + probe 12c (persistence failure is reported)
freeze_and_alarm|probe 12c (terminal text conditional on persistence) + 11a/11b
publisher_state_dir|EXCLUDED: pure accessor, driven by the containment guard
freeze_marker_path|EXCLUDED: pure accessor, driven by every freeze assertion
release_gate_worktree|EXCLUDED: cleanup helper, no observable of its own
cleanup_gate|EXCLUDED: EXIT trap, not reachable as a unit under test
"
declared_cov="$(printf '%s' "$COVERAGE" | sed '/^$/d' | cut -d'|' -f1 | sort -u)"
if [ "$declared_cov" = "$expected_fns" ]; then
  ok "coverage map accounts for ALL $(printf '%s\n' "$expected_fns" | wc -l | tr -d ' ') gate functions (set equality)"
else
  bad "coverage map does not match the gate's function set"
  printf '     | in gate, unaccounted: %s\n' "$(comm -23 <(printf '%s\n' "$expected_fns") <(printf '%s\n' "$declared_cov") | tr '\n' ' ')"
  printf '     | in map, not in gate:  %s\n' "$(comm -13 <(printf '%s\n' "$expected_fns") <(printf '%s\n' "$declared_cov") | tr '\n' ' ')"
fi
printf '%s' "$COVERAGE" | sed '/^$/d' | grep 'EXCLUDED' | while IFS='|' read -r f r; do
  printf '  ▪ %s -- %s\n' "$f" "$r"
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
# ⛔ GUARD THE SOURCE. Without this the runner has no 'set -e', so a slice that
#    fails to parse produces a syntax error, execution CONTINUES into the
#    snippet, and the run exits 0 with every negative assertion passing
#    vacuously -- the harness reporting green about a gate it never loaded.
source "$GATE_LIB" || { echo "HARNESS BROKEN: could not source the gate slice"; exit 3; }
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
# PROBE 8 -- two publisher invocations cannot both hold the merge critical
# section. Proved ACROSS TWO WORKTREES sharing one object store, never two
# shells in one: the lock lives in the COMMON git dir, so a per-worktree lock
# path would pass a same-worktree test and then never contend in production --
# a lock that always succeeds, indistinguishable from no lock until it matters.
#
# This row was previously discharged by a hand-run transcript. A transcript is
# not re-runnable and does not fail when someone changes the lock path, which is
# exactly the change it exists to catch.
# ---------------------------------------------------------------------------
head_ "Probe 8 -- the merge lock excludes ACROSS WORKTREES"
build_sandbox
LOCK_WT="$SANDBOX/second-worktree"
git -C "$SANDBOX/clone" worktree add --detach "$LOCK_WT" HEAD >/dev/null 2>&1

lock_snippet='acquire_merge_lock && echo LOCK=ACQUIRED || echo LOCK=REFUSED'
mk_runner() {
  cat >"$1" <<RUNNER
set -uo pipefail
die() { printf '%s\n' "\$*" >&2; echo LOCK=REFUSED; exit 1; }
need_cmd() { command -v "\$1" >/dev/null 2>&1 || die "missing \$1"; }
cleanup() { :; }
head_sha=HEAD
pr_number=9001
source "$GATE_LIB" || { echo "HARNESS BROKEN"; exit 3; }
$lock_snippet
sleep "\${HOLD:-0}"
RUNNER
}
mk_runner "$SANDBOX/lock.sh"

# NEGATIVE CONTROL FIRST: with nobody holding it, the second worktree must
# ACQUIRE. Without this, "REFUSED" below could mean the lock is simply broken.
out="$( cd "$LOCK_WT" && bash "$SANDBOX/lock.sh" 2>&1 )"
if grep -q 'LOCK=ACQUIRED' <<<"$out"; then ok "control: uncontended, the second worktree ACQUIRES"; else bad "control FAILED: lock refused with nobody holding it"; fi

# Now hold it from worktree A and attempt from worktree B.
( cd "$SANDBOX/clone" && HOLD=6 bash "$SANDBOX/lock.sh" >"$SANDBOX/holder.out" 2>&1 ) &
holder_pid=$!
for _ in 1 2 3 4 5 6 7 8 9 10; do grep -q 'LOCK=' "$SANDBOX/holder.out" 2>/dev/null && break; sleep 0.3; done
if grep -q 'LOCK=ACQUIRED' "$SANDBOX/holder.out" 2>/dev/null; then ok "worktree A ACQUIRED the lock"; else bad "worktree A failed to acquire"; fi
out="$( cd "$LOCK_WT" && bash "$SANDBOX/lock.sh" 2>&1 )"
if grep -q 'LOCK=REFUSED' <<<"$out"; then ok "worktree B REFUSED while A holds it (cross-worktree exclusion)"; else bad "worktree B acquired a lock A was holding -- the lock does not contend"; fi
if grep -q 'another merge critical section is active' <<<"$out"; then ok "refusal carries the diagnosis, not a bare failure"; else bad "refusal has no diagnosis"; fi
wait "$holder_pid" 2>/dev/null
# And it must be RELEASED, not permanently stuck.
out="$( cd "$LOCK_WT" && bash "$SANDBOX/lock.sh" 2>&1 )"
if grep -q 'LOCK=ACQUIRED' <<<"$out"; then ok "lock RELEASED after the holder exits"; else bad "lock never released -- the gate would deadlock the fleet"; fi
git -C "$SANDBOX/clone" worktree remove --force "$LOCK_WT" >/dev/null 2>&1 || true

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


# ---------------------------------------------------------------------------
# PROBE 12 -- @librarian QA, three fail-closed defects in Part 2. Each is a
# state that had to be MANUFACTURED, and each previously produced a green,
# confident, wrong answer.
# ---------------------------------------------------------------------------
head_ "Probe 12a -- the freeze must be re-read INSIDE the lock, on BOTH paths"
# ⚠ STRUCTURAL, and labelled as such. The ordering under test lives in the
#   script's TOP-LEVEL flow, which is not in the sourced slice, so this asserts
#   the publisher's control flow as text rather than executing it.
#
#   ⛔ The first version of this probe drove `refuse_if_frozen` twice from its
#      OWN snippet and then asserted the merge boundary was not reached. It
#      PASSED against the unfixed publisher, because it was testing the sequence
#      the probe wrote, not the sequence the script runs. A probe that supplies
#      the behaviour it is checking for is vacuous no matter how green it is.
flow="$(sed -n '/^refuse_if_frozen$/,$p' "$PUBLISHER")"
# One record per publish path: from each `acquire_merge_lock` to the FIRST
# `merge_pr` after it, flattened to a single line so the reader below counts
# PATHS and not lines. (The first version read line-by-line and reported 8
# "paths" for 2 -- a wrong denominator makes the ratio meaningless even when
# the pass/fail verdict happens to land correctly.)
blocks="$(printf '%s\n' "$flow" | awk '
  /acquire_merge_lock/ { cap=1; buf=""; next }
  cap && /merge_pr/    { print buf; cap=0; next }
  cap                  { buf = buf " " $0 }')"
paths="$(printf '%s\n' "$blocks" | sed '/^[[:space:]]*$/d' | wc -l | tr -d ' ')"
guarded="$(printf '%s\n' "$blocks" | sed '/^[[:space:]]*$/d' | grep -c 'refuse_if_frozen' || true)"
if [ "$paths" -eq 2 ]; then ok "found exactly 2 publish paths (doc-only + normal) between the lock and the merge"; else bad "found $paths publish paths, expected 2 -- the scan is not seeing what it claims to"; fi
if [ "$paths" -gt 0 ] && [ "$guarded" -eq "$paths" ]; then
  ok "ALL $paths paths re-read the freeze inside the lock before merging"
else
  bad "only $guarded of $paths paths re-check the freeze inside the lock -- a freeze created during the CI wait would be ignored"
fi

head_ "Probe 12b -- an unbuildable verification worktree must NOT read as green"
build_sandbox
out="$(run_gate '
  fresh_result_gate
  ( cd '"$SANDBOX"'/advancer && git fetch -q origin && git checkout -q -B main origin/main \
      && git merge -q --squash origin/candidate >/dev/null 2>&1 \
      && git -c user.email=p@x -c user.name=p commit -q --no-verify -m squash \
      && git push -q origin main ) >/dev/null 2>&1
  # Break ONLY `git worktree add`; every other git operation stays real.
  git() { if [ "${1:-}" = "worktree" ] && [ "${2:-}" = "add" ]; then return 1; fi; command git "$@"; }
  verify_landed_tree; echo "VERIFY_RC=$?"')"
if grep -q 'VERIFY_RC=0' <<<"$out"; then bad "FAILS OPEN: reported success though the checker never ran"; else ok "did not report success when the checker could not run"; fi
if grep -q 'UNVERIFIED' <<<"$out"; then ok "alarmed as UNVERIFIED -- absence of evidence, not evidence of green"; else bad "no UNVERIFIED alarm"; fi
if grep -q 'currency checker is green' <<<"$out"; then bad "printed the GREEN sentence about a checker that never ran"; else ok "did not print the green sentence"; fi
if [ -f "$(marker)" ]; then ok "publication FROZEN"; else bad "no freeze marker"; fi
rm -f "$(marker)"

head_ "Probe 12c -- unpersistable freeze, driven through the REAL caller"
# ⛔ The previous 12c substituted `echo REACHED_DIE_POINT` for the caller's
#    terminal diagnosis, so it could not observe what the operator actually
#    reads. @architect ran the real `verify_landed_tree` mismatch branch with an
#    unwritable marker and got BOTH "Subsequent publishes will NOT be stopped"
#    and "Publication is now FROZEN ... clear the freeze by hand." The second is
#    false and it is the one an operator acts on.
#    ⇒ Drive the REAL branch and assert on the COMPLETE output. A probe that
#      replaces the thing under test cannot observe the thing under test.
build_sandbox
out="$(run_gate '
  set -e
  fresh_result_gate >/dev/null 2>&1
  ( cd '"$SANDBOX"'/advancer && git fetch -q origin && git checkout -q -B main origin/main \
      && git merge -q --squash origin/candidate >/dev/null 2>&1 \
      && echo intruder > intruder.txt && git add -A \
      && git -c user.email=p@x -c user.name=p commit -q --no-verify -m "squash + intruder" \
      && git push -q origin main ) >/dev/null 2>&1
  freeze_marker_path() { printf "%s\n" "/nonexistent-dir-$$/ken-publisher-FROZEN"; }
  verify_landed_tree || true
  echo PROBE_END=1')"

if grep -q 'LANDED TREE IS NOT THE' <<<"$out"; then ok "the real alarm fired and still reports the TRIGGER"; else bad "trigger not reported"; fi
if grep -q 'COULD NOT BE PERSISTED' <<<"$out"; then ok "the persistence failure is reported"; else bad "persistence failure not reported"; fi
# ★ The discriminator: the COMPLETE output must never claim protection exists.
if grep -qE 'Publication is now FROZEN|Publication is FROZEN|clear the freeze' <<<"$out"; then
  bad "output CLAIMS PUBLICATION IS FROZEN while no marker exists -- the operator acts on a protection that is not there"
else
  ok "output never claims FROZEN/protection after persistence failed"
fi
if grep -q 'NOT.*frozen\|will proceed unblocked' <<<"$out"; then ok "and it says plainly that the next publish is NOT blocked"; else bad "does not state that main is unprotected"; fi
# And prove it: the next publish really does proceed.
out2="$(run_gate 'refuse_if_frozen && echo NEXT_PUBLISH_PROCEEDED=1')"
if grep -q 'NEXT_PUBLISH_PROCEEDED=1' <<<"$out2"; then ok "proved: the next publish proceeds (so the FROZEN claim would have been false)"; else bad "next publish blocked -- then a marker exists and this probe is not testing what it claims"; fi

printf '\n=== %s passed, %s failed\n' "$pass" "$fail"
[ "$fail" -eq 0 ]
