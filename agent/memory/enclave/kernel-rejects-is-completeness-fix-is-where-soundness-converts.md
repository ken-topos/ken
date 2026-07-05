---
scope: enclave
audience: (see scope README)
source: private memory
  `kernel-rejects-is-completeness-fix-is-where-soundness-converts`
---

# Kernel-rejects is completeness; the soundness risk is in the fix

**The classification heuristic (Ken VAL2 finding #5, 2026-07-03,
`evt_75xfr2jwmwqgf`).** A user recursive `data` with ≥2 fields of the same
recursive type failed `match` with `KernelRejected { TypeMismatch }` (even with
constant arms). Steward asked completeness-vs-soundness + whether a sibling case
could signal a *soundness* hole.

**Classify from the FAILURE MODE, not the root-cause site.** The kernel
*rejecting* a term that should type-check = **over-rejection = fail-closed = the
SAFE direction = a COMPLETENESS bug.** The elaborator built a malformed
dependent-match motive over the repeated recursive occurrences; the kernel
*correctly* refused it. Nothing false is admitted; it cannot inhabit `Bottom`. I
did not need the exact site to classify — "the kernel rejects" settles the
direction; the site only scopes the fix. **The rejection is POSITIVE soundness
evidence**: it shows the kernel's `Elim` checker is live and strict on exactly
this machinery, so the elaborator's defect can only ever *over-reject*, never
smuggle an unsound elimination past the kernel. The "sibling case where the
kernel *accepts* a malformed motive" is a separate hypothesis the evidence
*contradicts* (it's rejecting).

**The soundness risk lives in the FIX, and there's a specific conversion
vector.** A safe completeness bug becomes a soundness hole if the fix makes the
term type-check by the WRONG means — specifically, **by loosening the TRUSTED
check to swallow the outer layer's malformed output** (here: relaxing the
kernel's `Elim`/motive checker instead of fixing the elaborator to build the
correct motive). So the fix-gate, load-bearing check first:
1. **TRUSTED LAYER UNTOUCHED** — `crates/ken-kernel/` diff empty,
   `trusted_base()` unchanged. Fix in the outer layer (elaborator), build the
   *correct* thing; do NOT loosen the trusted check. A fix that touches the
   kernel checker to accept the previously-rejected term is the red flag —
   escalate before it lands.
2. **Correct-for-the-right-reason, not trivially-green** — the now-accepted case
   type-checks because the construction is genuinely correct (each recursive
   field gets its proper motive-instantiated type + IH slot), verified by a REAL
   structural-recursion test that *uses* the IHs and computes the right value,
   not a constant-arm match that merely type-checks.
3. **No over-correction into under-rejection** — discriminating pair: the valid
   case **accepts**, an ill-typed sibling **still rejects**. Widen the accepted
   set by *exactly* the valid programs, no more.

**The mechanism sharpened it (root cause pinned, `evt_zsq8w9g48f8s`).** The bug
was an **over-build**: the elaborator gave the first induction-hypothesis slot
an extra Π-layer (`Pi(ret,ret)` instead of flat `ret`) by wrongly folding a
helper's full-codomain-fold over a sibling IH column. An over-build makes the
*expected* type STRICTER → can only over-reject, never under-accept — which is
*why* there's no sibling-accept case (the error always ADDS structure, never
drops it; adding structure to an expected type can only reject more). Even
sharper: **because the kernel checks the `Elim`, EVERY way the outer layer can
mis-build is caught without a false proof** — over-build → kernel rejects;
under-build (drop a slot) → wrong arity → kernel rejects; wrong-but-well-typed
(mis-bound/off-by-one flat) → kernel accepts a type-correct `Elim` → a WRONG
VALUE in the tested ring, never a false proof (caught by a real-recursion
correctness test). So **the whole soundness gate collapses to one invariant: the
trusted checker stays untouched.** That's not just a checklist item — it's
provably the *only* path from this completeness bug to a soundness hole.

**Urgency calibration:** when the trusted layer is the holding backstop, a
completeness bug is **capability-driven urgent, not a soundness emergency** —
trust root intact, `main` honest, existing proofs unaffected, nothing false
admittable while it stands (it only rejects). So "fix carefully and promptly,
ahead of the feature catalog," NOT "halt the fleet" — and *fix it right, not
fast*, because a rushed change to the exact rejecting machinery is where the
completeness→soundness conversion lives. LIVE: the expedited Ken match-motive
fix WP returns to me for this gate. Sibling of soundness AC static vs runtime
face (a fix has faces; check the right one).
