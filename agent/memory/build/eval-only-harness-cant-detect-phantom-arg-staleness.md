---
scope: build
audience: (see scope README)
source: private memory `eval-only-harness-cant-detect-phantom-arg-staleness`
---

# An eval-only harness can't detect phantom-arg staleness

When auditing "no existing consumer breaks" under a kernel-term-producing
function's signature generalization (e.g. `resp_sum`'s 3-arg State-first-pinned
form → the general 4-arg `(g h rg rh)` form), a test still passing is not
sufficient evidence the test actually re-validates the new shape — check whether
the test's harness calls `ken_kernel::infer`/`check` at all, or only
`ken_interp::eval::eval`.

**Why:** `crates/ken-elaborator/tests/state_effect_build_eff6_integration.rs`
was untouched by `effect-composition-build` and still constructs `resp_sum` with
the OLD 3-arg shape (`nat_ty, empty_ty, resp_empty` — missing `rh` entirely). It
still passes. Not because the shape is compatible, but because the test drives
`ken_interp::eval::eval` directly on a hand-built `Term` and never calls
`infer`/`check` on it — `Resp` (the `resp_sum`-typed field) is used only as a
type-level index inside `ITree`/`Vis`/`bind`'s Π-types, never inspected by
`eval`'s runtime dispatch. An eval-only harness is therefore structurally blind
to any argument that is phantom/erased at runtime, no matter how wrong its shape
becomes under a producer's signature change. This is a DISTINCT trap from verify
field order arity against declaration not prose (that one was a real consumer
that never existed yet, i.e. a coverage gap in what's tested at all); this one
is an existing, still-running test whose own harness design cannot detect this
class of bug BY CONSTRUCTION, independent of coverage.

**How to apply:** when a WP generalizes/changes a kernel-term-producing
function's arity or argument shape, grep every call site of that function across
the whole test tree (not just the files the diff touched) — for any untouched
call site still using the old shape, don't just confirm "it still
compiles/passes"; trace whether the enclosing test actually calls
`infer`/`check` on a term that consumes the changed argument. If it only calls
`eval`, the passing result is not evidence of shape-correctness for that
argument — flag it (non-blocking if genuinely inert, but flag it) rather than
crediting it as coverage. effect-composition-build gate: caught
`state_effect_build_eff6_integration.rs`'s stale 3-arg `resp_sum` this way;
correctly reasoned as non-blocking (BV6's "no regression" claim survives since
the mismatch is truly inert) rather than either over-flagging as a build defect
or silently missing it as a QA gap.
