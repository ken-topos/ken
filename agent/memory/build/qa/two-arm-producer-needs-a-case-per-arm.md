---
scope: build/qa
audience: (see scope README)
source: private memory `two-arm-producer-needs-a-case-per-arm`
---

# A two-arm (or multi-arm) producer needs a discriminating case per arm

When a soundness AC bottoms out in a producer that is a **multi-arm
`match`/filter** and the spec enumerates **multiple categories** the producer is
meant to cover, author a **discriminating case per arm**. Exercising only one
arm is **green-vs-green** under a bug that dropped the *other* arm — the corpus
passes while the enumeration is silently incomplete.

Live (Sec4 trust-model): `trusted_base()` is
`matches!(Decl::Opaque | Decl::Primitive)` and §64 §1 says the TCB is *exactly
three things* — kernel, **primitive reductions (item 2)**, postulates/`foreign`
(item 3). My AC1/AC2 covered item 3 (postulate/foreign/hole) exhaustively but
left item 2 (`declare_primitive`→`Decl::Primitive`) **unexercised** — a producer
that dropped the `Primitive` arm would pass B1–B3 green. spec-author's Fidelity
caught it (non-blocking); I folded a B4 `registered-primitive-surfaces-in-delta`
case.

**Tell:** anchoring on the *security-critical* face of a producer (here: item-3
assumptions *hiding*) can under-cover the enumeration's other category.
Enumerate the producer's arms and the spec's categories; give each its own case.
The enumeration-arm analog of "a multi-dimensional guard needs a case per
dimension" (soundness AC static vs runtime face); sibling of discriminating
conformance verdict must flip.

**Disposition of the B4 fold itself** (a non-blocking strengthening surfaced at
review, folded after gates were already cast, racing the merge on `a81da90`) —
that is a distinct discipline from the discriminator-per-arm rule above; see
[[mid-review-fix-inline-escalate-or-track]] for the full fold-vs-track timing
rule and the Sec4 B4 SHA-race evidence.
