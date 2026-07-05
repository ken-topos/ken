---
scope: teams/kernel
audience: (see scope README)
source: private memory
  `gate-widening-exposes-latent-bugs-in-newly-reachable-code`
---

# Gate-widening exposes latent bugs in newly-reachable code

On `wp/dependent-match-nonnullary` (2026-07-03, Map Gap B, merged `282856c` PR
#254), widening `check_match_dependent`'s gate from `nullary` (flat +
all-ctors-empty-args) to `dependent_eligible` (flat + zero result-indices — i.e.
`List`/`Tree`, not just `Bool`) surfaced **two pre-existing bugs**, neither
introduced by the widening itself (both reproduced identically on unmodified
`origin/main`, confirmed before attributing them to my own change):

**1. `subst_var` used for goal-generalization, not binder-removal.**
`check_match_dependent`'s motive/per-branch-goal construction substituted the
scrutinee's (weakened) de Bruijn position with a fresh binder via
`subst_var(term, j, u)` — the standard β-reduction substitution, which
**decrements every index `> j`** (correct when a binder is genuinely *removed*
from the context, e.g. `(λx.body) arg ⇝ body[arg/x]`). But generalizing a
match's goal over its scrutinee does NOT remove the scrutinee's binder from Γ —
it stays a live context entry; only the NEW term (the motive body) stops
directly referencing it, favoring a fresh outer binder instead. Decrementing
indices above the scrutinee's position silently mis-indexes any OTHER free
variable declared BEFORE it (e.g. a generic `(a:Type)` parameter the return type
closes over) onto whatever sits one slot over — surfaces as a `TypeMismatch`
that looks unrelated to indexing. **Never manifested under the OLD
(nullary-only) gate** because a nullary scrutinee (`Bool`, etc.) essentially
never had another closed-over variable declared before it in the tested corpus.
Fix: a new `subst_var_generalize` (same traversal as `subst_var`, but the
`i > j` arm leaves `Var(i)` UNCHANGED instead of decrementing) — used for the
motive body, the per-ctor `expected_here`, and each IH's type.

**2. Unzonked universe metavariable reaching the raw kernel.**
`kernel_infer(cx.env, &cx.ctx, expected)` (a direct call into
`ken_kernel::infer`, bypassing the elaborator's metavariable machinery) could
see a raw `Level::Var` (unresolved universe metavariable) still stored in
`cx.ctx` for a bare surface `(a:Type)` parameter — pinned to `Type 0` only by
LATER unification (e.g. from `xs:List a` requiring `a:Type0`) that hadn't run
yet at this call site. The raw kernel has no notion of elaborator metavariables
(`Level::Var` is just an opaque, non-zero level to it), so `infer(App(List, a))`
fails with `TypeMismatch{expected:Type0, found:Type <meta>}` the moment a family
whose param is concretely `Type0` gets applied to a not-yet-pinned `a`. Fix:
zonk `expected` (shadow the parameter,
`let expected = &cx.metas.zonk_term(expected);`) AND a throwaway zonked copy of
`cx.ctx` before this one raw-kernel call.

**How to apply:** when a WP's job is to WIDEN an existing precondition gate
(nullary→non-nullary, one-family→many-families, etc.), the real regression risk
is rarely in the NEW code path — it's in EXISTING code that was only ever
exercised under the OLD, narrower precondition. Full-workspace-green is
necessary but not sufficient to trust; explicitly hunt for any assumption in the
touched function that the old gate's narrowness was silently protecting ("does
this ever get called with X declared before the scrutinee?", "is this
metavariable guaranteed resolved by the time this runs?"). Confirm any suspected
pre-existing bug is reproducible on UNMODIFIED `origin/main` too (via
`git stash`) before fixing it under the current WP's banner — this both grounds
the fix's scope and avoids mis-attributing an old bug as a new one. Sibling of
isolate executed vs present before naming perf cause (same discipline: isolate
what's NEWLY EXERCISED vs what changed) and hand built elim motive and method
gotchas (same WP family, sibling kernel-hand-building traps).

**Also carry:** when hand-deriving a repeating index/shift computation (IH wrap
order, field positions), hand-trace the SMALLEST case with `p > 1` (two
recursive fields, e.g. a `Tree`'s `Node l k r`) before trusting the formula — a
`p=1` case can't distinguish "correct" from "an off-by-one that happens to
cancel." My first attempt at the IH-slot substitution baked in a per-index `+i`
shift that was actively wrong (double-counting) — the correct fix uses a UNIFORM
(non-i-dependent) per-slot formula and lets the same `weaken(_,1)`-per-wrap
technique the existing `ColKind::Ih` precedent already uses accumulate the shift
naturally across the reverse-order wrap loop.
