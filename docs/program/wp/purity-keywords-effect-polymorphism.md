# SURF-1 — Purity keywords (`fn`/`proc`) + effect-row polymorphism

**Owner:** Spec enclave (elaboration) → Language build.
**Branch:** `wp/purity-keywords-effect-polymorphism` (off `origin/main`).
**Status:** Steward frame (this doc). Not yet elaborated.
**Sequence:** after CAT-1 (in flight), **before CAT-2** — CAT-2/Traversable is
the first consumer of effect-row polymorphism, so this must land first.
**Kernel:** untouched by construction (effects are a surface + elaborator +
checker discipline; the kernel has no effect notion — `spec/30-surface/36 §2`).

This frame pins the **settled operator decisions** (Pat, 2026-07-04) as fixed
inputs and **routes the mechanism questions to the Architect**. It is a frame —
objective, scope, acceptance, do-not-reopen — not the full spec. The enclave
elaborates it to team-ready rigor (§2c). Treat any "current state" line as
perishable: **verify against the landed code at pickup, not this doc.**

## 1. Objective

Replace Ken's single definition keyword `view` with a **two-keyword split that
agrees, by a checked bidirectional rule, with a definition's static purity**, and
**pin effect-row polymorphism** (the row *variable*) in the spec so the split is
well-defined for higher-order effectful definitions. Convert the `.ken` surface
to Unicode symbols in the same pass (BL3). One coherent surface-ergonomics WP.

The keyword must be a **reliable signal**: reading `fn` guarantees "this is an
unconditionally pure mathematical function"; `proc` warns "at least potentially
impure/imperative." Purity becomes a *checked declaration at the definition
site*, not a convention.

## 2. Settled decisions — FIXED INPUTS, do not reopen

These are operator rulings (Pat, 2026-07-04). The enclave elaborates *how*, not
*whether*.

1. **The split is at STATIC PURITY, across three keywords.**
   - **`const`** ⟺ a **zero-parameter pure** definition — a pure value. By
     referential transparency a nullary pure "function" always yields the same
     value, so it *is* a constant; naming it `const` is the honest signal.
     (Subsumes the pure `let`/value definition — `33 §1`.)
   - **`fn`** ⟺ a **pure function with ≥1 parameter**: statically,
     unconditionally pure — effect row is the **closed empty set**, **no row
     variable**. The verification layer may treat it as a mathematical function
     (`36 §1`).
   - **`proc`** ⟺ **everything else that is at least potentially impure /
     imperative**, at **any arity** (incl. nullary effectful like `now () :
     Instant visits [Clock]`): a concrete non-empty row (`[FS]`), an
     **effect-polymorphic** row (contains a variable, e.g. `[e]`), or a
     `space`/imperative op (`becomes`, `36 §4`).
2. **Effect-polymorphic ≠ pure.** A `proc` that *may* instantiate to the empty
   effect at some call site is **not** thereby an `fn`. The keyword classifies
   the **abstraction's guarantee**, never its best-case instantiation. (This is
   the crux that makes the binary split total: the polymorphic case lives
   decisively on the `proc` side.)
3. **The check is bidirectional and enforced** (the "strong version"): the
   keyword is verified against **both the signature and the body / transitively
   inferred effects**.
   - `fn` on a definition that performs or transitively infers **any** effect
     (concrete or via a row variable) is a **type error** (a false purity
     claim).
   - A definition whose effects are provably the closed-empty row **must** be
     `fn` (a `proc` there is a mismatch — see §5 for whether hard error vs lint).
   - No silent disagreement: the signal is only reliable if it cannot lie.
4. **Keyword spellings are `const`, `fn`, `proc`.** (`fn` over `func` for visual
   distinctness from `proc` — they share no shape; `const` for the zero-param pure
   value.) These spellings are fixed; do not propose alternates.
5. **`view` is retired.** Its roles carry over: a **zero-param pure** value is a
   `const`; a nullary def that performs effects at init is a `proc`; an operator
   is an `fn`/`proc` (or `const` if nullary-pure) with a symbolic name (`33 §6`).
6. **Kernel-untouched.** No `ken-kernel` diff, no new `Term`/`Decl`. Effects and
   rows are outer-ring (`36 §2`, `OQ-8` DECIDED); this is surface grammar +
   elaborator + effect-checker only. `OQ-8`/`OQ-8a`/`OQ-9`/`OQ-Space` decisions
   are **not** reopened.

## 3. Mandated deliverables

Each ends in a concrete implementable choice for the enclave to ground; the
Architect owns the two starred mechanism questions.

### D1 ★ — Pin effect-row polymorphism (the technical core)

Today `36`/`OQ-8` pin only **concrete** rows (`visits [FS]`). The **row
variable** an effect-polymorphic `proc` needs is *implied by the model* (the
denotation is a pure interaction tree indexed by the row; Koka rows are the cited
precedent) but is **not specified**. Pin it:

- **Surface + type syntax** for a row variable and (if adopted) an open row —
  e.g. `proc traverse (f : a -> Eff e b) (xs : List a) : Eff e (List b)
  visits [e]`. Decide the concrete spelling (a bare row var `[e]`, an open-row
  tail `[FS | e]`, both) and where row variables bind (implicit param, like
  type/level params `39`).
- **Inference + checking:** lift `36 §1`'s transitive-closure rule to a *row
  variable* — a definition's row is a type-level function of its arguments' rows;
  every concrete call site instantiates the variable to a closed row that the
  checker resolves. **Result required:** for any concrete instantiation the
  effect set is statically closed (no runtime effect discovery).
- **Interaction with the two locked features:** tail-resumptive-only handlers
  (`36 §5`) and capability tokens (`36 §3`) under a polymorphic row — confirm a
  handler folding a *subset* of `e` leaves the rest polymorphic (row-polymorphic
  handling) without breaking the totality / single-consumption WP story.
- **Register:** this is `OQ-8`-adjacent; record the extension in `36` and the
  OQ register (`spec/90-open-decisions.md`) as the row-polymorphism pin, citing
  `OQ-8` as its parent. State whether it is a pure spec addition or needs an OQ.

**Architect grounds the mechanism on landed code** (the effect-checker /
elaborator effect-inference path) before it is written; spec-author pins it into
`36` (+ `33` where declarations reference rows).

### D2 — `fn`/`proc` grammar + the bidirectional purity check

- **Grammar** (`32`, `33 §1`): `const`/`fn`/`proc` replace `view`; the
  `visits [row]` clause is legal on `proc` and (vacuously) illegal-non-empty on
  `fn`/`const`. `const` is the **zero-param pure** form (subsumes pure
  `let`/values, §2.5); a nullary *pure* def is `const`, not `fn`.
- **Checker:** implement §2.3 both directions on top of D1's row inference —
  `const` requires **zero params + provably-closed-empty** row; `fn` requires
  **≥1 param + provably-closed-empty** row (no variable); `proc` requires the row
  be non-empty **or** contain a variable **or** be a space op (any arity).
- **Error surface:** a distinct, legible diagnostic for each direction
  (`fn`-declares-pure-but-performs-`E`; and the `proc`-should-be-`fn` mismatch
  per §5). These are the discriminating conformance cases.

### D3 — Unicode surface (BL3)

Convert the `.ken` surface convention to Unicode symbols (`→`, `λ`, `∀`, `Σ`,
`Ω`, `⊑`, …) rather than ASCII digraphs. **Scope question for the enclave:** is
this a **lexer** change (accept Unicode as primary, ASCII as accepted alias) or a
formatting convention? Decide, and state whether ASCII spellings remain accepted
(recommended: accept both, emit Unicode). Coordinates with `31-lexical`.

### D4 — Migration

Migrate every existing `.ken` (prelude, `packages/*` incl. CAT-1's
`lawful-classes`/`lawful-functors`, `examples/rosetta/*`) and doc snippet from
`view` → `fn`/`proc`, **classified by the checker's own purity inference** (so
the migration is mechanical + checked, not hand-judged). Land together in one
workspace-green unit.

## 4. Acceptance criteria (testable)

1. **`fn` is a reliable purity signal (both directions).** An `fn` whose body
   performs a declared or **transitively-inferred** effect is a compile error
   (discriminating test per effect source: direct `perform`, a called `proc`, a
   space op). A provably-pure body under `proc` is flagged per §5.
1a. **`const` vs `fn` by arity.** A zero-param pure definition **must** be
   `const` (an `fn` there is flagged per §5); a ≥1-param pure definition **must**
   be `fn`. Include the implicit-param edge chosen in §5 (e.g. `const nil {A} :
   List A`).
2. **`proc` covers the polymorphic case.** An effect-polymorphic definition
   (`traverse`-shape) **must** be `proc` and is rejected as `fn`, even though it
   type-checks and runs **pure** when instantiated at the empty row — include
   that exact round-trip: `proc traverse` + a *pure* callback instantiates to a
   statically-pure, effect-free *call*, yet the definition stays `proc`.
3. **Static closure at instantiation.** Every concrete instantiation of a
   row-polymorphic `proc` has a statically-resolved closed effect set (no runtime
   effect discovery) — a test that a mis-declared caller row is rejected
   statically.
4. **Verification-layer treatment.** A pure-typed `fn` is provably effect-free
   and its `ensures` are value-level (the `36 §6` acceptance, restated for `fn`).
5. **Kernel-untouched.** `git diff origin/main -- crates/ken-kernel/` empty;
   `trusted_base` byte-unchanged; no new `Term`/`Decl`.
6. **Migration green.** `cargo test --workspace` green; the rosetta corpus still
   passes (16/0); no `.ken` retains `view`.
7. **Unicode parses.** A Unicode-surface `.ken` elaborates identically to its
   ASCII twin (if both accepted) or the corpus is converted and green.

## 5. Open sub-decisions for the enclave (bounded)

- **Mismatch severity:** a `proc` whose row is provably closed-empty (should be
  `fn`/`const`), or an `fn` that is actually zero-param (should be `const`) —
  hard error vs. lint. Recommend **hard error** for a reliable bidirectional
  signal (matches §2.3). Decide and pin.
- **Does "zero parameter" count implicit type/level params?** A polymorphic
  constant like `nil {A : Type} : List A` has an implicit param but is morally a
  constant *family*. Decide whether `const` = zero **explicit value** params
  (implicit type/level params allowed — recommended, `nil` is a `const`) or truly
  zero binders. Ground on the `39` implicit-param machinery.
- **Row-variable spelling and open-row tails** (D1) — Architect's call, grounded.
- **Unicode as lexer vs. convention; ASCII aliases retained?** (D3.)

Anything **beyond** this scope — a kernel touch, a new `Term`/`Decl`, reopening
`OQ-8`/`OQ-9`/`OQ-Space`, or changing the `fn`/`proc` classification rule —
**re-forks to Steward** before proceeding.

## 6. Do-not-reopen guardrails

- The **classification rule** (§2.1–§2.3) is fixed. Do not relitigate the
  `fn`/`proc` boundary, the spellings, or the bidirectional check.
- **Purity is unconditional** for `fn`; conditional/polymorphic purity is a
  `proc`. This is the whole point — do not "optimize" a polymorphic definition
  into `fn` because it *can* be pure.
- Effects stay **outer-ring**; the kernel's pure denotation (`36 §2`) is
  unchanged.

## 7. Dependencies & sequencing

- **Depends on:** nothing hard; builds on landed effect rows (`36`, L5) and the
  landed `class`/`view` machinery (`33`).
- **Blocks:** **CAT-2** (Applicative/Monad/**Traversable**) — Traversable's
  `traverse` is the first effect-polymorphic surface definition, so D1 must land
  first. CAT-1 (in flight) is unaffected and is migrated by D4.
- **Conformance:** `spec/conformance/surface/` (declarations + effects) — the
  discriminating `fn`/`proc` and row-polymorphism cases above.
