# WP X1 — Reference interpreter (core terms → values, strict CBV + sharing)

> **Status:** Steward frame — **awaiting spec-leader elaboration** (queued in the
> spec chain behind L5). spec-leader elaborates `spec/40-runtime/42-evaluation.md`
> (DRAFT v0 → implementation-ready, **pure-core scope**) to rigor, then the
> **Runtime team** builds.
>
> **Team:** Runtime · **Deps:** K1 (done), K3 (done) · **Size:** M · **Risk:** ★★
> · **Parallel** with V0/L-classes. ► **Feeds G1** (the vertical slice) together
> with V0 — V0 elaborates surface→core, X1 runs the core. The **oracle** every
> later backend (X3 native) is judged against.

## Objective

The **reference interpreter**: evaluate **core terms** (`spec/10-kernel/11`, the
elaborator's output) to **values** (`spec/40-runtime/41`, the K3 content-addressed
model), realizing exactly the kernel's reductions, **deterministically**, with
**canonicity** for closed ground computations. This *defines the meaning of a Ken
program* and is the differential oracle: on any disagreement with a later backend,
the interpreter is right by definition (`42 §5`).

## The framing that sets the risk level

X1 is **not in the TCB for type soundness** — it evaluates already-kernel-checked
core terms; it does not decide typing. But it **is** the semantic reference: a
bug here is a wrong *answer*, silently propagated to every backend validated
against it. So ★★ (a notch below the kernel): correctness is by **agreement with
the kernel's own reductions** (`§1` below) and by the canonicity/determinism
conformance corpus, not by a separate trust argument. The kernel already computes
these reductions for conversion (lazy WHNF / NbE); X1 is "the kernel's evaluator,
run to completion, to full values, with sharing."

## Scope

**IN (the G1 pure core):**
- Evaluate closed/ground **core terms** to **values** realizing the kernel
  reductions of `17 §1`: **β**, **Σ-projection**, **ι** (eliminators on
  constructors), **δ** (definition unfolding), the **observational** computations
  (`Eq`-by-type, `cast`, quotient elim — `16 §9`), and **prim** (the audited
  primitive reductions, `14 §5`).
- **Strict CBV with sharing** (`42 §2`, operator-decided): arguments + `let`
  eager, left-to-right; results shared via the K3 content-addressed heap (equal
  subcomputations deduplicated). `if`/`match` evaluate only the taken arm;
  `&&`/`||` short-circuit.
- **Determinism** (same closed term → same value) and **canonicity** (a closed
  inductive evaluates to a constructor form; closed `Eq`/`cast` compute; no
  well-typed ground program gets stuck).
- **`unknown` propagation** (`42 §4`, `41 §6`): a term depending on an open
  verification hole yields `unknown` by the Kleene/Heyting rules; a hole-free
  program never yields `unknown`.
- The `ken-interp` crate + the G1 end-to-end wiring (V0 elaborates → X1 runs).

**OUT — other WPs, do not build here:**
- **Effects** (`FS`/`Net`/`Clock`/`Console`/`Rand`, `space`/`becomes` mutable
  cells — `42 §3`): their operational meaning depends on **`36-effects`**, which
  **L5** is elaborating now. X1's effect evaluation is a **follow-on** once L5
  lands — the G1 slice is the pure, effect-free fragment.
- The **native backend** (X3) and the **differential corpus** runner (X4).
- The **REPL** / incremental re-checking polish (strategy T2; later).
- The `Lazy a` thunk type's full surface story (lazy-by-annotation) — wire the
  evaluation primitive if `41`/`42` pin it; otherwise defer with the effects.

## The elaboration this needs (spec-leader → spec-author)

`42-evaluation.md` is normative for *what evaluation computes* but DRAFT and not
yet at builder rigor. Elaborate, **pure-core scope**:
1. **The evaluation algorithm** — how CBV-with-sharing drives a core term to a
   value against the K3 heap: the value/closure representation reused from `41`,
   the eval/apply structure, where sharing/dedup is consulted, the WHNF-vs-full
   boundary. Reconcile against the kernel's reduction (`17 §1`) so results agree.
2. **Canonicity + determinism** stated as testable properties, with the closed
   observational cases (`cast A A refl a ≡ a`, `Eq`-by-type, quotient elim) given
   their evaluated values.
3. **`unknown` propagation rules** pinned (the Kleene/Heyting table from `41 §6`)
   — exactly where a hole turns a result into `unknown`.
4. **Effects explicitly deferred** with a one-line seam note (so the build team
   knows the effect cases are stubbed/stuck pending L5, not forgotten).

Conformance (`conformance/runtime/evaluation/`, with the validator): canonicity
of closed inductive/observational computations, determinism (same term → same
value), short-circuit/branch laziness, and `unknown` propagation. Apply the
**verdict-flip discipline** — e.g. a determinism case must distinguish a correct
shared evaluation from a recompute-divergence, not pass vacuously.

## Acceptance (testable)

1. **Canonicity:** a closed program over an inductive type evaluates to a
   constructor form; the closed observational computations (`cast`-refl, `Eq`-by-
   type, quotient elim) evaluate to their specified values — **no stuck closed
   ground term**.
2. **Determinism + sharing:** the same closed term evaluates to the same value;
   equal subcomputations resolve to the **same** content-addressed heap entry
   (assert dedup, not just equality).
3. **Branch laziness:** `if`/`match`/`&&`/`||` evaluate **only** the taken arm
   (assert the untaken arm's effect/divergence is *not* forced — a structural
   check, since the pure fragment can't diverge).
4. **`unknown`:** a hole-dependent term yields `unknown`; a hole-free term never
   does.
5. **Kernel agreement:** for a corpus of closed terms, the interpreter's value
   matches the kernel's own reduction (the interpreter is WHNF-run-to-completion).
6. **G1 end-to-end:** a surface program elaborated by V0 to core, then run by X1,
   produces the expected value — the vertical slice closes.

## Sequencing

Queued in the spec chain **behind L5**. When L5 elaboration merges, this is the
recommended next enclave WP (it completes the **G1 vertical slice** with the
already-in-build V0). Runtime is idle and ready (K3 shipped). Effects follow once
L5 is on `main`. Build queries: runtime semantics → Spec; design → Architect.
