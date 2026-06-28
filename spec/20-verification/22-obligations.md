# Obligation generation

> Status: **DRAFT v0**. Normative for what obligations are and how they arise;
> the extraction algorithm is specified to the level the Verify team needs.
> Contract for WS-V **V2**. Turns a spec'd program into a set of **proof
> obligations** — propositions in Ω, each in its local hypothesis context — that
> the prover (`23`) discharges and the kernel re-checks.

## 1. What an obligation is

An **obligation** is a triple

```
  ⟨ id , Γ ⊢ φ , provenance ⟩
```

- `id` — a stable identifier (for the protocol, `25`).
- `Γ ⊢ φ` — a **goal proposition** `φ : Ω` (`../10-kernel/12 §5`) in a **local
  context** `Γ` of hypotheses in scope at the point it arose.
- `provenance` — where it came from: the source span and the spec clause
  (`requires`/`ensures`/refinement/`prove`) responsible, used by diagnostics.

Discharging an obligation means producing a term `p` with `Γ ⊢ p : φ`, which the
kernel re-checks (`../10-kernel/18 §4`). **An obligation is exactly a typed
hole** of type `φ` in context `Γ` (`24 §2`): obligation generation *is* the
process of finding the holes elaboration leaves where a proof is required, and
proving = hole-filling. This unification is deliberate — partial verification
(`21 §5`) is just leaving some holes unfilled.

## 2. Where obligations come from

Four sources, all arising during elaboration (`../30-surface/39-elaboration.md`)
of the spec encoding (`21 §6`):

1. **Refinement introduction** — using `a : A` where `{ x : A | φ x }` is
   expected emits `Γ ⊢ φ a` (the value really satisfies the refinement). The
   reverse direction (`{x:A|φ} ≤ A`) is free.
2. **Postcondition** — checking a function body `b` against `ensures ψ` emits
   `Γ, params ⊢ ψ[b / result]` (the body establishes the postcondition). For a
   refined *result type* this is the §1 refinement obligation on the body.
3. **Precondition discharge at call sites** — calling `f` whose parameter
   requires `φ` emits, at the call, `Γ_call ⊢ φ[args]` (the caller meets the
   precondition). Inside `f`'s body, `φ` is instead an *assumption* in Γ.
4. **Partial-primitive application** — a bare fixed-width `+`/`-`/`*` or an
   unrefined `/` (or `%`) on `Int` emits a no-overflow / non-zero obligation at
   the operation site (`../30-surface/35-numbers.md §3`,
   `../40-runtime/43-termination.md §2`), per the OQ-1a partial-primitive
   discipline.

Standalone `prove name : φ` (`21 §3`) is the degenerate case: one obligation `·
⊢ φ` (or `Γ ⊢ φ` under its binders) with no body.

## 3. Hypothesis accumulation (the context Γ)

The power of the obligations comes from what is *assumed* in Γ at each point.
The extractor accumulates hypotheses as it walks the body:

- **Preconditions** `requires φ` enter Γ at the top of the body.
- **`let x := e`** adds `x` and, where `e`'s type is informative, the equation
  `x == e` (so later obligations can use the binding).
- **`match`/case split** adds, in each branch, the **equation identifying the
  scrutinee with that constructor** (`s == cₖ …`) and binds the constructor's
  fields. This is what makes case-analysis proofs go through: in the `nil`
  branch you may assume `xs == nil`.
- **Refined parameters** `{x:A|φ}` contribute `φ x` to Γ.
- **Conditionals** `if c then … else …` add `c == true` / `c == false` to the
  respective branches.

Each obligation is therefore discharged under *exactly* the facts that hold on
its path — path-sensitive, like refinement-type / verification-condition
systems, but with the hypotheses being **kernel propositions**, not an external
logic.

## 4. Body-as-motive (verifying recursive and dependent functions)

For a function whose correctness is *inductive* — a recursive `view`, or one
whose result type depends on a recursive argument — the obligation's structure
follows the **body as the motive**:

- The function elaborates to an application of the relevant **eliminator**
  (`../10-kernel/14 §3`) whose **motive** `M` is the (refined) result type as a
  function of the recursed argument.
- Each constructor branch of the eliminator yields an obligation **with the
  induction hypothesis in Γ**: when verifying the `suc n` / `cons x xs` branch,
  Γ contains the motive already established for `n` / `xs` (i.e. "the
  postcondition holds for the recursive call"). This is precisely structural
  induction, surfaced automatically from the function's own recursion.
- Non-recursive functions are the degenerate motive (no induction hypotheses);
  the same machinery covers both.

So "prove this recursive function meets its spec" becomes "discharge the
per-constructor obligations, each with the recursive-call's spec as a
hypothesis" — generated mechanically, no manual induction principle stated by
the user.

## 5. The extraction algorithm (sketch)

```
obligations(Γ, term, expectedType):
  walk the elaborated term;
  at each subterm whose *expected* type is a refinement/contract/dependent goal
  not discharged by plain type-checking, emit ⟨fresh id, Γ_here ⊢ φ, prov⟩;
  extend Γ at binders/splits per §3;
  at eliminators, set the motive and add induction hypotheses per §4;
  return the collected obligations.
```

- Obligations are **independent**: each is a self-contained `Γ ⊢ φ`. They may be
  proved in any order / in parallel (the agent-team and the prover both exploit
  this).
- The extractor is **untrusted**: a missing or malformed obligation cannot cause
  unsoundness, because the *kernel* still type-checks the elaborated program
  with its proof terms (holes included). A bug here causes a spurious failure or
  a missed check surfaced as an unfilled hole, never a false `proved`.
- **Completeness target:** every refinement/contract use generates the
  obligations whose discharge (plus kernel checking) suffices for the spec to
  hold. This is the V2 correctness criterion, exercised by conformance.

## 6. Output and hand-off

Obligation generation produces, per definition, the ordered obligation set with
contexts and provenance. This set is the input to the **classifier/prover**
(`23`) and, on failure, to **diagnostics** (`24`); its serialization is part of
the **protocol** (`25`). A definition with an empty obligation set (or all
discharged) is fully verified; one with open obligations is *partially* verified
(`21 §5`) and carries typed holes.

## 7. What WS-V must deliver here (V2)

The extractor: refinement/postcondition/precondition obligation emission (§2),
path-sensitive hypothesis accumulation (§3), body-as-motive induction plumbing
(§4), stable ids + provenance, and the partial-verification hole representation.
Acceptance: for a recursive function with an inductive postcondition, the
generated obligations + supplied proofs check in the kernel; removing a needed
proof leaves a precisely-located open hole. Conformance:
`../../conformance/verify/obligations/`.
