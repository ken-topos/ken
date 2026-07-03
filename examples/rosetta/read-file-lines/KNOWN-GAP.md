# KNOWN-GAP: GAP-fs-read-unwired — `read_bytes` has no real runtime
# reduction

## What's missing

`read_bytes : Bytes -> Bytes` is declared with a `[FS]` effect row
(`crates/ken-elaborator/src/bytes.rs`) but `ken-interp` has no actual file
I/O implementation for it. `crates/ken-interp/src/eval.rs`'s `apply`
special-cases exactly three primitives with real reductions —
`print_line`, `string_to_list_char`, `list_char_to_string` (~1275-1309) —
`read_bytes`/`write_bytes` fall through to the generic `prim_reduce`
catch-all, which stays `Neutral` (stuck) for any unrecognized symbol.
Calling `read_bytes` on any input never actually reads a file; it's stuck
forever.

## Impact

No Ken program can perform real file I/O today. This blocks not just this
task but any future example needing to read external input.

## Fix needed (capability, not a Language-lane workaround)

Wire a real `read_bytes` reduction in `ken-interp` (actual filesystem
read, `[FS]`-effect-gated) — this is `ken-interp`'s reduction surface
(Runtime's lane per the effect-row wiring precedent, `print_line`'s own
landing), not something Language can add from the surface/elaborator
side. Routed to language-leader / Steward as its own capability WP.

## Intended program (once resolved)

Read a file's bytes via `read_bytes`, decode to `String` (`bytes_decode`,
already declared), split on newlines (needs a `splitOn`/`lines` helper —
not yet built, would ride the landed `packages/collections` floor once
this gap closes), print each line via the already-working `printAll`-
style IO combinator (VAL2's `printLoop` probe generalizes to `List
String`).
