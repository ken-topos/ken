---
scope: build/qa
audience: (see scope README)
source: private memory `isolate-executed-vs-present-before-naming-perf-cause`
---

# Isolate executed-vs-present before naming a perf cause

On `wp/VAL2-rosetta-pangram` (2026-07-03), `gcd.ken`'s `natToDecimal(4)` went
from ~55ms (isolated) to >60s once prepended with 3 small decls
(`OrdResult`/`list_append`/`concat`). I shipped this as "prepending unrelated
declarations amplifies cost" / "sensitive to total declaration count" in a
`KNOWN-GAP.md`, without isolating whether the 3 prepended decls were merely
PRESENT or actually EXECUTED by the reachable computation.

**Why the framing was wrong:** Runtime's own falsification test (`RTP1` D1,
`wp/RTP1-interp-sharing`) prepended 0/5/10/20/30 wholly-**unused**
`view padN : Nat = Zero` decls ahead of the same `natToDecimal` call — cost was
flat (`elim_reduce` call count identical at every pad size). The real driver:
`gcd.ken`'s own pre-existing helpers
(`natAdd`/`natSub`/`natCmp`/`natGcdFueled`/`natGcd`) aren't inert padding —
`main`'s argument is `natGcd twelve eight`, so they're genuinely *executed*, and
`natGcd`'s own fuel (`natAdd a b = 20`, a much larger Suc-chain than the small
literals I'd tested against) pays the bug's cost independently, regardless of
anything prepended around it.

**How to apply:** before attributing a perf regression to "adding more
declarations to the environment," run the cheap isolating test FIRST: prepend
declarations that are structurally present but never referenced by the reachable
computation (a `view padN : T = <trivial>` loop). If cost stays flat, the driver
isn't "environment size" — it's something about what the reachable computation
itself executes (a larger input, an extra helper call, etc.), and the
fix-relevant framing is "which code runs," not "how much code exists." Ship the
isolating test's result, not just the correlation you first observed. Sibling of
probe recursion depth before writing the real test (measure before
writing/shipping a claim) — this one is specifically about isolating *executed*
vs *present* before naming a cause.
