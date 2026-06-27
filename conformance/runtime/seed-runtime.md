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

## runtime/capacity/loud-refusal (not silent)
- spec: `spec/40-runtime/44-capacity.md §2`
- given: exhausting the (engineering-chosen) capacity bound
- expect: a **loud, clear failure** — never silent drop/alias/corruption
- why: loud refusal over silent degradation, decoupled from Leech numerology.

## runtime/addressing/no-lattice-on-hot-path (oracle)
- spec: `spec/40-runtime/41-values.md §3`, `44 §4`
- given: the allocation/addressing path
- expect: addressing is hash (FNV-1a-style) + `memcmp`, slot ids a monotonic
  counter; **no** Leech quantizer / Co₀ canonicalization on the path
- why: the reality-check's correction — the lattice is not load-bearing for
  addressing.
