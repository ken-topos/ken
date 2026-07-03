# KNOWN-GAP: GAP-map-no-ops — `Map` is a bare type with no operations

## What's missing

`Map : Type -> Type -> Type` is declared in the prelude
(`crates/ken-elaborator/src/prelude.rs`) as an audited opaque primitive
(correctly not derivable — genuinely needs a runtime O(1)
content-addressed canonical form, per its own doc comment, same trust
tier as `String`/`Bytes`). But **no operations exist** for it anywhere in
`ken-elaborator` or either landed package (`packages/collections`,
`packages/lawful-classes`): no `empty`, `insert`, `lookup`, `toList`, or
any naming variant. A type with no constructor and no way to query a
value of it is completely unusable from Ken surface code.

## Impact

Any task needing key-value association (letter/word frequency counts,
memoization tables, symbol tables, ...) has no path today. This is not a
`List`-based-encoding-is-awkward problem — there is genuinely nothing to
call.

## Fix needed (capability, not a Language-lane workaround)

Wire the actual `Map` operations as primitives (mirroring how `String`/
`Bytes` get their ops: `declare_primitive` + `PrimReduction`, real
`ken-interp` reductions for `empty`/`insert`/`lookup`/`toList` at
minimum). This is exactly the same shape of gap as `read_bytes` (a
primitive TYPE is declared/audited but the operations that make it usable
are never wired) — routed to language-leader / Steward as its own
capability WP, likely paired with (or informing) any future stdlib `Map`
package once the primitive ops land.

## Intended program (once resolved)

Fold over the input string's `List Char` (via `string_to_list_char`),
threading a `Map Char Nat` accumulator (`insert`-or-increment per char),
then `toList` + `packages/collections`' `list_compare`/`compareChar` to
produce a sorted frequency report.
