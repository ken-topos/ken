# KNOWN-GAP: a constructor with 2+ fields of its own recursive type cannot
# be `match`ed

## What's missing

A `data` type whose constructor has **two or more fields of the type being
declared itself** — the ordinary shape of any binary tree — fails to
type-check when `match`ed, even in the most trivial form (constant-value
arms, no traversal logic at all).

## Confirmed empirically

A scratch bisection (`crates/ken-elaborator/tests/zzdebug.rs`, disposable),
fully isolated from this example's own logic:

```ken
data T7  = L7  | N7  T7          -- 1 recursive field    -- match: OK
data T8  = L8  | N8  Nat T8      -- 1 recursive + 1 other -- match: OK
data T10 = L10 | N10 T10 T10     -- 2 recursive fields    -- match: FAILS
data T9  = L9  | N9  T9 T9 T9    -- 3 recursive fields    -- match: FAILS (same error)
```

All four used the identical trivial shape
`match t { <nullary-ctor> => Zero ; <n-ary ctor with binders> => Zero } : Nat`.
The only variable across the four is the count of same-typed recursive
fields on one constructor. This is not "user recursive data is broken" —
`data MyList a = MyNil | MyCons a (MyList a)` (a from-scratch clone of
`List`, one recursive field) elaborates and `match`es fine. The trigger is
specifically **≥2 fields of the same recursive type in one constructor**.

The failure (`T10`, minimal repro):
```
KernelRejected { error: TypeMismatch {
  expected: (((λ Dg455. Dg70) : (Π Dg455. Type 0)) @1),
  found: (Π Dg70. Dg70)
}, ... }
```
— the computed "expected" motive/type for the match is a bare one-argument
lambda, while the actual constructor arity (2+ recursive fields) needs a
longer Pi-chain; the two never line up.

## Impact (broader than this example)

This blocks every ordinary binary tree, binary expression AST (two operand
fields), or any n-ary tree with ≥2 same-typed child fields — not scoped to
`tree-traversal` specifically. `List`/`Nat` (single recursive field each,
built into the prelude) are unaffected, which is why every other landed
example (using only `List`/`Nat`) never tripped this.

## Fix needed (capability, not a Language-lane workaround)

Root cause is in the elaborator's dependent-match motive computation for
user-declared inductives (`crates/ken-elaborator/src/elab.rs`) — likely a
context-extension or de-Bruijn-indexing defect when a constructor's field
list contains the recursive type more than once. Root-cause detail
dispatched to a research agent and routed to language-leader/Steward
separately from this dir's finding. This is squarely the VAL2 frame's "gap
whose fix needs a new capability" boundary — not something to route around
inside this light-gated mini-WP (there's no Ken-surface workaround: the
motive computation itself is defective, not a missing library feature).

## Intended program (once resolved)

See the commented-out definitions in `tree-traversal.ken` — an in-order
traversal of a small BST over `Char`, folded to a `"PASS"`/`"FAIL"` oracle
string (steers around the separately-tracked `natToDecimal` gap, matching
`palindrome`'s and `closures`'s oracle style).
