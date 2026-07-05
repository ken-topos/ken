---
scope: enclave
audience: (see scope README)
source: private memory `eliminator-termination-finiteness-not-stuckness`
---

# An eliminator's termination argument is finiteness, not stuckness

When elaborating the **termination / decidability-of-conversion** argument for a
W-style (Π-bound) eliminator's ι —
`elim_D M m̄ ī (cₖ … k …) ⇝ mₖ … (λ b. elim_D M m̄ … (k b)) …` — **do not
justify it by claiming the inner `elim_D (k b)` is "stuck under the binder
because `k b` has no constructor head."** That is **false** for the typical
case: `k` is the constructor's *branching function*, and if it is
constructor-producing — `k = λx. Vis e' (k' x)` — then `k b ⇝ Vis e' (k' b)` is
**constructor-headed even for an abstract `b`** (the head doesn't depend on
`b`), so the inner elim **does fire** — during normalisation *and* under
η-comparison in conversion (compare two IH-lambdas at their Π type → apply a
fresh `b*` → `k b*` whnf's to a constructor → ι fires). The inner elim is
genuinely neutral **only** in the incidental sub-case where `k` *inspects* its
branch (`k = λx. elim_Bool x …`, so `k b*` is stuck on abstract `b*`).

**The correct basis is finiteness, full stop.** The scrutinee is a **finite**
inductive (non-coinductive) tree and the branching functions are **finite**
λ-terms; ι recurses on `k b`, a **structurally-smaller child** reached *through*
the branching function (a β-step on `k`) rather than directly — the same
structural descent as a direct recursive arg, just staged through `k`. It
bottoms out by finiteness, not stuckness. (K1.5 §7.7/§9.4; Architect-caught
blocker, fixed `cdaa172`.)

**Why:** this is the same over-claiming reflex as spec conv omega shortcut trap
— reaching for an *appealing, mechanistically crisp* justification ("it's stuck,
so it can't loop") that is **stronger than true**, when the honest justification
is a *global* property (finiteness/well-foundedness) that is weaker-sounding but
actually holds. At the trust root the wrong-but-tidy story is worse than no
story: an implementer coding §7.7 literally ("leave the W-style inner elim stuck
on a constructor head") builds a conversion that **never fires `elim_W (Vis …)`
redexes** → valid programs become inconvertible — the exact bug a sibling
conformance case (`wstyle-iota-in-conversion`: "a constructor-headed scrutinee
always fires ι") was guarding against. The false metatheory and the correct
conformance case **contradicted each other**, which is how it surfaced.

**How to apply:** when writing any "this reduction terminates / conversion
decides" argument, (1) **name the global well-foundedness measure** (finite
structural descent on the inductive value) as the load-bearing reason, and only
then (2) describe the *mechanics* — and stress-test the mechanics against an
**abstract/open** scrutinee or branch variable (the conversion/η setting), where
"stuck because a variable is in the way" most often fails: a constructor head
that is **independent of** the bound/abstract variable still reduces. Ask "does
this redex fire when the branch var is abstract?" before asserting it's stuck.
If a decidability claim and a conformance case can both be read literally and
**disagree on whether a redex fires**, one of them encodes a bug — reconcile
before merge. Extends trust root test coverage discipline (assert the real
property) and cbv eliminator method laziness (the dual: there I correctly named
a non-strict position; here the error was mis-naming *why* a position is inert).
