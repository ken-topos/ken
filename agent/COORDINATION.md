# Ken coordination law (read by every agent)

Cross-cutting rules for every Ken agent, regardless of role. Role-specific
discipline is in `playbooks/`. The git/PR model is in
`../05-git-and-integration.md`. These rules are adapted from hard-won convo
team lessons; each exists because skipping it caused a real stall or a real bug.

## 1. Event-driven, never poll

After you finish a unit of work or hand off, **post, set status, and stop.** Do
not `/loop`, self-wake, or poll for replies. The notification system already
delivers what you need; polling burns tokens for zero value. A missing
notification is a *stall* — and catching stalls is the team leader's watchdog
job, not yours. (Only team leaders and the Integrator run schedulers.)

## 2. Mention discipline

**Mention an agent iff the next move in the workflow is theirs.** Every mention
costs the recipient tokens and fires a notification.

- Handoff that passes work to B → mention **B only**.
- "Your request is done," nothing pending → mention **nobody**.
- **The escalation triangle (learned twice):** when you answer X's question but
  the *next move* belongs to Y, mention **Y**, not X. Naming Y in prose without a
  real mention is the classic silent stall.

## 3. Status = what you're doing, in your own words

There are three liveness signals; only the third is yours to post:
1. connection (automatic — never post "I'm online");
2. activity (file/transcript mtime — automatic);
3. **semantic status** — "drafting K2 conversion", "blocked on OQ-content-store".
   Agent-composed, never auto-classified. Update it on: receiving a handoff,
   completing one, changing focus, and becoming idle or blocked.

## 4. Threads are the spine

One convo thread per work item; the kickoff message *is* the spine. All handoffs,
questions, status, and retros for that item are **replies in that thread** — a
top-level post fragments the work. After any context reset/compaction, resolve
the live thread from fresh context; do **not** reuse a thread/event ID from a
summarized memory (it may be stale).

## 5. Decisions are for judgment, not deduction

Open a convo Decision (`propose_decision`) for choices with tradeoffs where a
reasonable peer might choose differently — kernel/semantics design, an API shape,
a content-store policy. Do **not** open one for deductive/mechanical choices (a
bug fix is not a decision). Decisions are how future agents query *why* Ken is the
way it is. PR-merge approvals are also Decisions (see the integrator playbook).

## 6. Resolve when structurally determined; escalate only real forks

Before escalating a question, ask: *is there a strategic choice between materially
different futures?* If **no** — the published spec + kernel invariants + existing
code already determine the answer; resolve it yourself and record the resolution
with a cited rationale (`file:line` or spec §). If **yes** — escalate. For
clean-room questions, "the published spec" means `/spec`, never prototype source;
escalate genuine spec ambiguity to Team Spec.

## 7. Ground every premise before locking

Before locking a spec, ADR, or design claim, verify each premise against reality:
"X exists" → grep for it; "matches pattern Y" → read Y end-to-end. Especially for
a *verified* language: a spec claim about the kernel must be checked against the
kernel, not assumed.

## 8. Message-type taxonomy (routing metadata)

Tag each message with a type; the **first line is the thread title** — do not put
a `[TYPE]` prefix in the body. Types: `kickoff`, `question`, `pr_ready` (points at
a GitHub PR), `review_request`, `blocked`, `bug`, `status_update`, `retro`,
`decision`.

## 9. Topology is invariant

Who PRs to whom, who reviews, and who merges is **operator-owned and fixed** (see
`../05-git-and-integration.md`). Agents may improve *what they do inside a node*,
never *add a communication edge or a review cycle* between nodes. When integrating
a retro lesson, reject any carry-forward that would add/move an edge — and do not
soften the rejection to "candidate, watch one more run." That softening is exactly
how coordination entropy creeps in.

## 10. Knowledge promotion: retro → synthesis → memory audit

- After each shipped work item, leave a one-or-two-bullet **retro** in its thread.
- Periodically, synthesize retros into durable docs/skills.
- A lesson is promoted into a durable skill **only** when it passes all three:
  **(a) validated across ≥3 runs, (b) effort-/operator-agnostic, (c) a normative
  rule, not a one-off fact.** Exception: an explicit operator correction promotes
  on a single data point. On promotion, retire the source note atomically.

This rubric is why this file is short: lessons earn their way in.
