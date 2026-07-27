# Runtime conformance — seed cases

Format: `../README.md`. These pin the runtime corrections (typed values, not
uniform f64), durable canonical bytes for closure-free canonical data, private
runtime representation, and the runtime-local opaque callable boundary.

## runtime/values/equal-canonical-values-same-durable-bytes
- spec: `spec/40-runtime/41-values.md §2,§4`
- given: two independently-constructed structurally-equal closure-free
  canonical compound values
- expect: they compare equal and produce identical durable canonical bytes;
  copying, sharing, and interning are not observed
- why: durable canonicity and extensional equality are independent of storage.
  Ordinary closures and aggregates containing them remain outside this
  canonical contract; `values/README.md` pins that boundary.

## runtime/values/scalars-retain-distinct-types
- spec: `spec/40-runtime/41-values.md §1`
- given: an `Int`, a `Bool`, a `Float`
- expect: each retains its declared type and behavior, not a uniform `f64`;
  boxing and immediacy are private
- why: the "every value is an f64" model is not Ken's.

## runtime/values/int-small-to-bignum
- spec: `spec/40-runtime/41-values.md §1`, `35 §1`
- given: an `Int` computation that grows past a machine word
- expect: the value stays exact and preserves its durable canonical bytes;
  the physical promotion form is not observed
- why: arbitrary-precision `Int` permits a private small-integer fast path.

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
> loud refusal (`CapacityExhausted`, never silent), profile-declared
> accounting, semantics-invisible reclamation, and no semantic lattice
> dependency — are now pinned at X2 rigor in
> `capacity/seed-capacity.md` (grounded on `44` + the landed `store.rs`). The
> two cases that lived here (`runtime/capacity/loud-refusal`,
> `runtime/addressing/no-lattice-on-hot-path`) are **subsumed** there (one home
> per property); see that file for AC1–AC5.
