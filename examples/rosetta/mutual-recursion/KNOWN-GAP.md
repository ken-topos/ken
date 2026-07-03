# KNOWN-GAP: mutual recursion has no surface expression

## What's missing

Ken's surface has no way to declare two (or more) `view`s that call each
other. `crates/ken-elaborator/src/lib.rs`'s `elaborate_file` elaborates
`view` declarations strictly in source order, resolving names against
whatever has already been registered — there is no `mutual`/forward-declare
construct, and no code path gathers multiple *separately-named* `view`s into
one `declare_recursive_group` call (that function is only invoked internally,
for a single self-recursive `view` or for class-instance elaboration; see
`crates/ken-elaborator/src/elab.rs:1782` `elaborate_recursive_view` and
`:1489`/`:1514` in the class-instance path — neither takes a *group* of
user-named top-level `view`s).

## Confirmed empirically

A scratch probe (`crates/ken-elaborator/tests/zzdebug.rs`, disposable)
declaring the classic even/odd pair:

```ken
view isEven (n : Nat) : Bool = match n { Zero => True ; Suc m => isOdd m }
view isOdd (n : Nat) : Bool = match n { Zero => False ; Suc m => isEven m }
```

fails elaborating `isEven` (declared first) with
`UnresolvedCon { name: "isOdd" }` — `isOdd` isn't in scope yet. Declaring
`isOdd` first fails symmetrically on `isEven`. There is no source order that
elaborates both.

## What broke / why the encoding is awkward

This isn't a workaround-able ordering problem — it's a genuine missing
capability: some mechanism to (a) pre-admit both names (e.g. as `Opaque`,
mirroring how a single self-recursive `view` pre-admits itself before
elaborating its body — `elab.rs:1896-1919`), (b) elaborate both bodies with
both names in scope, then (c) route the whole group through one
`declare_recursive_group` call so `sct_check` can verify termination across
the mutual cycle (presumably via a lexicographic or multi-function descent
measure, similar to the Ackermann gap's need for lexicographic termination —
`examples/rosetta/ackermann/KNOWN-GAP.md`). None of that machinery is wired
to the surface today.

## Fix needed (capability, not a Language-lane patch)

A `mutual`-block (or equivalent forward-declaration) surface construct in
`ken-elaborator`'s parser + elaborator, wiring the whole group through
`declare_recursive_group` with a termination measure that spans the group.
This is squarely the VAL2 frame's "gap whose fix needs a new capability"
boundary (`docs/program/wp/VAL2-rosetta-pangram.md` §"gap → capability
boundary") — routed to language-leader / Steward as its own finding, not
patched inside this light-gated mini-WP.

## Intended program (once resolved)

See the commented-out definitions in `mutual-recursion.ken` — classic
even/odd over `Nat`, folded to a single `"PASS"`/`"FAIL"` oracle string
(steers around the separately-tracked `natToDecimal` exponential-blowup
finding, matching `palindrome`'s oracle style).
