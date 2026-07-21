#!/usr/bin/env bash
# Per-test-binary timing for a CI run, derived from the job log.
#
# WHY THIS EXISTS: `cargo test` emits no per-test timing, which was read as
# "we cannot identify slow tests." That was wrong. `cargo test` runs each test
# binary to completion before starting the next, and the log timestamps every
# `Running <target>` line — so the gap between consecutive `Running` lines IS
# that binary's wall-clock duration. The data was always there.
#
# LIMITS — read these before quoting a number:
#   * Granularity is the BINARY, not the individual #[test]. A binary with 40
#     tests reports one number. For per-test timing you need cargo-nextest
#     (program item C2).
#   * The final binary's duration is measured to the step's end, so it absorbs
#     any trailing teardown. Treat the last row as an upper bound.
#   * Requires the serial-binary execution model. If the suite moves to
#     nextest, this script's premise no longer holds — use `nextest --profile
#     ci` timing output instead and retire this.
#
# Usage:
#   scripts/ci-test-timings.sh              # most recent successful run
#   scripts/ci-test-timings.sh <run-id>
#
# Needs GH_TOKEN (see .devcontainer/mint-gh-token.sh).
set -euo pipefail

run_id="${1:-}"

if [ -z "${GH_TOKEN:-}" ] && ! gh auth status >/dev/null 2>&1; then
  echo "ci-test-timings: no GitHub credentials." >&2
  echo "  export GH_TOKEN=\$(bash .devcontainer/mint-gh-token.sh | tail -1)" >&2
  exit 1
fi

if [ -z "$run_id" ]; then
  run_id=$(gh run list --limit 20 --json databaseId,conclusion \
    --jq 'map(select(.conclusion=="success"))[0].databaseId')
  echo "ci-test-timings: using most recent successful run $run_id" >&2
fi

job_id=$(gh api "repos/{owner}/{repo}/actions/runs/$run_id/jobs" \
  --jq '.jobs[] | select(.name=="build + test") | .id')
if [ -z "$job_id" ]; then
  echo "ci-test-timings: no 'build + test' job in run $run_id" >&2
  exit 1
fi

log=$(mktemp); trap 'rm -f "$log"' EXIT
gh api "repos/{owner}/{repo}/actions/jobs/$job_id/logs" > "$log"

# The step boundary: `cargo test` prints `Finished \`test\` profile` when the
# last test target links, immediately before the first binary runs. Everything
# before that is compilation, everything after is execution.
gh api "repos/{owner}/{repo}/actions/runs/$run_id/jobs" \
  --jq ".jobs[] | select(.id==$job_id) | .steps[] | select(.name==\"Build\" or .name==\"Test\") |
        \"\(.name): \(.started_at) -> \(.completed_at)\"" >&2

python3 - "$log" <<'PY'
import re, sys
from datetime import datetime

lines = open(sys.argv[1], errors='replace').read().splitlines()
runs, last_ts = [], None
for l in lines:
    m = re.match(r'([0-9T:.-]+)Z', l)
    if m:
        last_ts = m.group(1)
    r = re.search(r'\bRunning (\S+)', l)
    if r and last_ts:
        runs.append((datetime.fromisoformat(last_ts), r.group(1)))

if not runs:
    sys.exit("ci-test-timings: no `Running` lines found — did the format change?")

end = datetime.fromisoformat(last_ts)
rows = []
for i, (t0, name) in enumerate(runs):
    t1 = runs[i + 1][0] if i + 1 < len(runs) else end
    rows.append(((t1 - t0).total_seconds(), name))

rows.sort(reverse=True)
total = sum(d for d, _ in rows)
print(f"\n{len(rows)} test binaries, {total/60:.1f} min of execution "
      f"(serial: one binary at a time)\n")
print(f"{'RANK':>4}  {'SECONDS':>8}  {'%':>5}  {'CUM%':>5}  BINARY")
cum = 0.0
for i, (d, n) in enumerate(rows, 1):
    cum += d
    if d < 1.0 and i > 25:
        rest = len(rows) - i + 1
        print(f"{'':>4}  {sum(x for x,_ in rows[i-1:]):>8.1f}  "
              f"{100*sum(x for x,_ in rows[i-1:])/total:>5.1f}  "
              f"{'100.0':>5}  ... {rest} remaining binaries, each under 1s")
        break
    print(f"{i:>4}  {d:>8.1f}  {100*d/total:>5.1f}  {100*cum/total:>5.1f}  {n}")

for k in (3, 10, 25):
    if k <= len(rows):
        print(f"\nTop {k:>2} binaries = {100*sum(d for d,_ in rows[:k])/total:.1f}% "
              f"of execution ({sum(d for d,_ in rows[:k])/60:.1f} min)")
PY
