# conv-eq-congruence — add the `(Eq, Eq)` congruence arm to `conv_struct` (Map capstone Gap-conv)

**Steward frame → Kernel build. Fix vector CONFIRMED + soundness-gated by the
Architect** (`evt_2s3brvnr2cta2`, thread `thr_2ttmjf6848y7j`, 2026-07-03).
Kernel **conversion-completeness** bugfix — no new capability, no `/spec` change,
no new mechanism, no trust-root growth. Owner: **Kernel** (implementer + QA).
Gate: **Architect** (soundness — he will cast on the real candidate) +
**kernel-qa** + CI. Findings → **Steward**.

## Why this WP exists

The Map-arc capstone's **law 4 conclusion** (`consSorted` / `isSortedAppend` /
`toListOrdered`) is blocked by a **real, general kernel conversion-completeness
gap**: two `Eq` *types* that differ only by a **reducible sub-term** fail to
convert. `isSorted`'s comparator is **pair-indexed** (`pairLeq : Pair k v ->
Pair k v -> Bool`) while the bound-fact chain (`Ordered`/`allKeys`/`allInList`)
is **key-indexed** (`leAbove : k -> Prop`); these are convertible only by
unfolding **both** down to `leq (pairFst m) (pairFst y)` — a delta step **inside
an `Eq` argument** that conversion currently refuses. No proof re-spelling bridges
a `Pair`-comparator and a `k`-predicate without hitting this (grounded:
foundation-implementer `evt_7nw6pahg6rc3a`, both attempts forced-mismatch). This
is the **third and last real gap** of the capstone (Wall 2 dissolved into the
convoy idiom; Wall 1 / `infer_j` is a separate Unit-2 WP).

## The bug (root cause — grounded by the Architect on `origin/main@f11c61d`)

`conv_struct` (`crates/ken-kernel/src/conv.rs`) has **no `(Term::Eq, Term::Eq)`
congruence arm.** Its arms cover
`Type/Var/Pi/Lam/Sigma/Pair/App/Proj/Const/Constructor/Elim/QuotElim/Ascript/Absurd`,
then `_ => false`. So two `Eq A a b` / `Eq A' a' b'` **types** that differ only by
a reducible sub-term (which `whnf`'s `Eq` arm leaves **stuck** when `eq_reduce` is
neutral) fall straight through to `false`. The syntactic-identity early-return
saves only the *identical-spelling* case; anything needing a delta step inside an
`Eq` argument (a comparator/view-application — exactly `pairLeq`/`leAbove`) is
**over-rejected** (fail-closed).

## The fix (vector CONFIRMED — do not redesign)

Add the missing congruence arm, **before `_ => false`**, in `conv_struct`
(`crates/ken-kernel/src/conv.rs`):

```rust
(Term::Eq(ty1, a1, b1), Term::Eq(ty2, a2, b2)) => {
    conv_struct(env, ctx, ty1, ty2)
        && conv_struct(env, ctx, a1, a2)
        && conv_struct(env, ctx, b1, b2)
}
```

**Why this is sound (the Architect's ruling — the posture you must preserve):** it
is the **correct congruence rule** for the `Eq` type-former — true **iff** all
three components are convertible, recursively via the same sound `conv_struct`. It
is the **missing congruence closure, not a loosening**: it can only recognise
*more true* equalities, never accept a *false* one. Proof-irrelevance is
untouched — Ω-typed **proofs** never reach this arm (`convert`'s `is_omega_type`
shortcut fires first); it compares `Eq` **types** only. Termination is the same
structural-descent-under-`whnf` pattern as the existing `App`/`Pi`/`Sigma` arms.
The direction is **fail-closed** (completeness only), so there is **zero soundness
pressure** — but the ACs still make you *prove* non-over-conversion, because a
conv change's one real hazard is accepting a false equality.

## Acceptance criteria (all testable; the Architect has already run AC3+AC4 once)

- **AC1 — exactly the congruence closure.** The arm recurses via `conv_struct` on
  `(ty, a, b)` — **no** ad-hoc unfold, **no** loosening of any other arm, placed
  **before** `_ => false`. Grep-verifiable. Do **not** touch `whnf`'s `Eq` arm or
  `eq_reduce`.
- **AC2 — kernel-conv-only diff.** `git diff origin/main` touches
  **`crates/ken-kernel/src/conv.rs` + kernel test files ONLY** — no new
  `declare_*` / `Term` variant / kernel decl; `trusted_base()` unchanged; no other
  crate. The TCB's *power* does not grow (no new inhabitant becomes well-typed that
  was not already def-equal); only conversion *completeness* increases. Verify by
  grep.
- **AC3 — the discriminating isolation-flip.** Land the `cTest` repro as a kernel
  test: `h : Equal Bool (idB x) True` (where `idB b = b`) coerced to
  `Equal Bool x True` — **FAILS pre-fix, PASSES post-fix** under **only** this arm.
- **AC4 — over-conversion gate (the soundness net — the real hazard).** (i) full
  `ken-kernel` suite green (Architect got **161/161**) — no (in)equality or
  proof-irrelevance test regresses; (ii) a **discriminating negative** stays
  **REJECTED**: `Equal Bool (idB x) True` vs `Equal Bool x **False**` (different
  endpoint) must **not** convert. Run `cargo test --workspace` before shipping.
- **AC5 — `Eq` arm ONLY (scope).** Do **NOT** add `Cast`/`J`/`Quot`/`QuotClass`/
  `Trunc` congruence arms in this PR. They share the latent `_ => false`
  completeness class but **none is demonstrated to bite** (Cast computes from
  endpoints, `J`→`Cast`), and Architect's explicit call is to **ship the `Eq` arm
  now** + audit the siblings as a **separate fast follow-on**, not a speculative
  same-PR expansion.

## Guardrails (do-not-reopen)

- **The vector is settled** (Architect `evt_2s3brvnr2cta2`, verified before ruling
  — isolation-flip + 161/161 + discriminating-negative). Add the congruence arm as
  written; do not redesign, do not "also fix" `whnf`/`eq_reduce`, do not expand to
  sibling formers.
- **Completeness, not soundness.** If anything here tempts a change that would make
  conversion accept *more* than the congruence closure (i.e. a genuinely non-def-eq
  pair), STOP and raise to Steward — that is the over-conversion hazard AC4 exists
  to forbid.

## Sequencing

- Branch `wp/conv-eq-congruence` off `origin/main`, frame committed with it (rides
  with the build — frame + fix merge together, no separate frame-merge). Gate on
  merge: **Architect** (soundness vote on the real candidate — he pre-verified the
  vector, so this is confirming the *landed* diff matches) + **kernel-qa** + CI.
  **Parallel-gate**: once the diff is confirmed `conv.rs`-only, mention Architect +
  kernel-qa together (the sct-(a)/(b) precedent — fast clean convergence).
- **On merge:** Steward signals Foundation to resume `map-verified-laws` on the
  held branch (rebased onto the new `main`) and build **law 4's conclusion**
  (`consSorted`/`isSortedAppend`/`toListOrdered`) — now unblocked. Laws 1/2/3/5
  still gate on the separate Wall-1 `infer_j` WP.
- **Fast follow-on (separate WP, not this one):** audit `Cast`/`J`/`Quot`/
  `QuotClass`/`Trunc` for parallel congruence arms (same latent completeness
  class). Steward tracks; not blocking the capstone.
- **Lane:** Kernel (`ken-kernel/conv.rs`). Same reviewer set + no-`/spec`-change
  profile as `sct-completeness` (a)/(b).
