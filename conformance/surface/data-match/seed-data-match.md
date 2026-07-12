# Surface conformance — `data` / `match` / refinements (L2)

Format: `../../README.md`. These pin Ken's algebraic-data surface (`spec/
30-surface/34-data-match.md`, impl-ready L2): real sum types with computing
eliminators, indexed families, `match` → `elim_D`, **required exhaustiveness +
reachability**, and refinement types. They are the **non-reproduction** of the
prototype's stubbed sums and missing exhaustiveness.

> This file is the **one home** for the `surface/data-match/*` property. The
> three bootstrap cases that lived in `../seed-surface.md`
> (`construct-then-eliminate`, `exhaustiveness-required`, `refinement-
> obligation`) are **subsumed** at L2 rigor (AC1, AC3, AC7); see that file's
> pointer.

## Reading disciplines (how to read every case below)

- **No new kernel rule.** Every case lowers to the **landed** kernel: `data` →
  inductive family + `elim_D` (`14`, K1/K1.5), `match` → `elim_D` (`39 §2.6`),
  refinement → **carrier `A` + emitted obligation** (`21 §2`, `22 §2.1`). A case
  that asserts a kernel *rejection* is asserting the **landed** kernel's verdict
  (`check_positivity`, eliminator well-formedness), not a new gate.
- **The exhaustiveness checker is untrusted; the safety is kernel-backed**
  (`34 §4.4`). The *safety* (no silently-partial `match`) holds even against a
  buggy checker — the kernel cannot type an `elim_D` missing a method. What the
  surface uniquely owns is the **named unmatched-pattern witness**, so the
  discriminating cases assert that **structural** output, not only accept/reject
  (a disabled checker still *rejects*, via the kernel — green-vs-green on the
  bare verdict).
- **type-possible-at-index ⇒ required; index-impossible ⇒ omittable** (`34
  §4.3`) is **one rule** at two index regimes; AC3, AC5, and AC9's explicit
  dependent-constructor coverage case are its faces, and the cross-case sweep
  asserts they agree.
- **(soundness)** cases encode a commitment that must never regress (`../../
  README.md`): `{TR3, TR5b, TR7}` — the headline exhaustiveness safety, the
  index-impossible auto-fill-by-absurdity (a wrong fill admits a partial
  function), and obligation completeness (a missed refinement obligation reads
  `proved`, `22 §intro`).
- **(oracle)** tags a deferred surface spelling to confirm against Ken's
  reference once it lands — here the **diagnostic token/format** of the
  unmatched-pattern witness (the *concept* "rejects naming the uncovered
  constructor" is locked; the literal error-kind string and witness rendering
  are `(oracle)`).

## surface/data-match/construct-then-eliminate (AC1)
- spec: `spec/30-surface/34-data-match.md §1`, `10-kernel/14 §3`
- given: `data Option a = None | Some a`; `match (Some 3) { Some x => x; None =>
  0 }`
- expect: **reduces-to** `3` (the emitted `elim_Option` ι-reduces on the `Some`
  constructor, `14 §3`) — a real constructor **and** a real, computing
  eliminator.
- why: sum types are finished, not lowered to an opaque base with no eliminator.
  **Flip:** the prototype's stub (opaque base, no `elim`) is **stuck** — it does
  **not** reduce to `3`. Structural: assert the reduct is the literal `3`, not
  merely "compiles".

## surface/data-match/match-elaborates-to-elim (AC2)
- spec: `spec/30-surface/34-data-match.md §3`, `39 §2.6`
- given: `match s { Circle r => r ; Rect w h => w }` on `s = Circle 2`; and a
  **nested** `match` (a `match` in an arm body)
- expect: the emitted core is an **`elim_Shape` application** (not a primitive
  `match` node), and it **computes** on the constructor (`Circle 2` ⇒ `2`);
  nested `match` ⇒ **nested `elim`**.
- why: `match` is not a kernel primitive (`34 §3`). **Flip:** an implementation
  that kept `match` as an opaque core former (no `elim_D`) emits a non-`elim`
  head — structurally distinguishable; one that fails to nest emits a single
  flat `elim` for a nested pattern. Structural: assert the head former is
  `elim_Shape` and (nested case) that the arm body is itself an `elim`.

## surface/data-match/exhaustiveness-required (AC3) (soundness) — TR3
- spec: `spec/30-surface/34-data-match.md §4.1`, `§4.4`
- given: `match (c : Color) { Red => 0 ; Green => 1 }` over
  `data Color = Red | Green | Blue` (the `Blue` arm missing); and the exhaustive
  version with all three arms
- expect: the missing-case version **rejects** as **non-exhaustive, naming the
  unmatched pattern `Blue`** (the witness, `34 §4.1`); the exhaustive version
  **accepts** and its `elim_Color` reduces.
- why: exhaustiveness is the headline safety the prototype lacks. **Flip — and
  why the named witness is load-bearing:** the bare verdict is *not*
  discriminating — under the exact bug (exhaustiveness check disabled) the
  missing-case `match` **still rejects**, because the elaborator cannot build a
  complete `elim_Color` (the `Blue` method has no body and MUST NOT be
  fabricated, `34 §4.3`) and the **kernel** rejects the under-applied eliminator
  (`34 §4.4`). So the green-vs-green trap is "both reject". The discriminating
  signal is the **named witness**: correct ⇒ "non-exhaustive: `Blue`" (surface,
  `(oracle)` on the exact token); disabled-checker ⇒ a bare kernel "eliminator
  under-applied" with **no** named pattern. Assert the witness `Blue`.

## surface/data-match/reachability-redundant-arm (AC4)
- spec: `spec/30-surface/34-data-match.md §4.2`
- given: `match (c : Color) { Red => 0 ; Green => 1 ; Blue => 2 ; Red => 9 }`
  (the 2nd `Red` arm subsumed by the 1st under first-match); and the
  all-reachable 3-arm version
- expect: the redundant-arm version **flags** the trailing `Red` arm as
  **unreachable** (warning/error, `34 §4.2`); the all-reachable version
  **accepts** with no flag.
- why: first-match reachability. **Flip:** correct ⇒ flags the 4th arm;
  buggy (no reachability) ⇒ **accepts silently**. Verdict (flag vs no-flag)
  flips on the exact bug. Companion (guards, `34 §3.3`/§4.2): a *guarded*
  `Red if p => …` followed by an unguarded `Red => …` is **reachable** (the
  guard may fail) — asserted so a checker that wrongly counts a guarded arm as
  covering is caught (it would mis-flag the unguarded `Red` as redundant).

## surface/data-match/indexed-impossible-pair (AC5) (soundness) — TR5a + TR5b
- spec: `spec/30-surface/34-data-match.md §2`, `§4.3`
- given: `data Vec a : Nat → Type { VNil : Vec a 0 ; VCons : {n} → a → Vec a n →
  Vec a (n+1) }`; (a) `view head {n} (v : Vec a (n+1)) : a = match v { VCons x _
  => x }` — **omitting** the `VNil` arm; (b) applying `head` to `VNil`
- expect — **the non-degenerate pair on one rule**:
  - (a) **accepts** — `VNil` is **index-impossible** at `n+1` (`0 ≢ n+1`); the
    arm may be omitted; the elaborator synthesizes the `VNil` method by
    **absurdity** (`34 §4.3`) and the kernel admits a **total** `elim_Vec`.
    **(TR5b, soundness)**
  - (b) **rejects** — `head` (domain `Vec a (n+1)`) applied to `VNil`
    (`: Vec a 0`) is a **kernel type error** (`0 ≢ n+1`). **(TR5a)**
- why: indexed non-emptiness is in the type. **Flip / non-degeneracy:** the pair
  must move in **opposite** directions on the *same* rule — accept the omission
  **while** rejecting the impossible application. A bug that treats
  index-impossible as type-possible would **reject (a)** (demand the `VNil`
  arm); a bug that fabricates a non-absurd `VNil` would **accept (b)** (or
  admit a partial `head`). Asserting only one side is green-vs-green; the pair
  pins that "index-impossible" is computed, not assumed.

## surface/data-match/dependent-constructor-vec-declaration (AC9)
- spec: `spec/30-surface/32-grammar.md §1`, `34 §2`, `39 §2.2`,
  `10-kernel/14 §1`
- given: the explicit dependent-constructor form for length-indexed vectors:

  ```ken
  data Vec (A : Type) : Nat -> Type where {
    VNil  : Vec A 0;
    VCons : (n : Nat) -> A -> Vec A n -> Vec A (n+1)
  }
  ```

- expect: **accepts** through parse, data-declaration elaboration, and kernel
  re-check. The emitted kernel family has parameter telescope `(A : Type)`,
  index telescope `Nat`, `VNil` at index `0`, and `VCons` with constructor
  telescope `(n : Nat) -> A -> Vec A n` targeting `n+1`.
- why: this is the new surface producer path for GADT-like declarations. A build
  that only accepts old `data D = C type_atom*` forms rejects before reaching the
  kernel; a build that parses the syntax but drops result indices emits the wrong
  constructor targets. Structural: assert the emitted family/constructor
  metadata, not merely that some declaration accepted.

## surface/data-match/legacy-form-explicit-signature-rejected (AC9)
- spec: `spec/30-surface/32-grammar.md §1`, `34 §2`
- given: the staged-out spelling that puts an explicit constructor signature in
  the legacy `=` form:

  ```ken
  data Box A = Mk : A -> Box A
  ```

  and the two accepted neighboring spellings:

  ```ken
  data Box A = Mk A

  data Box (A : Type) : Type where {
    Mk : A -> Box A
  }
  ```

- expect: the legacy `Mk : A -> Box A` spelling **rejects at the syntax
  boundary**: the `=` form accepts only `simple_ctor` / default-result sugar.
  The old simple `Mk A` form still **accepts**, and the explicit `where` form
  with `Mk : A -> Box A` **accepts**.
- why: this prevents the two declaration forms from silently converging. A
  parser that reuses the full `data_ctor` production in the legacy `=` arm would
  accept the staged-out spelling while still passing the positive explicit
  `where` cases.

## surface/data-match/proof-carrying-constructor-signature (AC9)
- spec: `34 §2.2`/`§2.3`, `39 §2.2`
- given: a checked-source-style proof-carrying constructor shape:

  ```ken
  data CheckedSource : Type where {
    MkCheckedSource :
      (sid : SourceId) ->
      (bs : Bytes) ->
      (len : Nat) ->
      UnitByteLength bs ->
      IsUtf8 bs ->
      SourceLength bs len ->
      CheckedSource
  }
  ```

- expect: **accepts** as a single constructor telescope with earlier binders
  (`bs`, `len`) in scope for later proof argument types and with result target
  exactly `CheckedSource`. This acceptance does not change CAT-5 `Source` and
  does not implement CAT-5 D3.
- why: the syntax must express evidence-carrying constructors without encoding
  them as records, class dictionaries, or smart constructors. **Flip:** a parser
  that treats `MkCheckedSource :` as the old atom-list syntax rejects; a scoping
  bug fails to resolve `bs`/`len` in the later proof argument types.

## surface/data-match/bad-constructor-result-target-rejected (AC9) (soundness)
- spec: `34 §2.3`, `39 §2.2`
- given: a constructor whose codomain is not the declared family at its declared
  parameters:

  ```ken
  data Vec (A : Type) : Nat -> Type where {
    BadHead  : List A;
    BadParam : Vec Bool 0
  }
  ```

- expect: **rejects** with a bad-constructor-result-target diagnostic naming the
  offending constructor and the expected family head `Vec A _`. The rejection
  must not be reported as a positivity failure, missing constructor argument, or
  generic parse error.
- why: constructor result targets define the family being admitted. Accepting
  another head or changed parameters would smuggle a constructor into the wrong
  inductive family. The diagnostic class is load-bearing for implementers: the
  error is at the surface result-target check before a kernel inductive
  declaration can honestly be formed.

## surface/data-match/explicit-signature-positivity-rejected (AC9) (soundness)
- spec: `34 §1`/`§2.3`, `39 §2.2`, `10-kernel/14 §8`
- given: the explicit-signature spelling of a negative recursive occurrence:

  ```ken
  data Bad : Type where {
    MkBad : (Bad -> Bool) -> Bad
  }
  ```

- expect: **rejects** through the kernel strict-positivity admission gate,
  surfaced at `MkBad`. The elaborator may preflight enough to report a better
  span, but the normative verdict is the landed kernel's negativity verdict; no
  new surface positivity rule is trusted.
- why: explicit constructor signatures must not bypass the existing kernel
  soundness gate. **Flip:** a build that treats the constructor type as an
  opaque field list or fails to route the emitted inductive through
  `check_positivity` admits `Bad`.

## surface/data-match/gadt-coverage-possible-vs-impossible (AC9) (soundness)
- spec: `34 §2`, `§4.3`, `39 §2.2`
- given: the explicit `Vec` declaration above and two matches:

  ```ken
  fn head (A : Type) (n : Nat) (v : Vec A (n+1)) : A =
    match v { VCons _ x _ |-> x }

  fn bad_head_any (A : Type) (n : Nat) (v : Vec A n) : A =
    match v { VCons _ x _ |-> x }
  ```

- expect:
  - `head` **accepts**: `VNil` is index-impossible at `n+1`, so its method is
    synthesized by absurdity.
  - `bad_head_any` **rejects** as non-exhaustive, naming the unmatched
    type-possible pattern `VNil` because `n` may be `0`.
- why: this pins the exact negative distinction requested for dependent
  constructors. A checker that treats all omitted constructors as impossible
  accepts `bad_head_any`; a checker that treats all family constructors as
  required rejects `head`. The pair must move in opposite directions on the same
  type-possible-at-index rule.

## surface/data-match/branch-refinement-is-hypothesis (AC6)
- spec: `spec/30-surface/34-data-match.md §3.3`, `20-verification/22 §3`
- given: a **dependent** `match` whose result type depends on the scrutinee —
  e.g. `match (v : Vec a m) { VNil => … ; VCons … => … }` with an `ensures` over
  the length, so each arm's expected type is the motive at that constructor
- expect: the emitted `elim_Vec` carries a **dependent motive** `M` (`34 §3.2`),
  and in the `VCons` arm the obligation context `Γ` gains the **scrutinee
  equation** `Eq (Vec a m) v (VCons …)` (`22 §3`) — usable as a hypothesis.
- why: per-branch definitional refinement is the surface origin of `22 §3`'s
  path-sensitive `Γ`. **Flip:** a **constant** (non-dependent) motive where a
  dependent one is required emits a *different* core term — the `elim_Vec`
  motive is `λ i x. T` with `x` unused, and the branch `Γ` lacks the scrutinee
  equation. Structural: assert the motive **mentions** the scrutinee/index
  (not a constant) and the branch hypothesis is present — verdict-independent,
  per the untrusted-layer lesson (a constant motive can still type-check, so the
  verdict alone is green-vs-green).

## surface/data-match/proof-returning-dependent-motive (AC8)
- spec: `spec/30-surface/34-data-match.md §3.5`,
  `spec/30-surface/39-elaboration.md §2.1`, `10-kernel/14 §3`
- given — **positive red-to-green**:

  ```ken
  fn km_scrutinee (b : Bool) : Bool = b

  fn km_proof_motive_positive (b : Bool)
    : Equal Bool (km_scrutinee b) (km_scrutinee b) =
    match km_scrutinee b {
      True  |-> tt ;
      False |-> tt
    }
  ```

  Companion **negative sibling**:

  ```ken
  fn km_proof_motive_negative (b : Bool)
    : Equal Bool (km_scrutinee b) True ->
      Equal Bool (km_scrutinee b) True =
    match km_scrutinee b {
      True  |-> \p. p ;
      False |-> \p. tt
    }
  ```

- expect:
  - the positive case **accepts**: the `match` elaborates to the ordinary
    `elim_Bool` with an `Ω`-codomain motive recovered from the expected target
    `P[x] := Equal Bool x x`, after transparent unfolding of `km_scrutinee`.
    The current pre-fix symptom to assert in the build regression is the
    whole-body rejection
    `KernelRejected { TypeMismatch { expected: Type 0, found: Ω0 } }`, not a
    generic `is_err()`.
  - the negative case **rejects** after branch specialization: in the `False`
    branch, the codomain is `Equal Bool False True` (`Bottom`), while `tt`
    proves `Top`. This must remain a wrong-specialized-branch failure, not an
    accepted proof by proof irrelevance and not a constant motive that ignores
    `km_scrutinee b`.
- why: proof-returning dependent `match` is ordinary dependent elimination into
  `Ω`, and branch obligations remain exact before proof irrelevance. This is
  the minimized Ken-owned shape for KM-dependent-match-proof-motive: a single
  two-constructor scrutinee, an `Equal`/proof target mentioning the scrutinee
  through one transparent reducible head, and branch-local proof closure. Direct
  `match b` proof motives already check on the pre-fix branch, so the reducible
  head is load-bearing; broader non-variable/nested/multi-scrutinee recovery and
  indexed-family proof motives stay out of this seed unless a later build proves
  they are the same mechanism.

## surface/data-match/refinement-obligation (AC7) (soundness) — TR7
- spec: `spec/30-surface/34-data-match.md §5`, `21 §2`, `22 §2.1`
- given: `def NonNeg = { n : Int | n ≥ 0 }`; (a) passing a plain `Int` `e`
  where `NonNeg` is expected (introduction); (b) passing a `NonNeg` where an
  `Int` is expected (forgetful)
- expect:
  - (a) the obligation `e ≥ 0` is **emitted** at that point (`22 §2.1`),
    discharged or left a visible hole — **never** a silent coercion past `φ`;
    the core image of the value is the **carrier `Int`** (no kernel `Σ`).
    **(soundness)**
  - (b) **no** obligation — `{n:Int|n≥0} ≤ Int` is **free** (the identity on the
    carrier, `22 §2.1`/§2.5).
- why: refinements enforce; using `A` as `{x:A|φ}` costs a proof, the reverse is
  free. **Flip:** a missed obligation on (a) reads `proved` with **zero** proof
  (the `22 §intro` linchpin — completeness is backstopped by nothing
  downstream), so observe the **emitted VC** structurally (obligation `n ≥ 0`
  is in the set), not just the final verdict. A spurious obligation on (b) (the
  forgetful direction) is the dual bug — assert the set is **empty** there. The
  pair (emit-on-intro / silent-on-forget) flips on the direction.

## Coverage map

| Case (AC)                         | Frame AC | Pins                                   | Tag        |
|-----------------------------------|----------|----------------------------------------|------------|
| construct-then-eliminate          | AC1      | real ctor + computing `elim_D`        |            |
| match-elaborates-to-elim          | AC2      | `match`→`elim_D`, nested→nested       |            |
| exhaustiveness-required           | AC3      | non-exh rejects **naming** `Blue`     | soundness  |
| reachability-redundant-arm        | AC4      | redundant arm flagged; guard subtlety |            |
| indexed-impossible-pair           | AC5      | reject impossible app / omit imposs.  | soundness  |
| dependent-constructor-vec-decl    | AC9      | explicit family/ctor targets          |            |
| legacy-explicit-signature-reject  | AC9      | `=` form stays simple/default-result  |            |
| proof-carrying-ctor-signature     | AC9      | telescope scoping, proof fields       |            |
| bad-constructor-result-target     | AC9      | target must be declared family        | soundness  |
| explicit-signature-positivity     | AC9      | kernel positivity still gates         | soundness  |
| gadt-coverage-possible-impossible | AC9      | omit impossible / require possible    | soundness  |
| branch-refinement-is-hypothesis   | AC6      | dependent motive + `22 §3` hypothesis |            |
| proof-returning-dependent-motive  | AC8      | `Ω` proof motive + exact branches     |            |
| refinement-obligation             | AC7      | emit-on-intro / free-on-forget        | soundness  |

## Cross-case sweep (internal consistency)

- **The coverage class agrees** (`34 §4.1`/§4.3): every coverage case treats a
  constructor as **required iff type-possible at the scrutinee's index**. AC3
  (`Color`, trivial index ⇒ all three required ⇒ `Blue` missing rejects), AC5
  (`Vec a (n+1)` ⇒ `VNil` index-impossible ⇒ omittable), and AC9 (`Vec A
  (n+1)` omits `VNil`, while `Vec A n` must still cover `VNil`) are the faces
  of one rule and must not contradict: a reading that made `VNil` "required at
  `n+1`" would also have to make `Blue` "omittable at `Color`" — they move
  together. No case asserts a constructor *both* required and omittable at the
  same index.
- **Obligation direction is consistent** (`22 §2.1`): every introduction emits
  (AC7a, AC3's "MUST NOT fabricate" is the analogous no-silent-fill on the
  eliminator side); every forgetful/free direction emits nothing (AC7b). No case
  emits an obligation for a forgetful coercion or a silent coercion for an
  introduction.
- **Untrusted-layer structural assertion** where the bare verdict is
  green-vs-green: AC3 (named witness), AC6 (dependent motive shape), AC7
  (emitted VC). Each names the **structural** signal, not just accept/reject —
  the cases that would otherwise pass vacuously under their exact bug.
- **Proof motives do not weaken branch checking** (`34 §3.5`, `39 §2.1`): AC8's
  positive accepts only because each branch checks at the constructor-specialized
  `Ω` target; its negative sibling must reject at the wrong branch target. A fix
  that accepts the negative by erasing the dependency or by using proof
  irrelevance before method checking contradicts AC6's dependent-motive
  discipline rather than extending it.

## surface/data-match/dependent-match-refinement (DS-5b, `34 §3.2.1`)

`§3.2.1` extends `§3.2` motive recovery so a branch's constructor index equation
(e.g. `Suc m ≡ Suc n` in `VCons`'s arm of `Vec A (Suc n)`) is carried into the
local context via the kernel's own `Eq`/`J`/`Cast` — **never postulated**. The
positives below are **landed** (DS-5b, `origin/main`); the two-vector `zip`
recursive step and `Fin`-indexed `lookup` are `(gated: DS-5c)` — the named
follow-on `§3.2.1` scopes. Vocabulary matches the acceptance suite
`ds5b_dependent_match_refinement_acceptance.rs` and the AC5/AC9 cases above:

```ken
data Vec (A : Type) : Nat -> Type where {
  VNil  : Vec A 0 ;
  VCons : (n : Nat) -> A -> Vec A n -> Vec A (n+1)
}
```

### surface/data-match/dr-injectivity-and-over-refinement (DS-5b) (soundness) — TR-DR
- spec: `spec/30-surface/34-data-match.md §3.2.1`; `10-kernel/16 §2.2`
  (same-constructor `Eq`-at-inductive); the `J`/`Cast` discharge (`16`)
- given: the peeled-recursive-field re-typing (`tail`), paired with the
  over-refinement negative on the **same** match shape:

  ```ken
  fn tail (A : Type) (n : Nat) (xs : Vec A (Suc n)) : Vec A n =
    match xs { VCons m y ys |-> ys }

  fn wrongGoal (n : Nat) (xs : Vec Nat (Suc n)) : Vec Nat (Suc n) =
    match xs { VCons m y ys |-> ys }
  ```
- expect:
  - `tail` **accepts** — `VCons`'s equation `Suc m ≡ Suc n` reduces (kernel
    same-constructor `Eq`) to `m ≡ n`, so the peeled field `ys : Vec A m` is
    `cast` up to the goal `Vec A n`. The emitted body carries a real `J`-derived
    `Cast`, and `trusted_base()` is **unchanged** (zero-`Axiom`).
  - `wrongGoal` **rejects** — `KernelRejected { error: TypeMismatch { .. } }`
    (assert the specific variant, not bare `is_err()`): the only equation
    `Suc m ≡ Suc n` licenses is `m ≡ n`, never `m ≡ Suc n`, so `ys : Vec Nat m`
    does **not** re-type to `Vec Nat (Suc n)`. No `Cast` is fabricated without a
    real premise — the kernel is the arbiter.
- why: the **non-degenerate discriminating pair** on a shared match shape — the
  refinement is exactly what injectivity licenses, no more. A bug that
  over-refines (fabricates a `Cast` to any goal) **accepts `wrongGoal`** (the
  unsoundness vector); a bug that under-refines (never re-types the peeled field)
  **rejects `tail`**. The pair flips in opposite directions on the same
  `Suc m ≡ Suc n` premise. This is the load-bearing soundness net, and it stays
  **live** while `zip`/`lookup` are gated below.

### surface/data-match/dr-sibling-convoy-and-goal-refinement (DS-5b)
- spec: `spec/30-surface/34-data-match.md §3.2.1`
- given: the single-level sibling convoy and the base-case goal refinement:

  ```ken
  fn firstIsSecond (n : Nat) (v : Vec Nat n) (w : Vec Nat n) : Bool =
    match v { VNil |-> True ; VCons m a xs |-> match w { VCons k b ys |-> True } }

  fn firstIsVNil (n : Nat) (v : Vec Nat n) (w : Vec Nat n) : Vec Nat n =
    match v { VNil |-> VNil Nat ; VCons m a xs |-> v }
  ```
- expect: both **accept**.
  - `firstIsSecond` — matching `v` refines `n`; the **outer** sibling binder `w`
    (a parameter sharing the index, never destructured by the outer match) is
    re-typed in lockstep, so the nested `match w` is exhaustive with **no**
    impossible `VNil` arm. Un-refined, this is `ExhaustivenessError` on the
    omitted `VNil`.
  - `firstIsVNil` — the `VNil` base-case arm **constructs** a fresh `VNil Nat`
    (no context binding to redirect); its checking goal is refined and the result
    `cast` back to the caller's goal.
- why: covers convoy (capability 2) and goal refinement (capability 3) — the two
  DS-5b forms beyond peeled-field injectivity. **Single-level** is load-bearing:
  one sibling, one nested match, no reuse of an enclosing-match field (that reuse
  is exactly the `zip` gap below).

### surface/data-match/dr-zip-two-vector (gated: DS-5c)
- spec: `spec/30-surface/34-data-match.md §3.2.1` (the named `DS-5c` boundary)
- given: the full two-vector `zip` recursive step, which nests a match on the
  second vector **and** reuses the first vector's own peeled tail in the
  recursive call:

  ```ken
  fn zipNat (n : Nat) (v : Vec Nat n) (w : Vec Nat n) : Vec Nat n =
    match v {
      VNil |-> VNil Nat ;
      VCons m a xs |-> match w { VCons k b ys |-> VCons m a (zipNat m xs ys) }
    }
  ```
- expect: `(gated: DS-5c)` — **rejects today** (`KernelRejected`), expected to
  **accept once `DS-5c` lands**. The gate is real: DS-5b's convoy handles **one**
  sibling through **one** nested match with no further reuse; reusing the
  enclosing match's `xs` in `zipNat m xs ys` hits `§3.2.1`'s named root cause
  (sibling-convoy cannot yet distinguish a genuine outer parameter from a field
  the enclosing match already bound). The rejection is fail-closed and
  kernel-backstopped (a completeness gap, "never unsound").
- why: **honest gate marker.** Un-stage to a positive accept when `DS-5c` lands;
  until then the AC8 net above (`dr-injectivity-and-over-refinement`) stays the
  live enforcer of the no-spurious-refinement posture. `(gated: DS-5c)`.

### surface/data-match/dr-lookup-fin (gated: DS-5c)
- spec: `spec/30-surface/34-data-match.md §3.2.1`; `50-stdlib/60 §3` (`Fin`)
- given: `lookup : (A : Type) → (n : Nat) → Vec A n → Fin n → A`, the safe
  positional accessor recursing through the vector's tail indexed by a bounded
  `Fin n` (`60 §3`).
- expect: `(gated: DS-5c)` — the `Fin`-indexed recursion into the tail is the
  same follow-on capability as `zip`'s two-vector step; it does not elaborate on
  the DS-5b elaborator and is staged to `DS-5c`. Design is total-by-construction
  (a `Fin n` **is** an in-range witness, no side-proof); only its elaboration is
  gated. Un-stage to a positive accept + its `§5` computation law when `DS-5c`
  lands.
- why: the second `DS-5c`-gated operation; kept design-stated + gated so the
  chapter's `§4` table (`lookup` gated on `DS-5c`) has a conformance home.

## Subsumed upstream (one home per property)

- `../seed-surface.md` `data-match/construct-then-eliminate`,
  `exhaustiveness-required`, `refinement-obligation` — **subsumed** here at L2
  rigor (AC1, AC3, AC7). That file now points here.
- The kernel-side eliminator/positivity commitments live in
  `../../kernel/inductive/` (`elim_Nat`/`elim_Vec` ι, positivity accept/reject,
  W-style IH) — **referenced, not duplicated**: this file pins the **surface**
  lowering (`data`/`match`/refinement → core), not the kernel's admission of the
  emitted core.

## Build-sequencing note

L2 **unblocks B2** (`Temporal` as an ordinary indexed `data`, not modalities —
keep the AC5 indexed-family path clean) and **T3** (test framework). The
exhaustiveness checker is a **surface** algorithm; the kernel already has the
total eliminator — keep the trust boundary crisp (`34 §4.4`): the kernel proves
the *eliminator* sound, the surface proves the *match covers* it with a named
witness.
