# KNOWN-GAP: read-file-lines — example not yet re-authored to the new `[FS]` substrate

## What changed

`GAP-fs-read-unwired` (`read_bytes` had no runtime reduction) is **CLOSED** —
`fs-driver-build` (merged <SHA-TBD>) wired a real `ken-interp` reduction, a
`run_io` driver arm, and a capability gate
(`crates/ken-interp/src/eval.rs`). `read_bytes` now really reads files,
real-capability-gated (`crates/ken-elaborator/src/{prelude,capabilities}.rs`).

## What's still missing (this example's residual)

This example (`read-file-lines.ken`) has **not** been re-authored to the new
signature (`Cap -> Bytes -> FS (Result Bytes IOError)`, was
`Bytes -> Bytes`), and a `splitOn`/`lines : String -> List String` helper
(needed to split the read bytes into lines) doesn't exist yet. It also needs
the surface `using cap : Cap FS` -> minted-`Cap_FS` -> `read_bytes` path
exercised by a real elaborated program — today's `fs-driver-build` tests
hand-feed the capability at the `EvalVal` level, not through a real surface
program.

## Fix needed

Tracked as its own follow-on WP (Steward, `fs-read-file-lines-flip`):
re-author this example to the new signature, add the `splitOn`/`lines`
helper, and exercise the real surface cap-injection path end to end.
