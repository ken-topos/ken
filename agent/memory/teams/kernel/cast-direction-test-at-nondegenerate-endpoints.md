---
scope: teams/kernel
audience: (see scope README)
source: private memory `cast-direction-test-at-nondegenerate-endpoints`
---

# Test a directional cast at non-degenerate endpoints

When a rule or schema builds a **directional transport** —
`cast A B e (a:A) : B` (value at *source* A, result at *target* B), or
`transport`/`subst`/a coercion — the **source/target order is load-bearing and
silent**: getting it backwards is type-wrong in general but **invisible whenever
`A ≡ B`**, because regularity collapses it (`cast B B refl a ⇝ a`) regardless of
order. So a test at a **degenerate/constant** instance (where the two endpoints
are definitionally equal) **cannot witness the direction** — a correct and a
reversed schema give the **same** verdict (green-vs-green). You MUST exercise it
at **non-degenerate endpoints (`source ≢ target`)**, where the reversed form is
ill-typed and the verdict flips.

**K2c-series-2 seam-3 (quotient respect, `16 §5.1`), Architect-caught
(`dec_4y5s10dy784q1`).** The respect obligation must carry `f y : M[y]` **into**
`M[x]` (the type the enclosing `Eq (M [x]) (f x) _` requires), i.e.
`cast (M [y]) (M [x]) (sym (cong M h')) (f y)`. I wrote it **reversed** —
`cast (M [x]) (M [y]) (cong M h') (f y)` — feeding `f y : M[y]` to a cast whose
*source* is `M[x]` and landing in `M[y]` where `M[x]` is needed. **Sound** (the
ill-formed expected-type rejects a valid proof — never wrong-accepts; the
closed-`Empty` guard still holds) but **incomplete**: it rejected *all*
dependent-motive uses. The prose intent ("transport `f y` from `M[y]` to
`M[x]`") was right all along — only the formula was backwards, and the Architect
approved it reading the intent and missing that the literal order contradicted
it.

**Why every test missed it (the co-varying blind spot).** The single Type-target
conformance case used a **constant** motive `M = λ_. Nat` (and the
closed-`Empty` probe `M = λ_. Bool`), so `M[x] ≡ M[y]` and the cast collapsed by
regularity in *both* directions. The schema (spec) and its test (conformance)
**co-varied on the same degeneracy** — both only ever instantiated `M` constant
— so neither could catch the other. Contrast **seam-2's** J-cast (correct) and
**seam-1's** index cast, whose `Vec A (suc n) → Vec A (suc m)` test *genuinely
changes the index* (`n ≢ m`) — which is exactly why seam-1's direction was
trustworthy and seam-3's wasn't.

**Why:** this is the cast/transport specialization of discriminating conformance
verdict must flip (right vs wrong must reach DIFFERENT verdicts) and trust root
test coverage discipline (exercise the distinguishing dimension, not the happy
degenerate case), and a sibling of spec conv omega shortcut trap (over-equating:
there definitional-equality fires too eagerly; here the *test's* definitional
equality masks a real asymmetry). The general root: **a degenerate instance can
satisfy a buggy and a correct rule identically** — pick the test point where the
bug the rule guards against actually changes the outcome.

**Generalizes past `cast` to any monotone-bound refinement — and KERNEL-BACKED
does NOT make the orientation pair redundant (Sec2 `62 §3.2`, 2026-06-30,
`a458053`, both gates confirmed).** Attenuation's bound
`{c' | authority c' ⊑ authority c ⊓ w}` is an order-dual `⊑` direction (like the
taint axis, taint axis orientation needs distinguishing pair). Unlike Sec1's
erased labels, the bound is a **kernel-re-checked refinement obligation**
(`34 §5`/`21 §2`, caps are real Π values) — yet the orientation is STILL
conformance-netted, not kernel-netted, because the **canonical witness collapses
the direction**: `attenuate` computes `authority c' = authority c ⊓ w`, so the
obligation `(c⊓w) ⊑ (c⊓w)` AND its reverse are **both refl** —
direction-degenerate at the meet. So a **backwards `⊑` still type-checks** under
the kernel obligation; only the **non-degenerate distinguishing pair** on STRICT
authorities (`c⊓w ⊏ c`) nets it (weaker cap **accepts** at a weak sink **while**
it **rejects** at a sink demanding the parent's full authority — the reject
flips to wrongly-accept under the bug; the accept alone is green-vs-green).
**The rule: kernel-backed ≠ orientation-netted.** A reflexive/constant/meet
witness collapses *any* directional check — `cast B B refl` (regularity), a
constant motive, OR a meet-valued refinement bound — so the non-degenerate pair
is required *whether or not* the kernel re-checks the obligation. The backstop
differs from the erased case (taint axis orientation needs distinguishing pair:
pair is *sole* net because erased) — here the kernel nets the *magnitude*,
conformance the *orientation* — but the pair is mandatory in both.

**How to apply:** (1) whenever a spec rule constructs a `cast`/transport/coerce
or a monotone-bound refinement, **write the source and target types explicitly
and check the value sits at the source and the result at the target** — trace
`value : source`, `result : target`, against the convention
(`cast A B e (a:A):B`), don't eyeball the `(A)(B)` order. (2) For its
conformance, **demand a non-degenerate endpoint**: for a motive-dependent cast
that means a **dependent** motive with `M[x] ≢ M[y]`; assert the
**correct-direction** form is accepted **and** the **reversed** form rejected
(verdict flips on direction alone). A constant-motive / `A≡B` case proves the
check *fires*, never the *direction* — say so and add the non-degenerate case
beside it. (3) Internal-consistency pass: if two sibling seams both build a
`cast`, verify they use the **same** convention and only one isn't quietly
reversed (seam-2 was right, seam-3 wrong — the contrast is the tell). Extends
the absence/degenerate-assertion family in conformance reconcile inherits spec
metatheory bugs.

**Concrete technique when the proof-type-check obstacle blocks infer_quot_elim
tests for opaque motives:** the kernel's `infer_cast` type-checks the proof,
requiring `convert(Type, m_y, m_x)` — which fails for opaque m_y ≢ m_x, making
any `infer(QuotElim{opaque_motive, ...})` test fail at the proof level, not the
direction level. Escape: test the **Cast reduction directly via `whnf`** with an
**inductive-indexed motive** so `cast_at_inductive` fires structurally for both
directions (bypassing proof-type-check) and produces **observably different
constructor args**. Correct `Cast(m_y, m_x, _, vcons A n a xs)` →
`vcons A m a (Cast xs)`; wrong `Cast(m_x, m_y, _, vcons A n a xs)` →
`vcons A n a xs` (bare xs, same index). The **forced constructor arg**
(tail-length `m` vs `n`) is the discriminant that makes the verdict flip without
needing a well-typed Cast proof. Pattern: `cast_at_sigma` / `cast_at_inductive`
are purely structural — use one to make the direction observable as concrete
output, not as an accept/reject in `infer`. (K2c-s2-build seam-3 second round;
`bb0b3ba`.)
