# C5 — verified `sort` (`isSorted ∧ Perm`, `Perm` at the right universe)

**Axis:** proof-carrying / verified programs. **Flavor:** B (should-PASS on
emission; full discharge is a known-gap). **Coupled with C2** — the `Perm`
conjunct must sit at the universe C2 establishes (`‖Perm‖` / count-equality,
never a proof-relevant Ω inductive).

## Why this is a blind spot

VAL2's `merge-sort` was an ordinary `Ord` sort with **no verification**. Ken's
distinctive value is that a `sort` can carry its own correctness: the result
type is the refinement `{ ys : List a | isSorted leq ys ∧ Perm ys xs }`, and the
elaboration **emits the conjoined obligation**. The load-bearing subtlety: the
`Perm` conjunct is what forces `sort` to *be* a sort — `isSorted`-alone is
**vacuous** (`const Nil` satisfies "the output is sorted"; the empty list is
sorted). Dropping `Perm` is a verification-soundness omission the kernel does
not catch.

## The pair

- **Sound arm — `sound-verified-sort.ken` — should-PASS (emission).** The
  explicit-comparator `sort (a) (leq) (xs) : { ys : List a | And (isSorted a leq
  ys) (Perm a ys xs) }` (the landed ES2 form). The result-introduction emits
  the conjoined obligation carrying **both** conjuncts. (Grounded: this exact
  view type-checks in `es2_acceptance.rs`; `isSorted`/`Perm` are real defs that
  unfold, not postulates.)
- **Unsound/stub arm — `unsound-const-nil.ken` — should-REJECT.** `sortBad _ _
  _ = Nil`, claiming the **same** refinement. `isSorted leq Nil` holds
  vacuously, but `Perm Nil xs` is **false** for non-empty `xs`, so the emitted
  obligation is **unprovable**.

## Expected behavior (exact)

- Sound arm: **PASS on emission** — the refinement elaborates and emits `And
  (isSorted a leq (sort …)) (Perm a (sort …) xs)`, **both conjuncts present**.
  Full **proof discharge** (the prover closing `Perm (insert …) …`) is a
  **known-gap** — the verified-sort proof term / prover discharge is not fully
  landed; emission-with-both-conjuncts is the checkable property today.
- Unsound arm: **should-REJECT** — the `Perm a Nil xs` conjunct cannot be
  discharged for non-empty `xs` (`count`-mismatch / no permutation witness).
  **If `sortBad` is ACCEPTED at the refinement, that is the finding** — either
  `Perm`
  was dropped from the emitted obligation (the untrusted-layer omission, reads
  `proved`-by-default) or the obligation is not enforced.

## Discriminates

Is the **`Perm` conjunct present and enforced**? The real `sort` (both
conjuncts, PASS) vs `const Nil` (isSorted-only satisfiable, `Perm` must fail) is
the flip. A `sort` test that asserted only `isSorted` would be green-vs-green —
`const Nil` passes it. This is the refinement-completeness discriminator,
promoted from the `sort-emits-issorted-and-perm` seed to a full verified
program.

## Surface-expressibility note

The refinement form `{ ys : List a | And (isSorted a leq ys) (Perm a ys xs) }`
is landed surface (`es2_acceptance.rs`). Whether the emitted obligation is
**discharged** (vs merely emitted) depends on the prover; if discharge isn't
reachable, record "emits both conjuncts; discharge deferred" — the emission
completeness (both conjuncts, `Perm` present) is the load-bearing check either
way. `Perm` must be the C2 `‖Perm‖`/count-equality form, not a proof-relevant Ω
inductive.
