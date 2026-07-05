---
scope: build
audience: (see scope README)
source: private memory `ken-cargo-build-lock-wedge`
---

# The ken-cargo + sccache build-lock wedge

`scripts/ken-cargo` (COORDINATION §12 build wrapper) takes a machine-wide
`flock` on `/tmp/ken-build-locks/build.lock` (`KEN_BUILD_SLOTS=1` → one build
across ALL ~26 agents at once) with a 1800s wait timeout. The wrapper does
`exec 9>"$lockdir/build.lock"; flock -w 1800 9; run()` where
`run() { exec cargo "$@"; }`. With `RUSTC_WRAPPER=sccache` (set by
`scripts/ken-env.sh`), cargo invokes sccache, which **inherits fd 9** (the
locked fd — not close-on-exec) and the sccache **server daemonizes** (PPid 1)
holding the flock — so after every build the machine-wide lock stays held
**forever**, and every subsequent `ken-cargo` queues behind it until the 1800s
timeout, then fails. Looks exactly like a "stalled build." Confirmed twice on
2026-06-29 (F4); it blocked the federation's first build WP's QA gate.

**Diagnose:** find who holds the lock by scanning `/proc/*/fd/*` for an fd
that resolves to `build.lock`, then reading that pid's `fdinfo` for the
lock line — the holder is a `sccache` daemon (`comm=sccache`, PPid=1), shown as
`lock: 1: FLOCK ADVISORY WRITE`. The queued `ken-cargo` victims show
`flock -w 1800 9` stuck in `locks_lock_inode_wait`, 00:00:00 CPU, no `target/`
activity.

**Workaround:** `kill <wedged-sccache-pid>` (sccache auto-restarts on next
compile; on-disk cache `SCCACHE_DIR` untouched; safe — no build is in flight
when it's just holding the lock), OR run that build with `unset RUSTC_WRAPPER`
after sourcing `ken-env.sh` (no sccache daemon → no fd-9 inheritance → no wedge;
warm `target/` keeps it fast). The lock frees immediately.

**One-line fix** (Steward escalating to the operator 2026-06-29 with this exact
fix): in `scripts/ken-cargo`, change `run() { exec cargo "$@"; }` →
`run() { cargo "$@" 9>&-; }` — `9>&-` closes fd 9 to cargo so sccache/rustc
can't inherit the lock; bash keeps fd 9 and releases it on exit. (The `slots>1`
`exec sem` path is unaffected — fd 9 isn't opened there.) Bash stays alive (one
extra process) holding the lock for cargo's duration instead of `exec`-replacing
into cargo.

**Verify before applying:** check `scripts/ken-cargo` for the `9>&-` fix — once
it lands on `main` this wedge is gone and this memory is obsolete (delete it).
Until then, a "stalled" `ken-cargo` is most likely this wedge. Found during the
compact wiped memory reflog first re-orientation on F4; reported as a
`bug`-class fleet item.
