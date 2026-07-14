# V0 (minimal elaborator) conformance — seed cases

Format: `../../README.md`. These pin the **surface → elaborate → kernel-check**
pipeline that **V0** delivers (`docs/program/wp/V0-elaborator.md`): the minimal
surface subset, name resolution to de Bruijn, bidirectional elaboration to core,
and the kernel as the sole judge of the emitted term. They extend — and must not
regress — the two on-`main` elaboration invariants in
`../seed-surface.md` (`well-typed-output`, `ambiguity-is-an-error`).

**Trust posture.** V0 is **not in the TCB**: its output is re-checked by the
kernel (de Bruijn criterion, `docs/PRINCIPLES.md`). A V0 bug yields a
*rejected-valid-program* or *bad diagnostic*, **not** unsoundness — so no case
here is `(soundness)`. The **one exception** to "the kernel backstops V0" is
**name resolution** (AC4): a capture bug produces a *well-typed-looking* core
term the kernel **accepts**, so the corpus must pin a **discriminating** case
where correct resolution and a capture bug give *opposite* verdicts. That is the
load-bearing guard (frame guardrails; COORDINATION §7).

**Tags.** `(oracle)` — expected core-term shape / default level solving / the
base-type environment is confirmed against the prototype at build time by the
Spec enclave (safe: V0 not in TCB). `(property)` — a pipeline/closure invariant
over many inputs, not a single trace.

**Citations.** Stable on-`main` anchors: `39-elaboration.md` §1–§4, §6 (trust
split, what elaboration does/guarantees, errors); `10-kernel/12-universes.md`
§1–§4 (levels); `13-pi-sigma.md §1` (Π-Form, Π-Intro, Π-η); `11-syntax.md §1,§2`
(core formers, de Bruijn binding); `18-judgments.md §3,§4` (bidirectional
`check`/`infer`, kernel verdict); `17-conversion.md §3.6` (decidable
`convLevel`). The **V0 elaborator algorithm** lives in `39 §5.1–§5.7` (landed on
`wp/V0-elaborator` by spec-author). The `§5.x` cites below are **reconciled
against that landed §5**: §5.1 surface subset, §5.3 name-res/de Bruijn, §5.4 the
`check`/`infer`/`elabType` algorithm, §5.5 pipeline, §5.6 errors, §5.7 level
reconcile — sub-numbers match (no drift), and one **content** refinement is
folded — the landed §5.6 detects a λ-vs-non-Π `LambdaVsNonFunction`
**structurally in V0** before the kernel, so `wrong-arity-rejected` attributes
that stage correctly (not "kernel"). Expected results are determined by the
on-`main` spec + the settled level/conversion decisions; none required reading
the prototype.

**Three brief corrections folded in** (flagged to spec-leader, not silently
dropped): the AC2 ascription example `(\x . x) : (A:Type) -> (x:A) -> A` is a
single λ against a **binary** Π and is *rejected*, not accepted — used as the
corrected Ok case `ascription-on-lambda` (two λs) **and** the AC3 dual-arity
reject `under-applied-lambda-rejected`; the AC4 `unbound` example typed its
binder with a free `A`, masking the intended free-`y` test — retyped with `Nat`;
the AC6 `poly` inner-Π level is `max(1,1)=1` (both components `1`), not
`max(1,0)`.

---

## AC1 — a trivial program elaborates and kernel-checks (the G1 precondition)

The headline: enough surface to write a dependently-typed function, run it
through `lex → parse → resolve → elaborate → kernel-check`, and get `Ok`
(`39 §5.5`, frame Acceptance 1).

### surface/elaboration/id-elaborates-checks (oracle)
- spec: `39 §5.1`, `§5.4`, `§5.5`; `13 §1`; `18 §3,§4`; `12 §1,§2`
- given: `view id (A : Type) (x : A) : A = x`
- expect: parses → elaborates → kernel-check **Ok**. Emitted core (de Bruijn):
  `Lam(Univ 0, Lam(Var 0, Var 0))` at declared type
  `Pi(Univ 0, Pi(Var 0, Var 1))`. Default level `0` (typical ambiguity,
  `12 §4`); inner `(x:A)->A : Univ (max 0 0) = Univ 0`; the whole type
  `: Univ (max 1 0) = Univ 1`.
- why: the minimal end-to-end pipeline. The body `x` is `Var 0` (the innermost
  binder); `id`'s declared Π re-checks against the emitted λ. Pins the exact
  core shape and the two Π levels (`12 §2`).

### surface/elaboration/const-elaborates-checks (oracle)
- spec: `39 §5.3`, `§5.4`; `11-syntax §2`; `13 §1`
- given: `view const (A B : Type) (x : A) (y : B) : A = x`
- expect: **Ok**. Body `x` is **`Var 1`**, not `Var 0` — the scope stack
  (innermost-first) is `[y, x, B, A]`, so `x` sits at index 1. Returns type `A`.
- why: a de Bruijn index that is **not** `0`. A resolver that always emitted the
  innermost index would return `y` here and the term would fail to check (`y :
  B`, expected `A`) — so this exercises index arithmetic, not just "the last
  binder."

### surface/elaboration/apply-elaborates-checks (oracle)
- spec: `39 §5.4`; `13 §1`; `18 §3` (application: infer `f ⇒ (x:A)→B`, check
  `u ⇐ A`)
- given: `view apply (A B : Type) (f : (x : A) -> B) (x : A) : B = f x`
- expect: **Ok**. Core body `App(Var 1, Var 0)` (`f` at index 1, the argument
  `x` at index 0 under stack `[x, f, B, A]`); `f : (x:A)->B` applied to `x : A`
  yields `B`.
- why: application elaboration with a Π-typed argument — `infer` the head, then
  `check` the argument against the domain (`18 §3`).

---

## AC2 — round-trip on the minimal surface

Each supported form (`let`, type ascription, base-type application) parses,
elaborates, and checks (frame Acceptance 2; `39 §5.1`).

### surface/elaboration/let-ascription-roundtrip (oracle)
- spec: `39 §5.4` (`let` rule); `11-syntax §1` (core `Let`); `12 §1` (U-Type);
  `18 §3`
- given: `let x : Type = Type in x`
- expect: **Ok**. The annotation `Type` is `Univ l1`, the value `Type` is
  `Univ l2` with `l1 = suc l2`; typical-ambiguity defaults `l2 = 0`, so
  `x : Univ 1 := Univ 0` and the body `x` is `Var 0 : Univ 1`. Core
  `Let(Univ 1, Univ 0, Var 0)`.
- why: a local binding with an ascription whose level constraint (`l1 = suc l2`)
  is solved during elaboration and emitted explicitly; the kernel re-checks
  `Univ 0 : Univ 1` (`12 §1`).

### surface/elaboration/ascription-on-lambda (oracle)
- spec: `39 §5.4` (ascription: check `e` against the ascribed type); `18 §3`
  (`(t:A)`: check `A ⇐ Type ℓ`, then `t ⇐ A`); `13 §1`
- given: `(\A . \x . x) : (A : Type) -> (x : A) -> A`
- expect: **Ok**. The two λs check against the binary Π; core
  `Lam(Univ 0, Lam(Var 0, Var 0))`, the same image as `id`'s body.
- why: type ascription drives **checking** mode (`18 §3`); a polymorphic
  identity written as a bare λ checks against its ascribed Π. (Corrected from
  the brief's single `\x . x`, which under-saturates the binary Π — pinned as
  a reject in AC3.)

### surface/elaboration/base-type-app (oracle)
- spec: `39 §5.4` (`Con` → environment lookup); `13 §1`; `18 §3`
- given: `view idNat (x : Nat) : Nat = x`, in a base environment providing
  `Nat : Type 0`
- expect: **Ok**. Core `Lam(Con "Nat", Var 0)` at type `Pi(Con "Nat", Con "Nat")
  : Univ 0`.
- why: a base type resolved from the environment (not a bound variable); the
  smallest non-`Type` example. The `Nat : Type 0` assumption is the `(oracle)`
  part — confirmed against the prototype's base environment at build time.

---

## AC3 — ill-typed surface is rejected (surfaced, not swallowed)

A body/return mismatch is rejected by the **kernel's `convert`** at the (Conv)
mode switch (`39 §5.4`, `18 §3`); a λ-vs-non-Π or non-function-head is detected
**structurally by V0** (`LambdaVsNonFunction` / `NotAFunction`, `39 §5.6`)
*before* the kernel call. Either way the error carries the surface span and is
surfaced, never swallowed (frame Acceptance 3; `39 §3` well-typed-output, `§4`).

### surface/elaboration/type-mismatch-rejected
- spec: `39 §5.6`; `18 §3` (the `(Conv)` switch); `12 §3` (non-cumulative)
- given: `view bad (x : Nat) : Bool = x`
- expect: elaborates to `Lam(Con "Nat", Var 0)` at declared type
  `Pi(Con "Nat", Con "Bool")`; kernel **rejects** — body `x : Nat` checked
  against `Bool`, and `Nat ≢ Bool`. The error surfaces with the source span of
  the body and names the two non-converting types.
- why: the kernel is the backstop for a body/return-type mismatch; V0 does not
  swallow it. Distinct base types do not convert.

### surface/elaboration/wrong-return-app-rejected
- spec: `39 §5.4`, `§5.6`; `18 §3`
- given: `view badApp (f : (x : Nat) -> Bool) (x : Nat) : Nat = f x`
- expect: elaborates `App(Var 1, Var 0)`; kernel **rejects** — `f x : Bool`
  checked against the return type `Nat`, `Bool ≢ Nat`.
- why: rejection through an application result, not just a bare variable — the
  mismatch is at the inferred result type of the spine.

### surface/elaboration/wrong-arity-rejected
- spec: `39 §5.4` (`check (RLam, notPi) → error`), `§5.6`
  (`LambdaVsNonFunction`); `13 §1`
- given: `view badLam (x : Nat) : (y : Nat) -> Nat = \y . \z . x`
- expect: **rejected** — after binding `y : Nat`, the inner `\z . x` is checked
  against `Nat`, which is not a Π. V0 raises `LambdaVsNonFunction`
  **structurally** (`39 §5.6`), surfaced in the type-mismatch class with the λ's
  span; the kernel is **not** reached for this term.
- why: too many λs for the declared codomain — an **over**-saturated λ. A λ can
  only `check` against a Π (`39 §5.4`); per the landed §5.6 this λ/non-Π
  mismatch is a V0-stage structural error, not a kernel `convert` failure. Pins
  the stage attribution (complements AC5).

### surface/elaboration/under-applied-lambda-rejected
- spec: `39 §5.6`; `13 §1`; `18 §3`
- given: `(\x . x) : (A : Type) -> (x : A) -> A`
- expect: kernel **rejects** — the single λ binds `A`, and the body `x` (= the
  bound `A : Type`) is checked against the remaining Π `(x : A) -> A`; `Type` is
  not that Π, so it fails.
- why: the **dual** of `wrong-arity-rejected` — too **few** λs for a binary Π.
  Together they pin both directions of arity mismatch. (This is the brief's AC2
  ascription example, which is a reject, not an accept.)

---

## AC4 — name resolution: nested binders + shadowing (the load-bearing guard)

The one place V0 can corrupt a *well-typed-looking* term: a mis-scoping/capture
bug resolves a variable to the wrong binder. The key case is **discriminating**
— correct resolution and a capture bug give **opposite** verdicts — and is
backed by a **structural** assertion on the resolved de Bruijn index that holds
*regardless* of the verdict, so it cannot pass vacuously (frame guardrails;
`39 §5.3`).

### surface/elaboration/shadow-outer-not-captured  ← discriminating
- spec: `39 §5.3` (first/innermost-match resolution); `11-syntax §2`; `18 §3`;
  `13 §1` (non-dependent `→` codomain)
- given: `view f (A : Type) (x : A) : Type -> A = \B . x`
  (full term `λ A. λ x. λ B. x`; the body `x` must resolve **past** the
  intervening `\B` binder to the outer `x`)
- expect: kernel **accepts** under correct resolution; a capture bug
  **rejects**. The return type `Type -> A` is **non-dependent** — its codomain
  `A` is resolved at the signature, so it is the **outer** parameter `A`
  (`Var 2` at the body), *not* the intervening `\B`. Correct: body `x` is
  `Var 1` (the outer `x`), whose type is that same outer `A` (`Var 2`) →
  `Var 2 ≡ Var 2` → **accept**. Capture bug (`x` → `Var 0` = the inner
  `B : Type`): the body now has type `Type`, checked against the codomain `A`
  (`Var 2`) → `Type ≢ Var 2` → **reject**.
- why: **the** name-resolution guard, made genuinely discriminating. The
  non-dependent `Type -> A` codomain pins the expected type to the *outer* `A`,
  so correct resolution (body `x` : outer `A`) accepts while a capture to the
  intervening `\B` (body : `Type`) rejects — opposite verdicts isolate the bug
  the kernel cannot otherwise backstop. **(Corrected — Architect-caught.)** A
  prior draft used a *dependent* `(A : Type) -> A` codomain (its `A` = the inner
  `Var 0`), under which the body rejects on **both** correct (`Var 2 ≢ Var 0`)
  and buggy (`Type ≢ Var 0`) resolution — vacuous, guarding nothing. The
  discriminating ingredient is the outer-referencing codomain. (Program matches
  the Architect's suggested form; the discriminating-case discipline — the K2c
  carry.)

### surface/elaboration/shadow-resolver-emits-outer-index (oracle)
- spec: `39 §5.3` (`indexOf` = first/innermost match); `11-syntax §2`
- given: the same `view f (A : Type) (x : A) : Type -> A = \B . x`
- expect: name resolution emits the body `x` as **`Var 1`** (the outer `x`
  parameter), **not** `Var 0` (the intervening `\B`). A direct,
  **verdict-independent** assertion on the resolver output.
- why: the most direct pin on the correctness-critical pass — it holds whether
  or not the term type-checks, so it **cannot go vacuous** (the failure mode of
  a verdict-only assertion). `indexOf` scans innermost-first and matches the
  first entry equal to `x`; since the intervening binder is named `B` (`≠ x`),
  `x` skips it and lands on the outer parameter. This is the same
  innermost-match rule that implements shadowing (`39 §5.3` property 1).
  Complements the discriminating behavioral case above.

### surface/elaboration/nested-app-each-binder (oracle)
- spec: `39 §5.3`, `§5.4`; `11-syntax §2`; `13 §1`
- given: `view nested (A : Type) (f : (x : A) -> A) (x : A) : A = f (f x)`
- expect: **Ok**. Stack `[x, f, A]`: `f` is `Var 1`, `x` is `Var 0`; core body
  `App(Var 1, App(Var 1, Var 0))`. Each `f` and `x` resolves to its correct
  binder; `f (f x) : A`.
- why: nested applications with repeated references — every occurrence resolves
  to the same correct binder. The positive complement to the shadow guard.

### surface/elaboration/unbound-name-rejected-at-resolution
- spec: `39 §5.3` (unbound ⇒ surface error), `§5.6`; `39 §4` (L1 surface error,
  distinct from a kernel error)
- given: `view unbound (x : Nat) : Nat = y`
- expect: **rejected at name resolution** — *before* any core term is emitted —
  with `error(unbound-name)` naming `y` and its source span. The kernel is
  **never** invoked (no core image exists).
- why: free variables are caught at the resolution stage, not the kernel; the
  error kind and stage are part of the contract (`39 §4`). Pins stage
  attribution: a name error is a surface/L1 error, not a kernel type error.
  (Retyped from the brief's `(x : A)`, whose free `A` would itself be unbound
  and mask the intended free-`y` test.)

---

## AC5 — pipeline integration (exact output; error at the right stage)

The emitted term is exactly a kernel-acceptable core `Term`, and each failure
mode is surfaced at its originating stage, not swallowed (frame Acceptance 3;
`39 §5.5`, `§5.6`).

### surface/elaboration/pipeline-emits-explicit-core (property)(oracle)
- spec: `39 §3` (well-typed-output), `§5.4` (levels explicit in output), `§5.5`;
  `12 §4`; `18 §3`
- given: every program AC1–AC2 **accepts** (e.g. `id`, `const`, `apply`,
  `idNat`)
- expect: each emitted core `Term` is **metavariable-free** and carries
  **fully-explicit** `Univ` levels (no unsolved level metavariables); the
  kernel's `infer` accepts it and returns the declared type.
- why: V0 solves all metavariables and levels during elaboration and hands the
  kernel fully-explicit core (`12 §4`: "the kernel never guesses a level"). The
  V0 specialization of the on-`main` `well-typed-output` invariant
  (`../seed-surface.md`).

### surface/elaboration/pipeline-errors-at-correct-stage
- spec: `39 §5.5`, `§5.6`; `39 §4`
- given: three inputs, one per stage —
  (a) `view id (A : Type) (x : A) : A =` (missing body);
  (b) `view u (x : Nat) : Nat = y` (free `y`);
  (c) `view bad (x : Nat) : Bool = x` (ill-typed)
- expect: (a) **parse error** with span — never reaches resolution; (b)
  **name-resolution error** naming `y` with span — never reaches the kernel; (c)
  **kernel rejection** surfaced as a surface error quoting the kernel's reason
  with the body's location. None is swallowed; each is attributed to its stage.
- why: the pipeline must fail at the **right** stage (parse vs. name-res vs.
  kernel) — a swallowed error, or a kernel that runs on a name-unresolved term,
  is a pipeline defect. Error-path coverage is as load-bearing as success-path
  (the K2 lesson).

---

## AC6 — levels reconcile (regression per the K2 / level-discipline lesson)

Explicit levels, the predicative-`max` Π rule, ≥2 distinct level variables, and
decidable level equality (`12 §1,§2,§4`; `17 §3.6`).

### surface/elaboration/id-pi-level-max (oracle)
- spec: `12 §1` (`Univ ℓ : Univ (suc ℓ)`), `§2` (Π-Form `max`); `13 §1`;
  `17 §3.6`
- given: `view id (A : Type) (x : A) : A = x`, default `A : Type 0`
- expect: the inner Π `(x : A) -> A` has level `max(0, 0) = 0` (`A : Univ 0`,
  codomain `A : Univ 0`); the full type `(A : Type 0) -> (x:A) -> A` has level
  `max(suc 0, 0) = max(1, 0) = 1`. Both levels appear **explicitly** in the
  emitted core.
- why: the predicative-`max` rule (`12 §2`) applied at both Π nodes, with the
  domain `Type 0` contributing `suc 0 = 1`. Exact levels, not "some level" (K2
  retro: precise expected levels).

### surface/elaboration/two-distinct-levels (oracle)
- spec: `12 §1,§2,§4`; `17 §3.6`
- given: `view poly (A : Type 1) (B : Type 2) (x : A) : A = x`
- expect: **Ok**. Two **distinct** explicit annotations `Type 1` and `Type 2`
  coexist in the core term; the inner Π `(x : A) -> A` (with `A : Univ 1`) has
  level `max(1, 1) = 1`. `Type 1` and `Type 2` are **not** interchangeable
  (`12 §3` non-cumulative).
- why: ≥2 distinct level variables exercised together (the K2 degrees-of-freedom
  lesson) — a single-level corpus hides a level-arithmetic bug. (Components are
  `(1,1)`, both from `A : Type 1`, not `(1,0)`.)

### surface/elaboration/level-equality-decidable (property)
- spec: `17 §3.6` (`convLevel`); `12 §1` (semilattice), `§3` (non-cumulative)
- given: the level annotations the elaborator emits (e.g. `max 0 0`, `max 1 1`,
  and a `Type 1` where `Type 2` is expected)
- expect: `convLevel` **decides** them — `max 0 0 ≡ 0` and `max 1 1 ≡ 1` accept;
  `Type 1 ≢ Type 2` (so a `Type 1` supplied where `Type 2` is required is
  **rejected**, no cumulative lift).
- why: the emitted levels are checked by the *same* decidable `convLevel` the
  kernel uses (`17 §3.6`) — V0 inherits decidability, it does not invent a level
  comparison. Ties the surface output to the K2c level semilattice.

---

## Regression: the on-`main` elaboration invariants still hold

### surface/elaboration/existing-invariants-still-green (property)
- spec: `39 §3`; `../seed-surface.md`
  (`surface/elaboration/well-typed-output`, `.../ambiguity-is-an-error`)
- given: every program V0 accepts, and any genuinely ambiguous input
- expect: **well-typed-output** — every accepted program's core image passes
  `kernel.check`; **ambiguity-is-an-error** — an unresolvable metavariable is a
  surface error with a span, never a silent default (beyond the declared
  level/numeric defaults).
- why: V0 is the first concrete elaborator; it must **satisfy** the pre-existing
  elaboration invariants, not redefine them. The K1/K2/K2c regression-anchor
  pattern, carried to the surface.

---

## LET-2 — local bindings as checked exposition

These cases pin the executable part of the local-binding authoring convention
(`docs/program/wp/let2-local-binding-convention.md`). The convention is a
judgment rule, not a count or an AST-depth metric. Its language-facing roots are
the existing non-recursive scope rule (`39 §5.3`) and strict evaluation
(`42 §3.2`; `36 §2.4` for effectful terms). The runtime sequencing behavior is
already homed in `../../runtime/evaluation/seed-evaluation.md` and
`../../runtime/effects/seed-effects.md`; LET-2 references those homes rather
than duplicating them here.

### surface/elaboration/guide-ken-examples-elaborate (property)
- spec: `39 §5.3`, `§5.4`, `§5.5`; `catalog/guide/README.md` "The three
  strands"; LET-2 AC1
- given: every tracked `catalog/guide/**/*.ken.md` document, enumerated from the
  repository rather than a fixed filename list
- expect: a fresh elaboration environment runs the real literate pipeline on
  each document. Every tangled `` ```ken `` fence parses and elaborates; every
  `` ```ken example `` fence elaborates; every `` ```ken reject `` fence
  rejects. The property fails with the document and fence location on the first
  stale example.
- why: the guide is executable teaching material. Checking only the new `let`
  examples would leave older examples free to rot, while a hard-coded path list
  would silently omit a newly added guide strand. The producer is the same
  literate elaboration path used by `ken check`, not a Markdown-text scan.

### surface/elaboration/let-rhs-binder-is-out-of-scope
- spec: `39 §5.3` property 2 and `§5.6`; LET-2 AC2
- given: `const let_rhs_self : Nat = let self_rhs_probe : Nat = self_rhs_probe in self_rhs_probe`
- expect: **rejected** with the specific
  `UnresolvedCon { name = "self_rhs_probe" }` diagnostic at the occurrence in
  the right-hand side. The harness matches the variant and name; a generic
  failure or bare `is_err` does not satisfy this case.
- why: resolution visits the right-hand side before adding the local binder to
  scope. The unique name prevents an earlier guide declaration from satisfying
  the reference as a global. This pins the stated non-recursive/out-of-scope
  reason rather than accepting coincidental rejection later in elaboration.

### surface/elaboration/let-body-binder-is-in-scope
- spec: `39 §5.3` property 2, `§5.4`; LET-2 AC2
- given: `const let_rhs_zero : Nat = let bound_value : Nat = Zero in bound_value`
- expect: **accepted**; the body occurrence resolves to the local binder while
  the right-hand side elaborates without it.
- why: the positive arm distinguishes the intended asymmetric scope boundary
  from an implementation that rejects local names everywhere. Together with
  `let-rhs-binder-is-out-of-scope`, changing only where the occurrence appears
  flips the verdict.
