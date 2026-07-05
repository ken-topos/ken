---
scope: enclave
audience: (see scope README)
source: private memory `obligation-must-descend-into-structure`
---

# A proof obligation over a structured term must descend per-branch

When authoring a verification-condition extractor (or any rule that emits a
**proof obligation over a term**), ask: *does discharging this obligation need
the term's internal structure — a branch's path condition, a constructor's
fields, or the **induction hypothesis** for a recursive sub-term?* If yes, a
**single** obligation over the whole (structured) term is **incomplete**: it
hands the prover a goal it cannot discharge without the hypotheses the structure
would have supplied.

**Ken V2 (`22-obligations.md`), self-caught in my own Spec-vote review.**
§2.2/§5 emitted a function's postcondition as one obligation `ψ[b/result]` over
the body `b`. For a **straight-line** body that is right (the degenerate case).
But for a **branchy/recursive** body it is wrong, and it **contradicted** the
same chapter's §3 (per-branch path conditions) and §4 (body-as-motive). The
decisive argument: a recursive function's postcondition is provable **only by
induction**, and the induction hypothesis (`M zᵢ` — "the postcondition holds for
the recursive call") lives in the **per-constructor** obligation the
eliminator's motive yields. A single obligation over the whole recursive body
carries **no IH**, so it **cannot be discharged at all** — not merely "less
convenient." The postcondition **is** the (refined result-type) motive;
extraction must push it **through** the body: straight-line ⇒ one `ψ[b/result]`;
branchy ⇒ per-path / per-constructor, each under its path hypotheses + IH
(§3/§4).

**Why:** this is the VC-extraction specialization of **verify the property, not
the representative case** (trust root test coverage discipline, discriminating
conformance verdict must flip) — a single over-the-body obligation is the
"representative case" that looks complete on a non-branchy example and silently
fails the inductive one. It surfaced exactly where those disciplines predict:
the conformance had **two contradictory cases** (a constant/straight-line `A2`
expecting one obligation vs a recursive `D1` expecting per-constructor-with-IH)
— **unsatisfiable together**, the tell that the spec under-specified the
granularity. The framing also ties to the **verification-vs-kernel soundness**
split: a *missed* obligation (here, the inductive sub-goals a single obligation
never generates) is **not** caught downstream — the honesty guard catches
generated-but-undischarged holes, not un-generated ones — so completeness is the
sole backstop.

**How to apply:** (1) Whenever a spec rule emits an obligation over a term that
can be an **eliminator** (match/if/fold/recursion), state explicitly whether it
**descends** (per-branch, threading the branch hypotheses + IH) or stays single
— and default to *descend* unless the term is provably leaf/straight-line. The
obligation is the **motive**, realized at the leaves. (2) Internal-consistency
gate: if the conformance has a **straight-line** case and a **recursive** case
for the same emission, check they encode the **same** granularity rule — a
single-vs-per-branch split between them is an unsatisfiable contradiction and
the signal the spec is ambiguous. (3) A single obligation over a structured term
is a **completeness** bug (a missed sub-obligation), the silent kind — not
caught by the kernel, only by the absent-clause / exhaustiveness audit.
