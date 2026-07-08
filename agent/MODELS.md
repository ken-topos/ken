# Model tiers

Ken runs a tiered fleet (currently T1+T2) to concentrate the most expensive
seat on the
highest-judgment and clean-room-critical work, using a capable
high-throughput model for high-volume code generation and coordination. Every
agent runs an **Anthropic model directly** on the subscription (OAuth); there is
no open-weight tier and no provider proxy.

**Tier vocabulary (canonical — use this, not model names).** Refer to seats by
**tier** everywhere — playbooks, WP frames, the tracker, convo — never by a model
name or a model characteristic (e.g. "N-months behind"). The tier↔model mapping
lives **only in the table below**, so it stays accurate as models change and no
downstream doc goes stale:

- **T1** — highest-judgment class (Opus-class). The clean-room enclave.
- **T2** — high-throughput build/coordination class (Sonnet-class).
- **T3** — lightweight class (Haiku-class). Not currently seated (Ken runs T1+T2).

A new T1/T2/T3 family (e.g. GPT 5.6 → **Sol** / **Luna** / **Terra**) is expected;
when a family swaps in, **only this table changes** — every tier reference
downstream is already correct.

| Tier | Model (current mapping) | Roles | Why |
|---|---|---|---|
| **T1** (enclave) | **Opus-class** (1M, high effort, extended thinking) | Spec-author, Conformance-validator, **Steward**, **Architect** | Highest judgment; the clean-room enclave; design + workflow authority. These calls are worth the most. |
| **T2** (build & coordination) | **Sonnet-class** | Build-team Leaders, Implementers, QA; Spec-team Leader; Librarian | High-volume code generation, coordination, mechanical gates, verification runs, doc observation. |
| **T3** | **Haiku-class** | (none currently) | Lightweight tier; held in reserve. |

The operator is the human product owner; Steward is the primary proxy into the
federation.

### Credit-window failover — GPT/Codex ≈ T2 (operator, 2026-07-04)

The Anthropic subscription credits are finite and the fleet **fails over to a
ChatGPT/Codex backend** when they are exhausted (operator-managed devcontainer
integration). Capability mapping for that backend (operator-stated):

- **GPT 5.5** (current strongest GPT) **≈ Sonnet-class ≈ T2.**
- **Opus-class (T1) is stronger than any current GPT** — there is **no T1 on the
  GPT backend** until the GPT 5.6 family (Sol/Luna/Terra = T1/T2/T3) ships.

**Operating rule while Anthropic/Opus credits remain:** the Opus enclave is the
**irreplaceable capability** — reserve the remaining credits for **T1-critical
work** (clean-room design, spec elaboration, soundness rulings, abstraction-
boundary pinning). **Defer T2 build/execution to the GPT backend** (separate
credit pool). Soundness gates cast on the GPT backend are **provisional** →
**Opus agents re-review the GPT work when the Anthropic subscription refreshes.**
So: elaborate + frame now (T1), build on GPT (T2), re-cert on refresh (T1).

**Steady state after refresh — the split team (operator, 2026-07-04).** Once the
Anthropic subscription refreshes, the fleet runs **split across two independent
credit pools**: **Anthropic supplies ONLY the Opus (T1) enclave**; **everything
else (build teams, coordination) runs on the ChatGPT/Codex pool (T2).** Because
the pools are independent, this enables **wider T2 fan-out AND sustained T1 work
simultaneously** — the Opus pool is never drained by build-team burn, so T1
throughput is maximized while the GPT pool scales the build fleet as wide as it
allows. This is the **ideal steady state for the catalog cadence** (T1 enclave
pins each abstraction boundary → a wide T2 fleet fans out): the topology now
matches the workflow. Phase 3 also folds in the Opus re-review of the Phase-2
GPT-only work.

**The three phases:** (1) **now** — whole fleet on Anthropic (shared pool);
reserve it for T1 enclave elaboration/framing, defer T2 builds. (2) **credits
exhausted → refresh** — whole fleet on GPT (≈T2, no T1); execute the shovel-ready
build queue; gates provisional. (3) **refreshed** — split team (Opus-only on
Anthropic ∥ everything-else on GPT); wide fan-out + sustained T1 + Opus re-review
of Phase-2 work.

### Delegating the mechanical tail — a T1-conservation tactic

Operator-originated (2026-07-04). An expensive role (esp. a **T1 enclave**
seat) may hand the **mechanical tail** of an authoring turn to a **cheaper-tier
(T2/Sonnet) subagent** — but only under a load-bearing criterion and two hard
guardrails. The point is to spend fewer **T1** tokens on work a T2 model does
identically; misapplied, it spends *more*.

- **Criterion — delegate IFF the subagent's verification is a deterministic
  re-check cheaper than the doing.** Delegate: 80-col reflow, ASCII→Unicode
  canonicalization, a fully-specified rename across N files,
  run-tests-and-report — each verified by the checker script / `git diff
  --word-diff`. Keep on T1: initial spec/seed prose, law statements, fork
  transcription, silence-resolution, clean-room judgment, **any semantic
  verification.**
- **Do NOT delegate:** (a) a grounding read you author *against* (the facts
  come back as a summary and you re-fetch anyway — negative ROI); (b) a
  reconcile / "does §X say Y" pass — that reintroduces the exact
  cite-vs-verify hazard `reconcile-don't-cite` exists to prevent. The
  delegation prompt is **mechanical-only, "no rewording"** — a subagent that
  "helpfully" rewords a normative sentence is a fidelity regression in
  fidelity-critical material.
- **GUARDRAIL 1 — the subagent MUST run on the cheaper tier (T2/Sonnet).** A
  `Task`/Agent-tool subagent **inherits the parent session's model by
  default**, so a subagent spawned from a T1 (Opus) seat is **Opus unless you
  pass the Agent tool's `model: sonnet` override.** Delegating *without* that
  override runs the mechanical tail on **T1** — the exact opposite of the
  tactic (it spends *more* of the scarce pool, not less). **Always spawn the
  tail on `model: sonnet`.**
- **GUARDRAIL 2 — right-size it.** The gain is **modest (~10–20% on
  edit-heavy turns, ~0 on design-heavy turns)** and easily **inverted by
  overhead**: a small reflow (one chapter) is usually cheaper done **inline**
  than paying a subagent's spin-up + full-context read. Delegate only when the
  tail is genuinely large and the delegated cost (even on T2) is below the
  inline cost. If in doubt, inline.

## Knobs (tune by observed quality, not up front)

The tier split is fixed (T1 enclave / T2 everyone else); the tunable
knob is **effort**, set per role in `moot.toml`.

- **Kernel and Verify QA** are soundness-adjacent — they are the likeliest
  candidates for a higher effort setting if verification quality lags. Start at
  the team-default effort; raise it on observed misses, not up front.
- **Publisher-path work stays mechanical.** The deep correctness and
  architectural review is the **Architect's** (T1) job on the merge Decision;
  publisher handling should remain scripted/mechanical, not a high-effort model
  task.

## Clean-room × roles (load-bearing)

The clean-room boundary is a **role** discipline, not a property of the model in
the seat (`CLEAN-ROOM.md`; the enclave↔reference-access mapping is incidental —
any model in an enclave seat reads references under the same discipline, any
model in a build/coordination seat reads none):

- Only the **enclave** (Spec-author, Conformance-validator, Architect) may
  consult copyleft references. The build and coordination roles see only `/spec`
  and `/conformance`, which are clean by construction. The AGPLv3 prototype
  (`yon`) is **not mounted** and is not consulted by anyone — there is zero
  AGPLv3 contact, which is strictly cleaner than the alternative.
- **Never send copyleft material to a build or coordination role.** Only the
  behavioral description in Ken's own words (the spec/conformance artifacts) may
  pass to the build tier; never copyleft source text itself. Ken's own MIT
  source is fine to send anywhere.
- **Copyleft references (⚠ GPL/AGPL/CeCILL — `smtcoq`, `spot`, `jif`) are
  enclave-only too.** Only the enclave (Architect / Spec) reads them, for
  *approach and behavior* only, under the leakage recheck (`CLEAN-ROOM.md`); they
  are never sent to a non-enclave seat and never vendored. Permissive refs (Lean,
  Z3, Quint, …) may be read by the enclave but, like all refs, are not a source
  for implementer agents — those build from `/spec`.

## Portability

Playbooks and `COORDINATION.md` are written **model-agnostic** — no reliance on
any single model's idiosyncratic behaviors — because the same coordination law
must hold regardless of which model sits in each seat.

**Routing mechanism.** Each agent is a normal Claude Code process launched by
`moot up`, which reads its `model` + `effort` per role from `moot.toml`
(`[agents.<role>]`) and passes `--model`/`--effort`. Every role — enclave and
build/coordination alike — runs **direct on the Anthropic subscription** (OAuth),
with **no `env` block** and **no proxy**. (A retired `run-llm-proxy.sh` /
`127.0.0.1:8090` once dispatched a non-Anthropic build tier by model prefix; it
is vestigial now that the whole fleet is Anthropic-direct — a stray `env` block
pointing a role at it would break that role's launch.) Operator runbook:
`local/mootup-agent-backends-setup.md`.
