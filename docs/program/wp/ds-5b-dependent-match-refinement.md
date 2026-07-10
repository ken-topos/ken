# DS-5b · Dependent-match index refinement — constructor injectivity + sibling convoy

**Owned by the Steward** (frame); **home: Kernel team** (soundness-adjacent
elaborator work — kernel-implementer → kernel-leader → kernel-qa → **Architect
soundness gate**). The **elaborator enhancement** that DS-5's Architect ruling
(`evt_1mnh5sngvhaty`) named as the sole prerequisite for the length-indexed
`Vector` API's `tail`/`zip`/`lookup`. Kicked in the operator's autonomous window
(2026-07-10) under the run's boundary rules (kernel/elaborator fixes may LAND
this run **if they are the right fix**, through the full ring + soundness gate;
soundness stays non-negotiable). **This is a language-capability / soundness-
adjacent call — logged prominently for the operator's end-of-run review**
(judgment log `K1`).

## What is already landed (do NOT rebuild — Architect-grounded on `origin/main`)

The indexed-family path is **complete at the kernel + family-declaration layer**,
with a landed 14-test regression suite (`tests/explicit_data_elaboration.rs`):

- `Vec (A : Type) : Nat → Type` with **index-refining** constructors
  (`vnil : Vec A Zero`, `vcons : (n:Nat) → A → Vec A n → Vec A (Suc n)`) —
  `data.rs::validate_ctor_result_target` fixes the params, refines the index.
- **Total `head : Vec A (Suc n) → A`** — single-scrutinee dependent `match`
  omitting the `vnil` arm, auto-discharged via `Term::Absurd` because the
  omitted arm's index premise `Zero = Suc n` is provably `Bottom`
  (`elab.rs::synthesize_omitted_index_method`, ~1270; `method_index_premises`,
  ~1203). An un-refined `v : Vec A n` head correctly **rejects** as
  non-exhaustive — the refinement is genuinely load-bearing.
- `Fin : Nat → Type` family decl elaborates (same mechanism).

The kernel already admits the family + the dependent `Elim`. **No kernel change
and no `data.rs` change is in scope here** — if either turns out genuinely
required, STOP and hand back to me (it would mean this frame mis-located the gap).

## The capability to add (precisely one coherent thing)

Dependent-`match` elaboration today recovers the motive over **the scrutinee's
own index only** (spec `34-data-match §3.2`). Extend it, in the dependent-match
path of `elab.rs` (the region around `method_index_premises` /
`synthesize_omitted_index_method`), to carry **each branch's constructor
equation** into the local typing context and use it to:

1. **Re-type peeled recursive fields via constructor injectivity.** In `tail`'s
   `vcons` branch, the branch learns `Suc m = Suc n` (scrutinee index vs ctor
   result index); by `Suc`-injectivity `m = n`, so the peeled field
   `xs : Vec A m` must convert to the required `Vec A n`. Today it does not —
   verbatim kernel reject `expected Vector a n, found Vector a m`.
2. **Re-type sibling context binders (the convoy).** In `zip`, matching
   `v : Vec A n` refines `n` (to `Zero` / `Suc m`); the sibling binder
   `w : Vec B n` in the surrounding context must be refined in lockstep so the
   inner match on `w` is exhaustive. Today it is not — inner match on `w` raises
   `ExhaustivenessError` (or, forced-arms, a kernel index `TypeMismatch`).

There is **no surface manual-convoy escape** to lean on instead: `match` has no
explicit-motive / `return` / `in` syntax (`ast.rs::Expr::EMatch` has no motive
field; `parser.rs::parse_match_expr`; §3.2 recovers the motive automatically with
no override). The automatic path is the only path, so this enhancement is the
capability — not sugar over an existing manual form.

This unblocks (per the DS-5 ruling): `tail : Vec A (Suc n) → Vec A n`,
`zip : Vec A n → Vec B n → Vec (Pair A B) n`, and `lookup : Vec A n → Fin n → A`
(its tail-recursion hits the **same** convoy path).

## Soundness bar (non-negotiable — the reason it routes through the Architect gate)

- **Injectivity discharged, never postulated.** The per-branch equation must be
  established from the kernel's own constructor no-confusion / `Elim`, **not**
  introduced as an `Axiom`/postulate. **Zero new `Axiom`, zero `trusted_base()`
  delta** — assert it with the DS-2-style executable before==after set-diff.
- **Kernel re-check stays the backstop, fail-closed.** The elaborated match must
  remain a term the kernel independently re-checks; an ill-typed convoy must
  **reject**, never slip through. Do not weaken a kernel check to accept the new
  shape — the enhancement produces honest terms, it does not relax the checker.
  (If you find yourself loosening a kernel-side check, that is the STOP signal.)
- **Refinement must be genuinely load-bearing, and must not OVER-refine (AC8
  discriminator).** Add a test that a branch which does **not** license an
  equation gets **no** spurious refinement — a wrong/over-asserted injectivity
  equation (e.g. refining a sibling that isn't index-linked, or equating
  distinct indices) must be **rejected**, asserted as the **specific** error
  variant (not bare `is_err()`), mirroring the landed "un-refined `head` rejects
  non-exhaustive" evidence. Over-refinement is the unsoundness vector here.
- **Termination / conv stress.** If the change touches conv/whnf or the motive
  solver, stress the recursive-inductive + function-typed case — a congruence/
  reduction arm can be sound yet open a non-termination (divergence ≠ unsoundness
  but is the operator's non-functional bar on the kernel path). Keep it total.

## Zero-regression bar (protects the whole tier, incl. in-flight DS-7)

- **The FULL pre-existing suite stays green** — not just new `Vec` tests. In
  particular the 14 `explicit_data_elaboration` tests **and every existing
  match-elaboration test**. A general fix to the shared match path can conflate
  similar-shaped-but-different cases, so run the whole suite, not a targeted
  subset.
- **Non-indexed matches must be INERT.** `List`/`Option`/`Bool` matches (no index
  to refine) must elaborate byte-identically — the new equation-in-context path
  must not fire when there is no index refinement. This is what keeps **DS-7**
  (`Applicative`/`Monad` proofs over `List`/`Option`, building in parallel on
  Foundation) safe: the two land additively on disjoint files (elab.rs + kernel
  tests vs catalog + elaborator tests) and must not interact. Demonstrate the
  inertness with a before/after on a representative non-indexed match.

## Spec fidelity (this is a spec + conformance touch — CI-gated, CV grounds it)

Update `spec/30-surface/34-data-match.md §3.2` to document the **extended** motive
recovery: peeled-recursive-field injectivity + sibling-binder convoy refinement,
with the totality consequences (`tail`/`zip`/`lookup` become well-typed). Keep it
faithful to what the elaborator actually does — spec and elaborator move
together. CV grounds the conformance-fixture implications (new positive cases for
`tail`/`zip`/`lookup`; the over-refinement negative). **CI-gated** (real
elaborator + kernel + conformance delta), not doc-only.

## Reversibility / boundary

- **Reversibility: hard-class** (a soundness-adjacent elaborator capability) —
  but revert-clean per the run's no-users reality; flagged for operator review
  regardless. Landing is sanctioned by the boundary rules **only because** it is
  the right fix at the right layer with the soundness gate intact — not because
  reverts are cheap.
- If the build discovers the gap is **not** where this frame locates it (needs a
  kernel change, or a `data.rs` change, or a surface-grammar motive syntax),
  **STOP and hand back** — the scope/altitude would be wrong and it's my re-call.

## Gate

Kernel ring: kernel-implementer build → kernel-leader re-derivation →
kernel-qa independent re-derivation (run the over-refinement discriminator + the
non-indexed-inertness check yourself) → **Architect soundness gate** (injectivity-
not-postulated, kernel-backstop-intact, zero-`Axiom`/zero-delta, full-suite-green)
→ `git_request` to Steward. CI-gated. Own retro. Resource discipline
(`CARGO_BUILD_JOBS=2`, scoped `-p` tests, never a bare/`--workspace` local run —
lean on CI for the full suite). Flag every surface/elaboration/functionality call
in the handback so the Steward logs it for the operator.
