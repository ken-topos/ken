# Behavioral assurance — Ken's downstream seam

> Status: **DRAFT v0**. Normative for *Ken's half of the seam* (what Ken exports
> and how delegated/tested obligations are stated); the **sibling** project that
> consumes it — **`Ward`** (ADR 0006) — is specified separately. This area is
> the surface form of the second half of Ken's mandate: **prove what can be
> proven, and state what must be tested.**

## 1. Why this belongs to Ken (one logic, two engines)

Ken proves things about a **closed, deterministic, total, static** world. The
things that escape that net are exactly those that cannot be closed as a
proposition over a pure function: **time/ordering** (liveness, fairness,
eventual consistency), **concurrency/distribution** (interleavings, partial
failure), **the environment** (the assumptions a proof took for granted),
**nondeterminism / probability**, and **agentic outputs with no propositional
oracle**.

These are not foreign to Ken. The world of **behaviors-over-time is itself a
topos** (Spivak–Schultz, *Temporal Type Theory* — behavior types are sheaves
over time). So:

> Ken occupies the **static, propositional, intuitionistic** fragment of a
> topos-internal logic; its complement occupies the **temporal/modal** fragment
> of *the same* logic — behaviors as sheaves over time, with "always/eventually"
> as modalities — discharging by **model → test → monitor** exactly the
> propositions that cannot be closed as static proofs.

Hence **one logic, two engines** (ADR 0006): one assertion language + one
semantics + one source artifact; two discharge engines with two trust domains —
Ken (static, in the small Rust TCB) and the sibling (temporal/behavioral,
classical model-checkers + monitors, outside the TCB). The seam below is an
*internal handoff within one logic*, not a foreign API.

## 2. The four-way epistemic status

Every specification claim (`../20-verification/21 §5`) is **proved** (kernel-
discharged), **tested** (assumed + runtime/test obligation), **delegated**
(temporal/behavioral, exported), or **unknown** (typed hole). The **assumption
boundary** this area exports is precisely the **tested + delegated + open** set
— what Ken could not guarantee statically, which is the exact specification of
what the sibling must model, test, and monitor.

## 3. The assumption-boundary export (the seam)

Ken emits a stable artifact (the elaborator's behavioral export) carrying:

| Ken static output | Becomes downstream |
|---|---|
| Proven postconditions `Q` | invariants the model may **assume**, not re-prove → smaller state space |
| Open assumptions `P` (undischarged `requires`) | the **nondeterministic environment**; the generator's input domain |
| Refinement / dependent types `{x:A | φ}` | well-formed **generators *and* oracles** (the types *are* the test generators) |
| Effect / interaction signatures (`../30-surface/36`) | the **event alphabet** of the behavioral state machine; the monitor's alphabet |
| Stated temporal obligations (§4) | **LTL / μ-calculus** properties to model-check and to monitor |
| The running implementation | instrumented **traces** validated against the model (trace conformance) |

The schema is **decided** (`OQ-export-ir`, ADR 0006): an **assume-guarantee
contract**, *generated* from verified content (so it cannot overclaim),
**Ken-native for the propositional parts** and **ITF-compatible for traces**
(Apalache/Quint interop). Generators carry **support structure only — never a
sampling measure**, which lives outside Ken source. Full schema in
[`71-assumption-boundary.md`](71-assumption-boundary.md).

## 4. Temporal obligations as *data*, not kernel modalities

Ken **states** a temporal/behavioral property as an ordinary **deeply-embedded
inductive value** — an LTL / μ-calculus `Temporal` datatype — with surface
notation elaborating to it. It is then **exported** (§3), not discharged in Ken.

This is deliberate and load-bearing: it gives Ken a first-class way to *say*
"eventually settled" / "never two leaders" while leaving the **kernel
untouched** (no `▷`/clock modalities in the trusted core — that would bloat the
TCB against ADR 0001/0004). The in-language layer is *expressive enough to state
and export*, not to *prove*; proving is the sibling's job. (If Ken ever needs to
*reason* internally about temporal properties, reopening a guarded/modal kernel
layer is `OQ-temporal` — weighed against the small-TCB principle.)

## 5. The three layers the sibling orchestrates (overview)

Specified in the sibling's own project; summarised here for the seam:

- **L1 — Behavioral modeling & model checking** (the TLA+ slot; **Quint +
  Apalache** are the most Ken-congruent — typed, effect-aware, SMT-backed). Fed
  by `Q` (assumed invariants) and `P` (the environment).
- **L2 — Spec-driven test generation** (QuickChick-style): Ken's refinement/
  dependent types are **generators and oracles**; plus model-based test-case
  generation from L1, and **metamorphic** testing for oracle-free agentic
  outputs.
- **L3 — Runtime verification & trace conformance**: monitors synthesised from
  the exported temporal obligations (LTL → Büchi, MOP-style), the **agentic
  safety envelope** (the LLM/agent as a nondeterministic oracle inside a
  verifiable FSM), and **trace conformance** closing the loop back to L1 (does
  the implementation *refine* the model?).

## 6. What this area must deliver

- The **assumption-boundary export** (§3) emitted by `ken-elaborator` — schema,
  stability guarantees, ITF trace compatibility.
- The **`tested`** and **`delegated`** surface (`../20-verification/21 §5`): the
  `assume`/`test` clause and the `Temporal`-as-data notation (§4), both
  *visible* in source and *exported*.
- **Conformance hooks**: instrumentation points for trace emission (cheap — RV
  overhead is instrumentation-dominated), and the trace format.
- The **open questions** (`../90-open-decisions.md §H`): export-IR schema,
  intuitionistic↔classical bridge, trace-format commitment (ITF vs native),
  conformance as CI-gate vs production-monitor vs both, agentic-oracle policy.

## Chapter map (planned)

| File | Subject |
|---|---|
| `71-assumption-boundary.md` | The export IR: `Q`/`P`/refinements/effects/temporal; schema; ITF traces — **DRAFT** (`OQ-export-ir` decided) |
| `72-temporal.md` | `Temporal` as deeply-embedded data; surface notation; LTL/μ-calculus encoding |
| `73-conformance.md` | Trace emission, instrumentation, trace-conformance (does code refine the model?) |
| `74-agentic.md` | Oracle-free outputs: metamorphic relations + RV watchdogs; the agent safety FSM |

`71` is drafted (the export schema); `72`–`74` are stubs pending the sibling's
bring-up. This README is the binding overview.
