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

## Enclave elaboration (T1, `origin/main @ 24a414b`; frame `@ cb90bcf`)

The frame elaborated to team-ready rigor. Division of labour mirrored CAT-1:
Architect owned the starred mechanism (D1); spec-author transcribed all pins
into `/spec` (Architect fidelity-gates the D1 prose); conformance-validator
authored the discriminating seed. **Kernel-untouched throughout** — every pin is
surface grammar + elaborator + checker over the DECIDED `OQ-8` model; `git
diff origin/main -- crates/ken-kernel/` stays empty, no new `Term`/`Decl` (AC5).

### E1 — Purity split `const`/`fn`/`proc` (D2, spec-author) → `36 §1.6`

The classification is defined over a definition's **declared** purity class
(`36 §1.6.1`): impure (`ρ_decl` non-empty, or contains a row variable, or a
`space` op) ⇒ `proc` at any arity; pure ⇒ `const` (0 explicit value params) or
`fn` (≥1). The split is **total** and **decidable** (all inputs are syntactic or
the terminating row fixpoint). Grammar: `32 §1` (the `decl` production; only
`proc` carries a `visits` clause) + `33 §1`; keywords: `31 §4` (`const`/`fn`/
`proc` **fixed**, not `OQ-syntax`; `view` retired).

- **The bidirectional check reuses the landed escape gate for the hard half**
  (`36 §1.6.2`). Because `fn`/`const` carry **no `visits` clause** (`ρ_decl =
  ∅`), the existing `§1.4` escape check *is* the "`fn` that performs/infers an
  effect is a compile error" direction (AC1) — SURF-1 only re-labels the
  pure-default gate as the purity-signal guarantee. The genuinely **new**
  direction is the reverse: a `proc` provably pure (empty declared row, no
  `space` op) is a should-be-`fn`/`const` mismatch.
- **§5(a) mismatch severity — HARD ERROR** (pinned, `36 §1.6.3`). A lint leaves
  the signal advisory, so a reader could not trust the keyword without
  re-deriving effects — defeating the point. Both directions hard = the
  *consistent* choice, since the `fn`-false-purity direction was already hard.
- **§5(b) "zero parameter" = zero *explicit value* parameters** (pinned, `36
  §1.6.3`). Implicit `{…}` binders — type, level, instance, or row (§1.5) — do
  **not** count: grounded on `39 §2.2`, an implicit is inserted/solved at each
  use site and erased, so a def whose only parameters are implicit is used
  exactly as a constant (`const nil {A} : List A` is written `nil`). `fn` begins
  at the first explicit value parameter.

### E2 — Row-variable surface (D1 ★, Architect-grounded) → `36 §1.5`

Architect's D1 ruling (`evt_53ybqtzjfv7yx`), transcribed by spec-author into a
new `36 §1.5` (normative extension of `§1.2`/`§1.3`); Architect fidelity-gates
the committed prose. **Headline: the internal row-variable machinery is already
landed; D1's gap is purely surface-writability + one bounded fixpoint lift.**

- **Landed ground truth (grep, `crates/ken-elaborator/src/effects/`):**
  `RowVar(u32)` + `RowType = Concrete | Var | Join` (`row.rs`), symbolic
  `infer_row_poly` (`row_poly.rs`), `check_row_poly_escape` with the `Var(x) ⊆
  Var(x)` rule, `apply_subst` for instantiation. But `RowVar` is constructed at
  **exactly one site** (`extract.rs`, fired only by a HOF-effectful parameter) —
  **no surface path from a written `visits [e]` to a `RowType::Var`** — `§1.3`
  "no surface row-variable binder" verbatim. D1 adds that surface path. (These
  code anchors are perishable build-facing detail — kept out of the normative
  spec, which states only the model.)
- **Load-bearing pins carried into `36 §1.5`** (Architect's fidelity-gate list):
  - **Surface variable required, not optional** — `§3.1` guarantee 1
    (manifest-in-the-type); the purity check must read the poly row off the
    signature. `[e]` (bare) and `[E | e]` (open tail) both accepted; a row
    variable **binds as an implicit parameter**, one variable, two occurrences
    (HOF-arg latent row + the declared row), same `RowVar`.
  - **Static closure is structural** (AC3) — a `RowVar` is eliminated only by
    instantiation at a concrete call or by deferral; at any boundary the row is
    concrete. No runtime effect discovery.
  - **No `Cap e`** — a row-poly `proc` performs its polymorphic effects only
    through its HOF argument (a closure the caller built with its own caps); its
    own direct-perform row is `∅`. Authority rides the argument.
  - **Recursive-fixpoint lift = the one build seam** — `§1.3`'s fixpoint is
    concrete-only; a recursive row-poly def (`traverse` folds a `List`, so it
    self-calls) needs it lifted to range over `RowType`. Monotone, idempotent
    (`e ∪ e = e`), terminating — the row-poly analog of CAT-1's bounded
    extension, outer-ring/kernel-untouched. **Flagged so the build won't hit it
    cold.**
  - **Fail-closed completeness residual** (for CV) — the `x ⊆ [E | e]` subset
    test is conservative single-arm; it *under-accepts* a straddling row
    (rejects-valid), never over-accepts. A known-completeness marker, not a
    soundness flip.
- **Register:** pure spec addition, **not** a fresh operator OQ — recorded as an
  `OQ-8` child pin (`spec/90-open-decisions.md`).

### E3 — Unicode surface (D3) → `31 §1c`

**Answer: both lexer and formatter**, a direct consequence of the DECIDED
`OQ-syntax` principles (`31 §1a`), not a new fork. The lexer accepts a curated
Unicode glyph and its ASCII transliteration as the **same token** (principle 2 —
**ASCII stays accepted forever**); the mandated formatter **emits canonical
Unicode** on save (principle 3); **keywords stay ASCII words** (principle 4, so
`const`/`fn`/`proc` are ASCII); confusable-rejection is a hard lexer gate
(principle 5, TR39). BL3 realizes the lexer + formatter and converts the corpus
(with D4).

### E4 — Migration (D4) — rule + worked classification

- **The rule:** every existing `.ken` and doc snippet migrates `view`/top-level
  `let` → `const`/`fn`/`proc` **classified by the checker's own purity
  inference** — mechanical and checked, not hand-judged. Land with the
  Unicode conversion (D3) as **one workspace-green unit**; rosetta 16/0; no
  `.ken` retains `view` (AC6).
- **Worked classification of the landed corpus** (grounding grep — ~444 `view` +
  ~18 top-level `let`):
  - **`const`** ← a pure value, 0 explicit value params (a bare `let x = …`).
  - **`fn`** ← a pure function, ≥1 explicit value param — e.g. `decimalAdd`,
    `eqChar`, `isSorted`, `list_append`, CAT-1's `list_assoc`/`bool_and`.
  - **`proc`** ← performs a concrete effect or uses a `Cap` — e.g. `read_bytes`
    (`FS a`, `Cap a`), `print_line` (`IO Unit`); **or** a `space`/`becomes` op.
  - **`proc`** ← effect-polymorphic (declares `[e]`) — **none yet** (CAT-2
    `traverse`).

  The corpus has **no** surface row-variable today, so migration needs no `[e]`
  spelling — every existing def is pure (`→ fn`/`const`) or concrete-effectful
  (`→ proc`). Landed `map`-style HOFs keep **pure** arrows, so they stay `fn`
  (only an *effect-polymorphic* arrow makes a HOF a `proc`).
- **Keyword-collision hazard (grounded):** **no** `.ken` in the corpus names an
  identifier `const`/`fn`/`proc` (grep-verified), so the migration is collision-
  free there. The **only** collision is the spec's V0 K-combinator example `view
  const` → renamed `fn konst` (`32 §8`).

### E5 — Build sequencing (held for the GPT window)

1. **D1 recursive-fixpoint lift** (the one bounded seam, E2) — lands first; it
   **gates `traverse` and CAT-2** (the first surface effect-polymorphic def).
2. **D2 checker** — the bidirectional purity check over the existing row
   inference; the `fn`-false-purity half is the landed `§1.4` gate.
3. **D3 + D4** — the Unicode lexer/formatter and the `view →`
   `const`/`fn`/`proc` + ASCII→Unicode corpus migration, landed together as one
   workspace-green unit.

Nothing is held on the enclave side (D1 landed, so the row-variable-dependent
pieces are folded); all of the above is **outer-ring, kernel-untouched**.
