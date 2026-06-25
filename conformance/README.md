# Conformance corpus

The executable, black-box behavioral tests that define "correct Ken." Each case
pins a specific spec section (`../spec/`) and states an **input → expected
behavior** that any conforming implementation must satisfy. Conformance is the
CI gate every team's work passes (`../docs/program/04-git-and-integration.md`);
it is also how the Spec enclave cross-checks the spec against the prototype
**oracle** without copying source (`../CLEAN-ROOM.md`).

## Layout

```
conformance/
  kernel/        — 10-kernel/ behaviors (universes, pi-sigma, inductive,
                   identity, cubical, conversion, judgments)
  verify/        — 20-verification/ (spec-syntax, obligations, prover,
                   diagnostics, protocol)
  surface/       — 30-surface/ (lexical, grammar, declarations, data-match,
                   numbers, effects, collections, ffi-io, elaboration)
  runtime/       — 40-runtime/ (values, evaluation, termination, capacity)
  stdlib/        — 50-stdlib/ (lawful instances, verified building blocks)
```

## Case format

Until the surface syntax and harness are fixed, cases are written as
**structured markdown** (or a small JSON/TOML once the harness lands,
OQ-harness). Each case:

```
## <case-id>          e.g. kernel/universes/type-in-type-rejected
- spec: <section>     e.g. spec/10-kernel/12-universes.md §1
- given: <input>      a core term / surface program / obligation
- expect: <behavior>  accepts | rejects(reason) | reduces-to(v) | proved
                      | disproved(countermodel) | incomplete(hole) | error(kind)
- why: <one line>     the property this pins (often a non-reproduction of a
                      prototype gap, or a soundness commitment)
```

Cases tagged **(oracle)** are to be confirmed against the prototype's observed
behavior by the Spec enclave; cases tagged **(soundness)** encode a kernel
soundness commitment (`../spec/10-kernel/README.md §5`) and must never regress.

## Seeds

This directory is seeded with representative cases per area (the files below)
that establish the format and pin the **load-bearing non-reproductions** of the
prototype's gaps. The build teams grow the corpus as they implement; a spec
claim with no conformance case is a claim no one can rely on
(`../spec/00-overview.md`).

- `kernel/seed-kernel.md` — `Type:Type` rejection, dependent Σ, `J` on
  non-`refl`, SCT accept/reject (the four kernel commitments most worth pinning
  first).
- `verify/seed-verify.md` — a proved postcondition, a disproved one with a
  countermodel, an incomplete one with a hole, and the soundness regression (Z3
  cannot force a false `proved`).
- `surface/seed-surface.md` — `2 : Int` vs `2.0 : Float`; sum-type
  construct-then-eliminate computes; `match` exhaustiveness failure.
- `runtime/seed-runtime.md` — dedup + O(1) equality; `Int` past 2⁵³ exact;
  `unknown` propagation.
