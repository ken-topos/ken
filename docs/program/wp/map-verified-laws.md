# map-verified-laws — prove the 5 deferred inductive Map laws (Map-arc CAPSTONE)

**Steward frame → spec enclave (proof-strategy elaboration + conformance flip +
discrepancy reconcile) → Foundation (build the proofs).** This is the **capstone**
the whole Map-container arc was for: with **both** enabling capabilities now
merged — **Gap A** (`surface-transport`, the `J` former + transport package,
`19955d8`) and **Gap B** (`dependent-match-nonnullary`, non-indexed dependent
match, `282856c`) — the 5 inductive Map laws that Map-build honestly deferred
become **provable**. Owner: **Foundation** builds the proofs; the **spec enclave**
elaborates the per-law proof strategy + the conformance flip first (hard
clean-room proof-engineering a ~1-yr-behind build model should not invent from
scratch — §2c). Gate: spec enclave elaboration → **Architect** (soundness: real
proof terms, zero `trusted_base` delta, kernel-checked) + **Spec review /
conformance-validator** (the seed flip is faithful) + Foundation QA + CI.
Findings → **Steward**.

Base: `origin/main`. Branch (pre-staged by Steward): **`wp/map-verified-laws`**.

## The locked inputs — DO NOT REOPEN

- **Both enabling capabilities are LANDED (verify at pickup, cite the merged
  code, not this line).** Gap A = `packages/transport/transport.ken` (5
  combinators over the `J` former) + `elab.rs::infer_j`; Gap B = the widened
  non-indexed dependent-match gate. The proofs are written **on top of** these —
  this WP builds **no new capability**, only the 5 proofs + their supporting
  list lemmas + the conformance reconcile.
- **`toList`-ordered is proved as the Ω `isSorted` form — NEVER as a
  permutation.** The permutation law (`toList` lists exactly the inserted entries
  once each) is **proof-relevant** (distinct interleavings = distinct
  derivations) → it **cannot** be `data Perm : Ω` directly
  ([[proof-relevant-inductive-cannot-be-declared-at-omega]]); it needs
  `‖Perm_rel‖` / count-equality and inherits the C5 prover gap. That law **stays
  deferred** (`52 §7c`, seed `tolist-permutation-law-deferred`) — **do not pull
  it into this WP.** Prove only the Ω-valued `isSorted` form (`52 §5.3`).
- **Zero `trusted_base` delta.** The 5 proofs reduce through the **existing**
  `Term::J`/`Term::Cast` (transport) + dependent-match `Term::Elim` — **no** new
  `declare_primitive`/`declare_postulate`, no `Axiom`. A law proved by an `Axiom`
  postulate is the exact anti-pattern the conformance net catches — a proof that
  cannot be built is **re-deferred to Steward**, never postulated.
- **Unbundled explicit-`leq` encoding** (the landed `map.ken` idiom, Architect-
  ruled): every op/law takes `leq`/`reflLeq`/`antisymLeq`/`transLeq`/`totalLeq`
  as **separate bare parameters** (the `C5-verified-sort` idiom). Do **not**
  reintroduce `where Ord k` (that spelling is `(oracle)`-deferred to a later
  Language surface WP). The core `lookup` laws lean on `refl`/`total`/`trans`
  only; **only** the overwrite/uniqueness face needs `antisym → Equal` (the
  ADR-0010 canonical-carrier dependency) — keep that face's canonicity dependency
  scoped and deferred, orthogonal to this buildability WP.

## ★ GROUNDED STATE (fixed inputs — elaborate against THIS, verify at pickup)

Grounded against `origin/main` (2026-07-03, scout `a44388f9`; this worktree has
only stubs — cite via `git show origin/main:<path>`). **Line numbers are
perishable — verify at pickup, do not trust this frame over the landed code.**

**The 5 deferred laws (verbatim, `spec/50-stdlib/52-map.md` §5.1–5.3, §7d):**

| # | Law | Statement | Gap-tag |
|---|-----|-----------|---------|
| 1 | Ordered-preservation (§5.1) | `Ordered m ⇒ Ordered (insert k v m)` | A + B |
| 2 | found-after-insert (§5.2) | `lookup k (insert k v m) = Some v` (found-branch by `refl`, no `antisym`) | A + B |
| 3 | locality (§5.2) | `distinct k k' ⇒ lookup k' (insert k v m) = lookup k' m`, `distinct k k' := ¬(IsTrue (leq k k') ∧ IsTrue (leq k' k))` | A + B |
| 4 | `toList`-ordered (§5.3, load-bearing) | `Ordered m ⇒ isSorted (λ a b. leq (fst a) (fst b)) (toList m)` — **comparison-free** (`toList` never calls `leq`) | **B only** |
| 5 | agreement (§5.3) | `lookup k m = assoc k (toList m)` — aligns `lookup`'s comparison descent against the ordered list | A + B |

**Law 4 additionally needs two list lemmas** (`52 §5.3`): `isSorted`-over-`++`,
and `allKeys ↔ allInList (toList)`. These are Gap-B-only (structural, no `leq`).

**`packages/collections/map.ken` current state (162 lines):** carrier `data
Tree k v = Leaf | Node (Tree k v) k v (Tree k v)`; ops `empty`/`toList`/`fold`/
`insert`/`lookup`/`member`/`fromList` + Set ops; Ω-prop `view`s `allKeys`,
`Ordered` (constant-motive, prover unfolds, never postulates). **Only ONE
non-inductive law is actually present: `lookupEmptyIsNone` (`:161-162`,
`= tt`).** The 5 inductive laws are **ABSENT** (not stubbed, no `Axiom`).

**⚠ DISCREPANCY 1 — RECONCILE (enclave, spec-author owns `52-map.md`):** the spec
(`52 §7`/§8) and seed (`seed-map.md`) both assert **TWO** shipped Branch-A proofs
(`Ordered empty` **and** `lookup empty`), but `map.ken` contains **only**
`lookupEmptyIsNone` — a `Ordered empty` / `orderedEmpty` proof term is **not in
the file**. Either (a) add `Ordered empty` as an additional (trivial, non-
inductive: `Ordered Leaf ⇝ ⊤` by `tt`) Branch-A proof in this WP, **or** (b)
correct the spec/seed narrative. Recommend (a) — it is a one-line `tt` proof and
makes the "two Branch-A proofs" narrative true. Resolve explicitly; do not ship
with the narrative ahead of the code.

**Transport idiom (Gap A — the proofs MIRROR this).** `packages/transport/
transport.ken` ships `subst`/`cong`/`cast`/`sym`/`trans` (lowercase type params;
equality spelled `Eq ty x y`, base `Refl`). The `J` former:
`J <motive> <base> <eq>`, motive a 2-arg lambda `\b' e'. G[b']`, infer-mode,
codomain sort **unconstrained** (admits Ω motives — `elab.rs::infer_j`). **The
exact idiom for firing a stuck `if leq k k'` / stuck `match` via an order
hypothesis** (`surface_transport_acceptance.rs` AC3 + `53 §3`):
`proof = J (\x _. G[x]) (pf : G[True]) (sym q)` where `q : Eq Bool (leq k k')
True`. **Base is `tt`** (not `Refl`) when the reduced goal is a `⊤`-shaped
`IsTrue`; use `sym` when the hypothesis is in its natural orientation. This is
how laws 1,2,3,5 discharge their stuck-comparison obligations at Ω — **no
truncation** (the `J` Ω-motive suffices; a sort-polymorphic `subst` is NOT needed
and NOT provided).

**Dependent-match idiom (Gap B — induct on `Tree`/`List`).** Gate is **widened,
non-indexed** — `elab.rs` `dependent_eligible` at **`:535-553`** (`ind.indices
.is_empty()`), **NOT** the stale nullary `:455` the spec/seed still cite
(**⚠ DISCREPANCY 2 — reconcile the `:455` citation to `:535-553` in `52`/seed**).
Idiom (`dependent_match_nonnullary_acceptance.rs` AC1): `match t { Leaf => … ;
Node l k r => \h. … }` where the arm binds the **per-branch-narrowed** hypothesis
`\h.` (`h`'s domain narrows to the motive at the reconstructed constructor); a
`Node` gives **two IH slots**. Scope: **non-indexed `List`/`Tree` only** — indexed
families/GADTs are excluded (a distinct later WP).

**Conformance seed to flip (`conformance/stdlib/map/seed-map.md`):**
- `map-verified-laws-deferred (soundness)` (`:482-505`): currently pins the 5 laws
  **absent-not-`Axiom`**, per-law gap-tagged. **FLIP → proven:** the 5 become real
  proof terms in `map.ken`, still **zero `trusted_base` delta**; the absence-pin
  inverts to a presence-pin.
- `no-shipped-code-leans-on-a-deferred-law (soundness)` (`:316-340`): add the
  **consumer-delta-flip variant** — a consumer that type-checks **using** a real
  correctness law (found-after-insert or agreement), whose reduction an `Axiom`
  would short-circuit → the true **proved-vs-`Axiom` value-flip** the Branch-A
  green-vs-green cases could not deliver.
- **STAYS deferred (do NOT touch):** `tolist-permutation-law-deferred` (proof-
  relevant, C5 gap). `tolist-ascending-by-key`'s *proof* un-defers with law 4.

## Mandated deliverable outline (each item → a concrete choice)

1. **The 5 law proofs in `map.ken`**, each a real proof term (zero `trusted_base`
   delta), by induction on `Tree` (dependent-match, Gap B) + transport of stuck
   `leq`-comparisons via order hypotheses (`J`/`sym`, Gap A). Enclave elaborates a
   **per-law proof skeleton** (induction target, the transport at each stuck
   comparison, the base witness `tt`-vs-`Refl`); Foundation fills it in.
2. **The two list lemmas** for law 4 (`isSorted`-over-`++`, `allKeys ↔ allInList
   (toList)`) — structural, Gap-B-only.
3. **Discrepancy reconciles:** (1) add `Ordered empty` (`tt`) or correct the
   spec/seed "two Branch-A proofs" narrative; (2) fix the stale `:455`→`:535-553`
   dependent-match gate citation in `52`/seed.
4. **Conformance flip:** `map-verified-laws-deferred` absent→proven; add the
   `no-shipped-code-leans-on-a-deferred-law` **consumer-delta-flip** case. Leave
   `tolist-permutation-law-deferred` deferred.
5. **Ω-discipline statement:** `toList`-ordered is the Ω `isSorted` form; the
   proofs never reduce a stuck boolean by fiat — every stuck `leq` is discharged
   by a transported order hypothesis (which is why law 4 clears Gap A: it is
   comparison-free and needs only Gap B).

## Acceptance criteria

- **AC1 — the 5 laws are REAL, kernel-checked proof terms.** Each of laws 1–5 is
  a proof term in `map.ken` that the kernel accepts (whole-declaration `check`,
  `check.rs:984`, the true soundness net) — **not** an `Axiom`. A discriminating
  negative: replacing any proof with an `Axiom` postulate is caught by the
  conformance `trusted_base`-delta net (it would grow the cone).
- **AC2 — ZERO `trusted_base` delta (load-bearing).** `trusted_base()` unchanged;
  no new `declare_primitive`/`declare_postulate`/`Decl` variant; the proofs reduce
  through existing `J`/`Cast`/`Elim`. **Grep the diff:** zero `crates/ken-kernel/`
  touched. Set-equality before/after loading the package (mirror the transport
  package's own `*_adds_zero_trusted_base_delta` test).
- **AC3 — `toList`-ordered is the Ω `isSorted` form, permutation UNTOUCHED.** Law
  4 proves `Ordered m ⇒ isSorted … (toList m)` at Ω; grep confirms **no** `Perm`
  inductive, no `‖·‖` truncation, no attempt at the permutation law. It stays a
  named deferred follow-on.
- **AC4 — conformance flipped faithfully.** `map-verified-laws-deferred` now pins
  the 5 laws **present + proven, zero-delta** (was absent); the new consumer-delta
  case exercises a real correctness law with a **discriminating** proved-vs-`Axiom`
  flip. CV/Spec review confirms no over-claim (a law not actually built stays
  deferred, honestly).
- **AC5 — discrepancies reconciled.** The "two Branch-A proofs" narrative matches
  the code (either `Ordered empty` is added, or the narrative is corrected); the
  dependent-match gate citation is current (`:535-553`).
- **AC6 — no regression.** `cargo test --workspace` green; `ken-cli` rosetta green.
  Every prior proof still checks (monotone — the WP only **adds** proofs).

## Guardrails (do-not-reopen)

- **Ω `isSorted` only for `toList`-ordered — NEVER permutation** (proof-relevant,
  C5 gap, stays deferred).
- **Zero `trusted_base` delta; no `Axiom`.** A law that cannot be honestly built
  is **re-deferred to Steward**, never postulated — the honest-ceiling discipline
  that kept Map-build correct.
- **Unbundled explicit-`leq` encoding** — no `where Ord k` reintroduction.
- **Keep the canonicity (`antisym → Equal`) overwrite/uniqueness face scoped +
  deferred** — orthogonal to this buildability WP (ADR-0010).
- **Kernel / `trusted_base` off-limits** — outer-ring proofs over the existing
  trusted formers; kernel untouched.

## Sequencing

- **Gate:** touches `/conformance` (seed flip) + spec reconciles + hard proof
  design → **spec enclave elaborates** the per-law proof strategy + conformance
  flip + discrepancy reconcile on this WP branch (spec-leader may judge the frame
  sufficient and pass straight to Foundation, or elaborate a proof skeleton
  first — their call, given proof difficulty). Merges to `main` via the
  Integrator, then **Foundation** builds the proofs. **Architect** (soundness:
  real proof terms, zero delta, kernel-checked) + **Spec review / CV** (seed flip
  faithful) + Foundation QA + CI.
- **Lane:** Foundation (proofs in `map.ken`) + conformance (`seed-map.md`) +
  spec reconcile (`52-map.md`). Branch off `origin/main`.
- **Priority vs `[FS]`:** this capstone is **higher priority** than `FS-driver`
  for the (serial) enclave — sequence `map-verified-laws` proof-strategy
  elaboration **first**, `[FS]` after.
- **Closes the Map-container arc:** deliverable-1 (Map-build) + the two
  buildability errata + Gap A + Gap B + this = the full Map story, spec ↔
  conformance ↔ build consistent, the honest ceiling raised from 2 non-inductive
  laws to the full 7.
