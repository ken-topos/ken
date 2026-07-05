---
scope: build
audience: (see scope README)
source: private memory `general-fix-can-conflate-similar-shaped-different-cases`
---

# A general fix can conflate similar-shaped but different cases

On `wp/L-match-ih-fix` (2026-07-03, VAL2 finding #5), the frame's root-cause pin
was precise: `compile_match_matrix`'s `ColKind::Ih` branch over-built an
induction-hypothesis binder's type by folding `tail_codomain` over sibling Ih
columns, when >=2 of a constructor's own recursive fields produce adjacent `Ih`
columns that should each be flat `ret_ty`. My first fix generalized this too
far: "always flat, never fold `tail_codomain` in the `Ih` branch." This passed
every new targeted test (a real tree `size`/fold, a discriminating
valid/ill-typed pair, 0-3-recursive-field regression cases) — but regressed two
*pre-existing* `l2_acceptance.rs` tests over a **different** shape: a
*single*-recursive-field type (`NatL = Zero | Succ NatL`) with a **nested**
sub-pattern (`Succ (Succ m)`).

**Why the conflation happened:** both shapes produce consecutive `ColKind:: Ih`
entries in `col_kinds` — structurally indistinguishable at that level. But their
correct binder types differ: true siblings from the SAME `build_ctor_buckets`
call (one constructor's own multiple recursive fields) are independently flat;
an Ih column sitting inside a NESTED split's own method still owes that
enclosing split's non-flat pending continuation (the split's own motive
codomain), which must still fold via `tail_codomain`. A fix that treats all
adjacent `Ih` columns identically will get one of the two cases wrong, and
neither the new tests (which only combine "flat siblings, no nesting") nor
casual code reading surfaces this — it only trips on a scenario the new tests
don't happen to cover.

**How to apply:** when a bug's frame diagnoses one specific *shape* (e.g. "a
constructor with >=2 recursive fields"), be suspicious that a fix expressed as
"always do X instead of Y" may be over-general — check whether the mechanism
you're changing is ALSO reached by a different, unrelated shape (here:
nested-pattern splitting) that happens to look identical in the local data
structure. The only reliable net for this is running the FULL pre-existing test
suite (`cargo test --workspace`), not just the new acceptance tests scoped to
the frame's own AC — the new tests, by construction, tend to only vary the
dimension the frame called out. If the full suite catches a regression, the fix
likely needs to be made *conditional* (tag/count/flag distinguishing the two
shapes) rather than uniformly generalized. Sibling of isolate executed vs
present before naming perf cause (verify a plausible- looking causal/mechanistic
story against a falsifying test before shipping it) — this one is specifically
about a FIX's scope, not a diagnosis's scope.
