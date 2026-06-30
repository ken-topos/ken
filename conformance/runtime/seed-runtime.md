# Runtime conformance — seed cases

Format: `../README.md`. These pin the runtime corrections (typed values, not
uniform f64; conventional content addressing) and the content-addressed
properties.

## runtime/values/dedup-shares-slot
- spec: `spec/40-runtime/41-values.md §2,§4`
- given: two independently-constructed structurally-equal compound values
- expect: they occupy the **same slot** (global dedup); `==` is O(1) (slot-id
  comparison)
- why: content addressing gives sharing + O(1) structural equality.

## runtime/values/scalars-are-typed-immediates
- spec: `spec/40-runtime/41-values.md §1`
- given: an `Int`, a `Bool`, a `Float`
- expect: each is an **unboxed typed immediate** (machine word / `i1` / `f64`),
  not routed through a uniform `f64` nor a heap slot
- why: the "every value is an f64" model is not Ken's.

## runtime/values/int-small-to-bignum
- spec: `spec/40-runtime/41-values.md §1`, `35 §1`
- given: an `Int` computation that grows past a machine word
- expect: transparent promotion to a heap bignum; value stays exact
- why: arbitrary-precision `Int` with a small-int fast path.

## runtime/evaluation/canonicity
- spec: `spec/40-runtime/42-evaluation.md §1`
- given: a closed computation of an inductive (or a closed `Eq`/`cast` op)
- expect: **reduces** to a constructor form (resp. computes); no closed
  well-typed ground program gets stuck
- why: canonicity (a soundness commitment).

## runtime/evaluation/unknown-propagates
- spec: `spec/40-runtime/41-values.md §6`, `42 §4`
- given: a value depending on an open verification hole, combined via `∧`/`∨`
- expect: `unknown ∧ false = false`, `unknown ∨ true = true`, else `unknown`;
  the program **runs**
- why: partial verification runs and marks where the gap bites.

> **The capacity / addressing cases moved.** The store-capacity commitments —
> loud refusal (`CapacityExhausted`, never silent), dedup-aware accounting,
> reclamation, and `no-lattice-on-hot-path` — are now pinned at X2 rigor in
> `capacity/seed-capacity.md` (grounded on `44` + the landed `store.rs`). The
> two cases that lived here (`runtime/capacity/loud-refusal`,
> `runtime/addressing/no-lattice-on-hot-path`) are **subsumed** there (one home
> per property); see that file for AC1–AC5.
