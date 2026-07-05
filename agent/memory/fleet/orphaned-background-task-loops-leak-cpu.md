---
scope: fleet
audience: (see scope README)
source: private memory `orphaned-background-task-loops-leak-cpu`
---

# Hand-rolled background bash loops can orphan and leak CPU

**Observed 2026-06-30.** Two `python3 - <<'PY'` markdown-reflow one-liners that
**spec-author** (Opus) launched as **Claude-Code background bash tasks** to wrap
`72-temporal.md` to 80 cols hit an **infinite loop** (the `wrap_para`/blockquote
logic never advances the cursor on a `>`-only line) and ran **orphaned for ~53
min each**, pegging a full core apiece and ballooning to **2.8 GB + 3.0 GB
RSS**. spec-author had finished B2 the normal way (file correctly wrapped +
merged to `main` `9527aca`); the background tasks just leaked. Found because the
operator noticed "a couple python procs at 1 CPU each." Killed (`kill -9` the
python PIDs + their parent bash) → freed 2 cores + ~6 GB on the **shared box
that OOMs under parallel builds** (ken cargo build lock wedge).

**Two durable lessons:**

1. **The federation watchdog has a host-resource blind spot.** Steward §7 scans
   *convo-level* stalls; it does NOT see host-level leaks (runaway loops,
   orphaned background tasks, RAM growth). **Add a process-scan to the Steward
   watchdog tick:** `ps -eo pid,pcpu,etime,args --sort=-pcpu | head` — flag any
   non-`claude` proc pegging ~100% CPU for minutes (esp. `python3 -` stdin
   heredocs and `*.output`-fd background tasks), diagnose via the parent chain +
   cwd (`/proc/<pid>/cwd`, parent `ps`), and reap orphans. Sibling of steward
   coldstart infra checks (which covers proxy/git/WS, not CPU/RAM).

2. **Hand-rolled text-munging loops are a hazard — diagnose by the parent
   chain.** A `python3 -` proc hides its script in the cmdline arg, but the
   **parent bash** (`ps -o args -p <ppid>`) shows the full heredoc, the **cwd**
   names the owning agent's worktree (`.worktrees/<role>`), and **fd 1/2 →
   `.../tasks/<id>.output`** marks it a Claude-Code background task. Orphan-safe
   to kill once you confirm the target artifact already landed on `main`.
   Candidate playbook caveat: prefer a bounded/`timeout`-wrapped reflow, or the
   80-col width *check* (read-only) over an in-place rewrite loop — the rewrite
   loop is where the non-termination hides.
