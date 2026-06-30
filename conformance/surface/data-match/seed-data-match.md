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
  §4.3`) is **one rule** at two index regimes; AC3 and AC5 are its two faces and
  the cross-case sweep asserts they agree.
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

## surface/data-match/refinement-obligation (AC7) (soundness) — TR7
- spec: `spec/30-surface/34-data-match.md §5`, `21 §2`, `22 §2.1`
- given: `type NonNeg = { n : Int | n ≥ 0 }`; (a) passing a plain `Int` `e`
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
| branch-refinement-is-hypothesis   | AC6      | dependent motive + `22 §3` hypothesis |            |
| refinement-obligation             | AC7      | emit-on-intro / free-on-forget        | soundness  |

## Cross-case sweep (internal consistency)

- **The coverage class agrees** (`34 §4.1`/§4.3): every coverage case treats a
  constructor as **required iff type-possible at the scrutinee's index**. AC3
  (`Color`, trivial index ⇒ all three required ⇒ `Blue` missing rejects) and AC5
  (`Vec a (n+1)` ⇒ `VNil` index-impossible ⇒ omittable) are the **two faces of
  one rule** and must not contradict: a reading that made `VNil` "required at
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
