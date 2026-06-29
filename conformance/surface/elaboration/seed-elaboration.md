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
`convLevel`). The **V0 elaborator algorithm** lives in `39 §5.1–§5.7` — fleshed
to pseudocode by spec-author **concurrently** with this corpus; the `§5.x`
sub-numbers below follow the elaboration-plan structure and are **reconciled
against spec-author's landed numbering before lock** (section numbers drift —
the K2c lesson). Expected results are determined by the on-`main` spec + the
settled level/conversion decisions; none required reading the prototype.

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

## AC3 — ill-typed surface is rejected (by the kernel, surfaced not swallowed)

V0 emits a well-formed core term; the **kernel** judges it and the rejection is
surfaced with location (frame Acceptance 3; `39 §3` well-typed-output, `§4`).

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
- spec: `39 §5.6`; `13 §1`; `18 §3` (checking a λ requires a Π)
- given: `view badLam (x : Nat) : (y : Nat) -> Nat = \y . \z . x`
- expect: kernel **rejects** — after binding `y : Nat`, the inner `\z . x` is
  checked against `Nat`, which is not a Π type (an **over**-saturated λ).
- why: too many λs for the declared codomain. A λ can only check against a Π
  (`18 §3`); `\z . …` against `Nat` fails.

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

The one place V0 can corrupt a *well-typed-looking* term: a capture bug yields
core the kernel **accepts**. So the key case is **discriminating** — correct
resolution rejects, a capture bug accepts (frame guardrails; `39 §5.3`).

### surface/elaboration/shadow-outer-not-captured  ← discriminating
- spec: `39 §5.3` (first-binder resolution, capture-avoidance); `11-syntax §2`;
  `18 §3`
- given: `view shadow (A : Type) (x : A) : (A : Type) -> A = \A . x`
  (full term `λ A. λ x. λ A. x`)
- expect: kernel **rejects**. Under correct capture-avoiding resolution the body
  `x` is `Var 1`, whose type is the **outer** `A` (`Var 2` at the body); it is
  checked against the codomain `A` of `(A : Type) -> A`, which is the **inner**
  shadowing `A` (`Var 0`). Distinct binders → distinct de Bruijn indices → `Var
  2 ≢ Var 0` → reject.
- why: **the** name-resolution guard. If the resolver instead *captured* `x`
  under the inner `\A` — giving its type as the inner `A` — the term would
  type-check and be **wrongly accepted**. Correct ⇒ reject, capture bug ⇒
  accept: opposite verdicts, so this case isolates the bug the kernel cannot
  backstop. (The discriminating-case discipline — the K2c carry.)

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
