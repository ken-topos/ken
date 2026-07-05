---
scope: enclave
audience: (see scope README)
source: private memory
  `tested-not-trusted-posture-needs-reachability-precondition`
---

# A tested-not-trusted posture is sound only with a reachability precondition

When a spec argues a component is **not in the type-soundness TCB** because it
is an **outer-ring evaluator** whose correctness is netted by a **differential /
conformance corpus** rather than the kernel (native backend, interpreter, any
un-trusted execution path — "a bug is a wrong *value*, never a false `proved`"),
that posture has **two** load-bearing halves, and the corpus only pins one:

1. **Correctness net (what the conformance seed pins):** the component's output
   agrees with the oracle (differential equivalence) / passes the discipline
   corpus. This catches a *wrong value*.
2. **Reachability precondition (easy to leave as prose):** the component is
   **structurally unreachable on un-validated input** — its entry is **gated
   behind kernel admission**, so it only ever runs already-kernel-checked terms.

If (2) fails, (1) is **void**: an un-admitted term reaching the evaluator makes
a bug a **real soundness hole**, not a wrong value — the whole not-in-TCB
argument collapses. The correctness net is **necessary but not sufficient**.

**Live (X3a native backend, 2026-07-01):** my `seed-backend.md` pinned the
differential-equivalence net (BD1/BD2) + the not-in-TCB posture (BD3, trust
chain kernel `Q` / interpreter-oracle `tested` / backend `tested`), and §45
§1/§2 *stated* "runs already-kernel-checked core" — but that precondition wasn't
a **hard obligation** until the Architect surfaced it. Steward captured it as a
**hard X3-build AC**: *`ken-codegen`'s entry must be structurally gated behind
kernel admission — unreachable on un-kernel-checked core, **producer-grepped,
not a runtime assertion**.*

**How to apply:**
- When reviewing/authoring a tested-not-trusted / not-in-TCB posture, ask: *what
  guarantees this component only runs validated input?* If it's only prose, flag
  the **reachability gate** as a **hard build/structural obligation**
  (producer-grepped that the entry is unreachable on un-checked terms), distinct
  from the correctness corpus.
- It is a **structural/build** obligation, **not** a conformance-value property
  — a differential corpus cannot net it (the corpus only runs *valid* terms). So
  it belongs in the **build frame** as a hard AC, not in the discipline seed.
  (Keep it out of a resolving-Decision seed — two arm producer needs a case per
  arm's no-mid-Decision-mutation sub-rule.)

Generalizes untrusted layer backstop hole for omissions: there the omission is a
**missing check** the untrusted layer should have emitted; here it is a
**missing gate on the input path** into the untrusted layer. Sibling of trusted
by typing guarantee is not kernel proved Q (label the trust level honestly) —
this adds *what the honest label depends on structurally*.
