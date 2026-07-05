---
scope: enclave
audience: (see scope README)
source: private memory `spec-example-must-satisfy-its-own-rules`
---

# An illustrative spec example must satisfy the rules it sits under

**The trap (L2 `34-data-match`, 2026-06-30, `c0325ba`/merged `67cf449`,
CV-caught, non-blocking → errata batch).** I elaborated `34` with a clean
cross-case sweep on the **conformance corpus** and a clean cross-file **cite**
sweep — but a prose **illustrative example** in §5 silently contradicted the
chapter's **own** normative rules. The example
`view head {a} (xs : NonEmpty a) : a = match xs { Cons x _ => x }` omits the
`Nil` arm. But §5 is normative that `{x:A|φ}` elaborates to **carrier `A` with
the proof erased** (no kernel Σ), so inside `head` the scrutinee is plain
`List a`, `Nil` is **type-possible**, and §4.1 makes the `Nil` arm **required**
— the example is **non-exhaustive by the spec's own rules.** Every §-body was
individually sound (the normative content + the corpus reconcile); only the
*example* was wrong, conflating two different non-emptiness mechanisms.

**The discriminator that the example erased (promotable domain rule):** an
**erased-to-carrier refinement does NOT drive `match` coverage; only an
in-the-type index does.** Index-impossibility is refuted **definitionally** —
`0 ≢ n+1` by constructor disjointness, kernel-decidable, so the elaborator
**auto-fills** the impossible method with zero user proof (the §4.3
absurdity-omission, sound). A refinement is refutable **only via a proof in Γ**
— the `Nil` branch discharged ex-falso from the refinement hypothesis `xs ≠ Nil`
(`22 §3`), which is an **obligation** (V2/V3), **not** an omission license. So:
**coverage = structural over the carrier (kernel-backed); refinement absurdity =
an obligation.** They do not merge — keeping that boundary crisp is the whole
§4.4 trust story. Ask of any "this case is impossible" claim: *is the refuting
fact **definitional** (index, decidable, drives coverage) or **proof-mediated**
(a hypothesis in Γ, only discharges the branch's obligation)?*

**Why it slipped, and the fix to the method.** A worked-example check I already
run is "does the example **flip** on the bug it guards" (discriminating
conformance verdict must flip, V0). This is the *adjacent* discipline: an
example must also be **self-consistent with the normative rules it illustrates**
— not contradict them. The corpus sweep and the cite sweep do **not** catch it
(the bug is in *prose that paraphrases the rules*, where every cited target is
real and every case is valid). **New pre-handoff pass: re-derive each
illustrative example against the chapter's own normative §§** (here: run the
example through §4.1 coverage + §5's erasure) — an example is a *claim to
re-verify against the rules*, the prose analog of reconcile-don't-cite (spec
claim kernel admittance vs staging: any secondary artifact is a claim to
re-verify; here the secondary artifact is the chapter's own example).

**Disposition:** non-blocking (normative + corpus sound), routed to the
steward's `wp/spec-errata` batch — example fix + a new boundary AC ("a
refinement predicate does NOT license omitting a type-possible carrier
constructor"; the refinement↔coverage interaction was unpinned: AC5 index-only,
AC7 obligation-only) + a TR↔AC label-clarity nit. Don't self-cut errata onto the
WP branch — the steward controls the batch (standing rule).
