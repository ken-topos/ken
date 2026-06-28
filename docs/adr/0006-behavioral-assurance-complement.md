# ADR 0006 — Behavioral-assurance complement (the sibling)

- **Status:** Accepted; sibling named **Ward**
- **Date:** 2026-06-27
- **Deciders:** the operator

## Context

Ken proves things **propositionally** about a closed, deterministic, total world
— its kernel is a small dependent type theory with observational equality (ADR
0005) and a proof checker. Cast as weakest-precondition writ large, a Ken
artifact says: *given assumptions `P`, conclusions `Q` hold, mechanically
certified.*

The things that **escape** that net are exactly those that cannot be closed as a
proposition over a pure function: **time and ordering** (liveness, fairness,
eventual consistency), **concurrency and distribution** (interleavings, partial
failure, message loss), **the environment** (the `P` the proof assumed), **non-
determinism / probability**, and — for an agentic language especially —
**outputs with no propositional oracle** (was the agent's action "good"?). For
these the honest move is not "prove" but **model, then test, then monitor**.

A scoped research review (operator + web-research agent,
`local/Post-Implementation Verification and Ken Language.md`) establishes the
principled core: the world of **behaviors-over-time is itself a topos**
(Spivak–Schultz, *Temporal Type Theory*) — behavior types are sheaves over time.
So Ken occupies the **static, propositional, intuitionistic** fragment of a
topos-internal logic, and its downstream complement occupies the
**temporal/modal** fragment of *the same* logic. The complement is not a foreign
bolt-on; it is the temporal extension of Ken's own semantics. The seam between
them is **Ken's statement of what it assumed**.

## Decision

**Ken's behavioral-assurance complement is a tightly-coupled but *separate*
sibling project — **`Ward`** — joined to Ken by an `assumption-boundary` export.
One logic, two engines.**

1. **One logic, one source of truth, two engines.** The *assertion language and
   its topos semantics* are designed as a **single whole**: a property means the
   same thing whether Ken proves it or the sibling monitors it. The contract is
   written **once, in Ken**, and the four-way epistemic status (below) is
   visible in that one source. But the **discharge engines are two**, with two
   trust domains and two runtimes — Ken (static, inside the small Rust TCB,
   authoring/CI time) and the sibling (temporal/behavioral, classical
   model-checkers + monitors, often production runtime, **outside** the kernel
   TCB). The seam is an *internal handoff within one logic*, not a foreign API.
2. **The sibling owns what cannot be proved.** Model checking (the TLA+/Quint+
   Apalache slot), spec-driven test generation (where Ken's dependent/refinement
   types are generators *and* oracles), and runtime verification / trace
   conformance (monitors synthesised from specs; the agentic safety envelope).
   None of this enters Ken's kernel.
3. **Four-way epistemic status at Ken's surface** (`OQ-spec`,
   `20-verification/21 §5`): every claim is **proved** (kernel-discharged),
   **tested** (assumed + runtime/test obligation), **delegated**
   (temporal/modal, exported), or **unknown** (typed hole). The source
   distinguishes proof from test from model from gap on its face.
4. **Temporal/modal obligations are stated as *data*, not kernel modalities.**
   Ken can *state* a temporal property as an ordinary **deeply-embedded**
   inductive value (an LTL / μ-calculus `Temporal` datatype), with surface
   notation elaborating to it — **zero kernel change**, TCB untouched (tier-1,
   ADR 0001/0004). It is then **exported**, not discharged in Ken. (This is the
   small-kernel-preserving form of an in-language modal layer: state-as-data,
   not `▷`/clock modalities in the trusted core.)
5. **The seam is the assumption-boundary export IR.** A stable artifact emitted
   by the elaborator carrying: discharged invariants `Q` (the sibling may
   *assume*, not re-prove → smaller state space); open assumptions `P` (the
   nondeterministic environment / generator input domain); refinement predicates
   (well-formed generators **and** oracles); effect/interaction signatures (the
   event alphabet of the behavioral state machine / the monitor's alphabet); and
   stated temporal obligations (LTL/μ-calculus to model-check and to monitor).
   Trace half **ITF-compatible** for direct Quint/Apalache interop.
6. **Ken specs *its half* of the seam** in `spec/70-behavioral/` — what Ken
   exports, how delegated/tested obligations are stated and made visible, and
   the conformance hooks. The **sibling's internals** (Quint emitter, generator
   deriver, monitor synthesiser) live in the sibling's own project/spec.

## Consequences

- **Ken stays small and static** — the tier-1 small-auditable-TCB principle is
  preserved; the classical/temporal model-checking machinery never enters the
  kernel. "Designed as a whole" means *one logic*, never *one binary*.
- **The two-artifact tax is minimised.** The sibling's model is **derived from
  Ken's export**, not hand-written in a parallel language, so it cannot silently
  drift; **trace conformance** (does the implementation refine the model?) keeps
  it honest continuously.
- **`OQ-spec` is resolved** for the proof interface and the epistemic taxonomy;
  the **state model / `old`** remains deferred to `OQ-Space` (lean
  explicit-state).
- **New open questions** (the live design work, register §B/§H): the export-IR
  schema; the intuitionistic↔classical bridge and whether the refinement mapping
  is itself a Ken-checked artifact; ITF vs a Ken-native trace format;
  conformance as CI gate vs production monitor vs both; the agentic-oracle
  policy (metamorphic relations + RV watchdogs; the safety FSM the agent is a
  nondeterministic oracle inside).

  > **Update (2026-06-27):** these follow-on questions are now **DECIDED**
  > (kept for the record) — `OQ-export-ir`, `OQ-classical-bridge`,
  > `OQ-conformance`, `OQ-agentic-oracle`, and `OQ-Space` (the `old`/state
  > model). See the resolution log in `../../spec/90-open-decisions.md` and
  > `../../spec/70-behavioral/`. Only `OQ-sampling-policy` and
  > `OQ-discharge-attestation` remain deferred (Ward-blocked).
- **Naming.** The sibling is **`Ward`** (operator, 2026-06-27) — "to keep watch
  / guard": it watches what proof cannot reach, and is the agentic safety
  envelope. Pairs with Ken ("the range of what one can know"): Ward handles what
  is beyond Ken's ken.

## Revisit if

- The in-language modal layer needs to be more than *data* (e.g. Ken must itself
  *reason* about temporal properties, not just state/export them) — that would
  reopen the kernel-cost question (guarded/`▷` type theory) and must be weighed
  against the small-TCB principle.
- A single combined artifact/runtime is ever shown to be necessary — it would
  have to justify re-importing model-checking into Ken's trust domain.

## References (in `local/`, read-not-copy)

Schultz & Spivak, *Temporal Type Theory* (the topos/sheaf foundation); Quint +
Apalache (typed/effectful TLA, ITF traces); QuickChick / Luck / Foundational PBT
(generators from dependent types); guarded DTT + *Temporal Refinements for
Guarded Recursive Types* (in-language `▷`/clock, LTL/CTL via μ-calculus — the
route Ken deliberately keeps as *data* rather than kernel modalities);
Linux-kernel RV, PyMOP (monitor synthesis); AgentVerify, *Watchdogs and Oracles*
(agentic RV).
