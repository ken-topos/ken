---
scope: build/qa
audience: (see scope README)
source: private memory `isolate-mechanism-from-orthogonal-fail-closed-gates`
---

# Isolate a mechanism from orthogonal fail-closed gates when probing a boundary

**The technique (Ken `L-match-ih-fix` gate, 2026-07-03, `evt_5mk4ep6e94j3q`).**
Gating the VAL2 #5 match-motive completeness fix (sibling-countdown
`ColKind::Ih(remaining)` so a ctor's batch-sibling IH slots get flat types while
a genuinely-enclosing nested split still folds its pending tail), I wanted to
confirm my own mechanism-read concern: could an *enclosing* batch's own siblings
over-fold in a **nested ≥2-recursive-field** case the acceptance suite didn't
cover? First probe: a recursive `depth` over `Node (Node a x b) y c`. It
REJECTED — but with `NotTerminating("SCT: idempotent self-loop …")`, the
**termination checker**, not the motive machinery. Inconclusive: SCT fired first
and masked whether the motive builds. Second probe: the **same nested pattern
with CONSTANT arms** (no self-recursion ⇒ SCT never runs) — ACCEPTS. That
isolated the motive/IH-type build and answered the real question: the nested
case builds correctly; the recursive rejection was a *separate, orthogonal* SCT
limitation, correctly fail-closed, out of the WP's scope.

**Why this matters.** A completeness fix lives behind a stack of independent
fail-closed gates (elaborator motive build → SCT/termination → kernel positivity
→ exhaustiveness). Any one of them rejecting a probe reads as "the fix doesn't
cover this case" when the *actual* cause is a different gate you weren't
testing. To characterize the boundary of the gate you care about, the probe must
trigger THAT gate and **not** the others — otherwise a rejection is
uninterpretable (which gate rejected?) and you'll either wrongly under-scope the
fix as incomplete or wrongly block on a limitation that isn't the fix's job.

**How to apply.** When probing whether a fix's mechanism over-restricts in an
untested shape: (1) identify the other fail-closed gates on the same path; (2)
construct the probe to *avoid* them — strip recursion to dodge SCT, use total
arm-coverage to dodge exhaustiveness, use a known-positive type to dodge
positivity — so only the mechanism under test can reject; (3) if a probe
rejects, classify WHICH gate (read the error constructor:
`NotTerminating`/`TypeMismatch`/`NonPositive`/exhaustiveness) before concluding
anything about the fix. A rejection from an orthogonal gate is a *separate,
correctly-fail-closed* completeness boundary, not a defect in the fix — report
it as such (named + out-of-scope), don't fold it into the verdict. Corollary:
this never changes a soundness verdict for a completeness fix (every gate
rejecting is the safe direction) — it's about honestly *scoping* what the fix
covers vs. what remains fail-closed elsewhere. Sibling of kernel rejects is
completeness fix is where soundness converts (the fix-gate that this
boundary-probe supports) and named floor must be grepped not assumed (grep/probe
the actual behavior, don't reason abstractly about expressibility).
