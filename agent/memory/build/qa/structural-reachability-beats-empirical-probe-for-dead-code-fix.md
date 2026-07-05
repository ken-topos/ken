---
scope: build/qa
audience: (see scope README)
source: private memory
  `structural-reachability-beats-empirical-probe-for-dead-code-fix`
---

# Structural reachability beats an empirical probe for a dead-code fix

When QA-verifying a dead-code-elimination-style fix (a static check decides
"skip this computation, it's unused" vs "compute it"), the strongest
verification is **structural reachability**: trace the producer (parser/
elaborator/compiler) to determine whether the surface language can even emit a
term that exercises the "used" branch at all. If it can't, the fix is a safe
unconditional skip against everything expressible today — stronger evidence than
any empirical probe, because an empirical probe only samples the input space
while a reachability trace covers all of it by construction.

**Why:** on RTP1 (`ken-interp`'s `elim_reduce` eager-IH fix), runtime-leader
asked for an extra probe — an IH consumed "indirectly" via let/application, to
make sure the static `term_var_free` check didn't false-negative. Instead of
writing a scratch `.ken` program, I traced `compile_match_matrix`/
`build_ctor_buckets` in the elaborator and found the injected IH-lambda column
is **never counted by the resolver** (real-depth counting skips `Ih` columns;
`cx.ctx` never pushes for them) — so no surface `match` can ever produce a `Var`
reference into that slot. The "used=true" branch was *structurally unreachable*
from any term the elaborator currently emits, which is a complete proof for the
current surface, not a sampled data point.

**How to apply:** when a fix's safety hinges on a "provably unused ⇒ skip"
claim, first ask "can the producer even build a term where this is used?" by
reading the producer's own construction code (resolver/elaborator indexing,
AST-building) before reaching for an empirical repro. If the answer is "yes,
constructible" — then empirical probes are the right next step (you need a case,
not just a proof it's possible). If "no" — say so explicitly in the verdict;
it's a stronger claim than "I tried a few cases and they passed." Complements
perf primitive vs fix the evaluator fork (same WP family) — this is the QA-side
verification technique, not the design-fork discipline.
