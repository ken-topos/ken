---
scope: teams/kernel
audience: kernel-leader, kernel-implementer, kernel-qa, architect (soundness
  co-review)
source: distills former private memories `k5-top-bottom-intro-elim-kernel-gap`,
  `k6-conv-struct-eq-congruence-gap`, and `k7-eq-at-inductive-operand-whnf-gap`
  (all RESOLVED/merged historical WP narratives — kept here only for the
  reusable technical shapes, not as status)
related: exhaustive-term-traversals,
  trust-root-reduction-change-needs-full-workspace-gate,
  conv-reduction-arm-gate-needs-termination-stress
---

# Three shapes of kernel completeness gap

Pushing real (non-postulated) law proofs through the kernel tends to surface
incremental, narrowly-scoped kernel-completeness gaps one at a time — each a
genuine but small hole in an otherwise-sound trust root, not a soundness bug.
Three shapes recurred across the ES4-lawproofs arc (K5, K6, K7) and are worth
recognizing on sight in future kernel work.

## Shape 1 — a prop is produced but never consumed (K5)

`Eq` at an inductive with two concrete constructor arguments observationally
reduces (`obs.rs::eq_reduce`) to `Top` (same ctor) or `Bottom` (different ctor)
— but `Top`/`Bottom` were prelude Ω-props only ever **produced** (by `eq_reduce`
and SCT/negation code), never **consumed**: no `Top` introduction, no `Bottom`
elimination (`Bottom` was `Decl::Opaque`, not a 0-ctor inductive, so it couldn't
be `Elim`'d). A law whose "same value" branch goal reduces to `Top` (needs `tt`)
or whose contradictory-hypothesis branch needs ex-falso over a `Bottom`-typed
hypothesis is stuck with nothing to close it. **The fix is textbook and
standardly sound:** add `tt : Top` (sound because `Top` is a sub-singleton,
`Equal Bool True True ≡ Top` by conversion) and `absurd : Bottom → C` (sound
because `Bottom` is *empty* — the eliminator is vacuous, so it can never produce
two different results from two proof-irrelevantly-equal `Bottom` proofs, and
therefore cannot break proof irrelevance regardless of codomain; for law-proving
purposes the codomain is always Ω, so `Bottom`-elim into Ω is the minimal safe
scope). Neither rule inhabits `Bottom` from nothing; a `Bottom` only ever arises
as a variable in an unreachable false-hypothesis branch. **Check for this shape
whenever a new observationally-reduced base proposition (Top/Bottom-like) is
introduced: does something actually consume it, on both the "true" and "false"
side?**

## Shape 2 — a stuck-term congruence arm is missing for one former (K6)

`conv.rs::conv_struct` (structural congruence) carried a positional congruence
arm for every stuck former — `Pi`/`Lam`/`Sigma`/`Pair`/`App`/
`Elim`/`Proj1`/`Proj2` — but **none for `Term::Eq(ty,a,b)`**, so two stuck `Eq`
nodes fell to `_ => false`. **The hard constraint on the fix: the only
admissible arm is positional** —
`conv_struct(ty1,ty2) && conv_struct(x1,x2) && conv_struct(y1,y2)` — never
cross-wise. A cross-wise arm (`x1≡y2 ∧ y1≡x2`) would admit `Eq A a b ≡ Eq A b a`
as a *definitional* equality — smuggling propositional symmetry into conversion,
which is unsound (Ken proves `sym` propositionally; making it definitional
collapses directed `Eq` and enables unproven-symmetry transport via `cast`). Any
discriminating test for a new congruence arm must include a **reject** case
(`Eq Bool a b` must NOT convert with `Eq Bool b a` for distinct `a`,`b`) —
confirming the new case passes is necessary but not sufficient. **A conv.rs
change is the kernel's most delicate surface — more scrutiny than a
typing-admission change, not less**, because over-acceptance here is unsoundness
(an unproven equality becomes transportable), whereas a typing-admission gap is
merely completeness. Also worth knowing: a positional congruence arm does not
universally close every case that "looks like" it should — check whether the two
stuck terms are already convertible some other way (e.g. both operands reduce to
the same head via `whnf`) before concluding a new congruence arm is the
operative fix; it may be a sound-but-customerless addition to ship on its own
completeness merits while the actual blocking case needs a different mechanism
entirely (case-split / lemma / Shape 3 below).

## Shape 3 — sibling normalization functions must whnf consistently (K7)

`ken-kernel/src/obs.rs::eq_at_inductive` peeled both value operands with
`peel_app` on the **raw** terms and head-matched `Term::Constructor` without
first calling `whnf` — while its sibling `eq_at_type` (same file, same role)
**did** whnf first. That asymmetry meant an operation-wrapped hypothesis (e.g.
`Equal Bool (bool_leq x y) True` where `bool_leq True False` is a redex that
whnf's to `False`) reached `eq_at_inductive` un-normalized, stayed neutral, and
never collapsed to `Bottom` — so `absurd` had nothing to discharge even though
the branch was genuinely contradictory. **The fix is airtight-sound because it
can only make the function recognize genuine constructor-heads it currently
misses** — whnf is the kernel's own sound reduction, so
`whnf(a) = False ⇒ a ≡ False` definitionally; there is no risk of fabricating a
false `Top`/`Bottom` and no cross-wise/symmetry hazard (this is a *reduction*
fix, not a conversion-rule fix, so the K6-shaped hazard does not apply). **When
two sibling kernel functions do structurally the same job on related term
shapes, check they normalize their operands consistently** — an asymmetry
between siblings is a classic place for this gap to hide, and it's cheap to grep
for (`whnf` present in one, absent in the structurally-parallel other).

## The meta-lesson across all three

A "reduces entirely within capability K_n" claim silently asserts that **every
intermediate redex fires** on the landed kernel. Clearing the axis you're
actively debating (e.g. "does the congruence gap block this?") does not clear a
different axis the same term also depends on (e.g. "does the operand actually
reduce to a constructor first?"). Before asserting a law is "realizable now via
K_n," check every reduction/consumption step the claim depends on, not just the
one under discussion — the general buildability-ruling-must-ground-every-axis
discipline, applied specifically to kernel reduction/consumption steps.
