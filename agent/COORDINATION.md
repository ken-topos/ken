# Ken coordination law (read by every agent)

Cross-cutting rules for every Ken agent, regardless of role, team, or model.
Role-specific discipline is in `playbooks/`; model tiers are in `MODELS.md`; the
git/PR model is in `../docs/program/04-git-and-integration.md`. These rules are adapted from
hard-won mootup team lessons; each exists because skipping it caused a real stall
or a real bug. They must hold identically across Opus, GLM, and DeepSeek agents.

## 0. The shape: a ring of rings

- **Within a team — a sequential token-ring.** Generally one agent is active at a
  time; the others are in a supporting role, called on when the active agent
  needs them (e.g. an implementer asks for a clarification). Keeping the whole
  team on one task maximizes coherence and effectiveness. Do not fan a team onto
  several tasks to chase parallelism — coherence beats it. This includes waiting
  on CI: a team **waits idle** for its CI run rather than pipelining or stacking
  PRs (ADR 0002). Idle is cheap and load-friendly; throughput comes from other
  teams' rings, not from this one multitasking.
- **Across teams — parallel.** The teams are independent rings spinning at once;
  that parallelism is the entire reason the work is articulated into teams. The
  rings couple at only three points: PRs to `main` (via the Integrator), the
  roadmap gate dependencies, and the **sanctioned cross-team query edges** (§9,
  §11). Keep that coupling thin — it is what serializes the federation if abused.

## 1. Event-driven, never poll

After you finish a unit of work or hand off, **post, set status, and stop.** Do
not `/loop`, self-wake, or poll for replies. The notification system delivers
what you need; polling burns tokens for zero value. A missing notification is a
*stall* — catching stalls is the team leader's watchdog job, not yours. Only team
leaders, the Integrator, and the Steward run schedulers.

## 2. Mention discipline

**Mention an agent iff the next move in the workflow is theirs.** Every mention
costs the recipient tokens and fires a notification.

- Handoff that passes work to B → mention **B only**.
- "Your request is done," nothing pending → mention **nobody**.
- **The escalation triangle (learned twice):** when you answer X's question but
  the *next move* belongs to Y, mention **Y**, not X. Naming Y in prose without a
  real mention is the classic silent stall.

## 3. Status = what you're doing, in your own words

Three liveness signals exist; only the third is yours to post:
1. connection (automatic — never post "I'm online");
2. activity (file/transcript mtime — automatic);
3. **semantic status** — "drafting K2 conversion", "blocked-on-spec: OQ-17".
   Agent-composed, never auto-classified. Update it on: receiving a handoff,
   completing one, changing focus, and becoming idle or blocked.

## 4. Threads are the spine

One mootup thread per work item; the kickoff message *is* the spine. All handoffs,
questions, status, and retros for that item are **replies in that thread** — a
top-level post fragments the work. After any context reset/compaction, resolve
the live thread from fresh context; do **not** reuse a thread/event ID from a
summarized memory (it may be stale).

## 5. Decisions are for judgment, not deduction

Open a mootup Decision (`propose_decision`) for choices with tradeoffs where a
reasonable peer might differ — kernel/semantics design, an API shape, a
content-store policy. Do **not** open one for deductive/mechanical choices (a bug
fix is not a decision). Decisions are how future agents query *why* Ken is the
way it is. PR-merge approvals are also Decisions (see the integrator playbook).

## 6. Resolve when structurally determined; escalate only real forks

Before escalating or querying another team, ask: *is there a strategic choice
between materially different futures?* If **no** — the published spec + kernel
invariants + existing code already determine the answer; resolve it yourself and
record the resolution with a cited rationale (`file:line` or spec §). If **yes**
— escalate. For clean-room questions, "the published spec" means `/spec`, never
prototype source. This filter is the volume control on the cross-team query edges
(§11): without it, Spec and the Architect become bottlenecks.

## 7. Ground every premise before locking

Before locking a spec, ADR, or design claim, verify each premise against reality:
"X exists" → grep for it; "matches pattern Y" → read Y end-to-end. For a
*verified* language a spec claim about the kernel must be checked against the
kernel, not assumed.

## 8. Message-type taxonomy (routing metadata)

Tag each message with a type; the **first line is the thread title** — no
`[TYPE]` prefix in the body. Types: `kickoff`, `question`, `pr_ready` (points at
a GitHub PR), `review_request`, `blocked`, `bug`, `status_update`, `retro`,
`decision`.

## 9. Topology is invariant — including the query edges

Who PRs to whom, who reviews, who merges, and **which cross-team query edges
exist** is operator-owned and fixed. The sanctioned edges are exactly:

- any team → **Spec** leader — behavioral-contract questions ("what must this do
  to be correct?").
- any team → **Architect** — component-design questions ("how should I structure
  this / which design?").
- any team → **Steward** — scope/priority (forwarded to the operator),
  workflow/process, and research requests.
- any team → **Integrator** — merge status (usually via the team's own leader).

Agents may improve *what they do inside a node*, never *add a communication edge
or a review cycle* between nodes. When integrating a retro lesson, reject any
carry-forward that would add/move an edge — and do not soften the rejection to
"candidate, watch one more run." That softening is how coordination entropy
creeps in.

## 10. Knowledge promotion: retro → synthesis → promotion ladder

- After each shipped work item, leave a one-or-two-bullet **retro** in its thread.
- The **Steward** harvests retros across teams and promotes lessons up a ladder
  (see the steward playbook): team-local → archetype source → this file.
- A lesson promotes only when it passes all three: **(a) validated across ≥3 runs
  *or* independently in ≥2 teams, (b) effort-/model-/operator-agnostic, (c) a
  normative rule, not a one-off fact.** Exception: an explicit operator
  correction promotes on a single data point. On promotion, retire the source
  note atomically. Cross-team replication is a *stronger* generalization signal
  than single-team repetition — use it.

## 11. Cross-team query protocol

The edges in §9 are thin synchronous couplings between otherwise-parallel rings.
Use them sparingly and always event-driven:

1. **Filter first (§6).** Most "what should I do here" answers are already in
   `/spec` + conformance + the component design. Only a genuine gap or fork earns
   a query.
2. **Ask and stop.** Post a `question` mentioning **only** the target's leader
   (Spec leader / Architect / Steward), set status `blocked-on-<target>`, and
   stop. Resume on notification — never poll.
3. **Bias to staying on-task.** Your team's default is to *wait out* a short
   block, preserving ring coherence; your leader reorders to an independent ready
   task only when the block is genuinely long.
4. **Front-desk on the answering side.** The target's leader triages to protect
   its own ring's focus — answers trivial/known questions itself, batches
   non-urgent ones, interrupts its active agent only for true blockers.
5. **Outcomes:** a quick interpretive answer; a **durable artifact edit** (a
   `/spec` clarification + conformance test, or a component-design note) so the
   next team never asks again; or, for a real fork, a **Decision**. Every query
   should leave the shared artifacts better — the query rate is a health gauge,
   and it should decay over time.

## 12. Resource discipline (shared 8-core / 16 GB laptop)

Build parallelism multiplies with agent parallelism; the dev box is small.
Violating this OOMs the machine and stalls everyone. Full rationale +
configuration: `../docs/ops/compute-budget.md`.

- **Build and test only through `scripts/ken-cargo`** — never raw `cargo build`/
  `cargo test`. It holds a machine-wide lock (`KEN_BUILD_SLOTS`, default 1) so
  only one build runs at a time across all agents. Bypassing it is the fastest
  way to swap-death the box.
- **Scope to the touched crate** (`-p <crate>`), not `--workspace`. Full-workspace
  builds, the conformance suite, and any `--release`/LTO build run **in CI**, not
  on the laptop. Lean on CI green (the Integrator does), don't reproduce it
  locally.
- **`source scripts/ken-env.sh`** at session start for the shared `sccache` +
  `CARGO_HOME`, so you don't recompile dependencies other agents already built.
- **Idle = paused.** A resident agent costs RAM even when not building. If your
  ring is blocked or waiting (including waiting on a CI run — ADR 0002), quiesce;
  don't hold the box hot.
- This is a *current-hardware* constraint, not a design value — it relaxes as
  hardware grows (the Steward/operator raises the caps; do not raise them
  unilaterally).

## 13. Liveness: keep the rings turning

Token rings stall — an agent finishes, forgets to hand off, and the ring goes
quiet. Treat stalls as the **default** failure mode, defended in depth by three
recurring watchdogs, each catching the layer below it failing:

- **Team leader → its own ring.** Enumerated patterns: handed-off-but-silent,
  PR-open-no-reviewer, blocked-without-a-blocker-mention, idle-with-ready-work.
- **Integrator → the PR pipeline.** Green-draft-not-marked-ready,
  ready-but-unreviewed-past-interval, approved-but-unmerged, merge-queue stuck.
- **Steward → the federation (the backstop).** A whole team idle, a *stalled
  leader*, a dropped cross-team query, a blocked dependency chain, no movement
  toward the active gate. The watcher-of-watchers — it catches a watchdog that
  itself stalled.

Rules for every layer:
- **Enumerate the stall patterns explicitly** in the watchdog prompt — a generic
  "check for activity" misses the nuance.
- **Diagnose before you restart.** Capture the stalled agent's state first; a
  blind restart no-ops a permission-prompt or rate-limit stall.
- **Distinguish waiting from stalling.** A team idle while its CI run is *in
  progress* is normal (ADR 0002), not a stall — leave it alone. Recover only when
  CI has *finished* and no one took the next step (mark-ready, mention reviewer,
  fix red, merge).
- **Graduated recovery:** detect → mention the one blocked agent → re-mention
  next interval → escalate up the chain.
- **Escalation chain:** member → team leader → Steward → the operator. The buck
  stops at the operator (human): if the Steward goes quiet, the absence of its
  updates is the operator's signal. Watchdogs are the only schedulers (§1);
  everyone else is event-driven.

## 14. GitHub signals arrive via mootup (no GitHub notifications)

Agents receive **no GitHub notifications.** GitHub is the system of record for
code and review, but every *actionable* GitHub event reaches you as a **mootup
message that mentions you** — opened/ready PRs, requested reviews, change
requests, approvals, merges.

- **Never poll GitHub on a timer** for state. Act when the mirrored mootup message
  mentions you. You *may* fetch one specific PR's detail via your token when a
  message points you at it — that's pull-on-demand, not polling.
- **CI results are a watchdog's job, not a worker's.** GitHub doesn't push CI
  outcomes to agents either. The `ken-ci` bridge mirrors `check_suite` results
  into mootup (red → the implementer; green → mark ready); until it exists, the
  **scheduler roles only** (team leader, Integrator) read CI status for *their*
  PRs as part of their *existing* recurring watchdog pass (`gh pr checks`) and
  post the outcome. This is not a new timer and not a worker activity — after you
  push, you stop, and you learn a red result from a mention.
- **If you took a GitHub action that hands the next move to someone, mirror it
  into the right space mentioning them** — request changes → mention the
  implementer; approve → mention the Integrator; merge → mention affected leaders.
  The `ken-ci` bridge automates this when present; until then the acting agent
  posts it, or the move is silently lost.
- The full event→message map (what, where, mentioning whom, posted by whom) is in
  `../docs/program/04-git-and-integration.md §4`.
