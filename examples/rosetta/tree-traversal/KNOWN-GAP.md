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

Root cause pinned precisely (research dispatched, confirmed): in
`crates/ken-elaborator/src/elab.rs`, `compile_match_matrix`'s `ColKind::Ih`
branch (2364-2394), via the `tail_codomain` helper (2296-2322). A
constructor with 2+ recursive fields produces one "Ih" (induction-
hypothesis) column per recursive field, laid out as flat siblings in the
pattern-matrix (`build_ctor_buckets`, 2515-2597 — e.g. `[Real, Real, Ih,
Ih]` for `N10 x y`). The bug: each Ih slot's type is computed by folding
*everything still pending after that column* via `tail_codomain` — which,
with 2+ Ih columns, wrongly includes the *next sibling* Ih column as if it
were an outer split's genuine continuation. `tail_codomain`'s fold-every-
remaining-column behavior is correct for that outer-continuation case
(its actual intended use, at the split-column call site, 2475-2480) but
wrong here: it over-builds the first Ih's type as an extra Pi/arrow layer
around what should be the plain, non-arrow `ret_ty`. That's exactly the
kernel `TypeMismatch` above (`expected` a bare lambda, `found` a Pi). A
scope-confusion bug, not de-Bruijn shifting or type-deduplication: the
code has no way to distinguish "my own constructor's next Ih sibling"
from "the enclosing match's real pending tail," and always folds them
together. With 0-1 recursive fields there's no sibling Ih to wrongly
sweep up, which is why `List`/`MyList`-shaped types are unaffected.

Candidate fix (not attempted — outside this light-gated mini-WP's scope):
the `ColKind::Ih` branch should compute `ih_ty` as plain
`weaken(ret_ty, real_depth_so_far)` when the pending tail is itself
sibling Ih columns from the same constructor, reserving `tail_codomain`'s
full-fold behavior for a genuinely-outer pending split. Routed to
language-leader/Steward as its own capability WP (VAL2 frame's "gap whose
fix needs a new capability" boundary) — there's no Ken-surface workaround,
the motive computation itself is defective, not a missing library
feature.

## Intended program (once resolved)

See the commented-out definitions in `tree-traversal.ken` — an in-order
traversal of a small BST over `Char`, folded to a `"PASS"`/`"FAIL"` oracle
string (steers around the separately-tracked `natToDecimal` gap, matching
`palindrome`'s and `closures`'s oracle style).
