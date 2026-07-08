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

**Disposition sub-rule (sharpened):** a **non-blocking strengthening** surfaced
at review, when the branch is **free**, is **folded now** (a scope-mismatched
carry silently never lands — trust level prose vs locked adr crosscheck's Lc
lesson) — BUT fold-now is only clean if the fold stays **ahead of the Decision's
SHA anchor**. Folding **after** a Decision is *proposed* (SHA recorded, gates
cast on it) moves the tip out from under the record, and the merged SHA must
equal the Decision's authoritative SHA (multipiece erratum verify all on main).
So: fold **before** the gate opens; or if a gate is already cast, either hold it
as a **named fast-follow** (don't move the tip under a live Decision) or fold +
explicitly **re-anchor the Decision's SHA** to the new tip + re-affirm carrying
gates. Live (Sec4): I folded B4 after gates were cast on `a81da90` → tip moved
to `e940fe2`, but `a81da90` **merged to main** (`446c2f3`) before the fold
caught up (a reviewer's "still on 0e4a93d" was already stale), so the SHA race
became a **forward erratum**, not a pre-merge fix — the Steward cherry-picked
`e940fe2`'s B4 delta onto main (additive, no revert/force-push; the
Lc-coherence-erratum pattern), the three gates carrying. Sharpened: once a
proposed tip can merge out from under you, a later fold is a **fast-follow
erratum**, so when a gate is already cast prefer **holding the strengthening as
a named fast-follow** over moving the tip.

**Temporal-race edge (ES3, 2026-07-01).** "Fold ahead of the SHA anchor" is not
just a *logical* ordering — it is a **race against the merge pipeline**. When a
Decision is **proposed** (SHA recorded, gates voting), the publisher path may be
mid-**PR-publish** on that SHA; a fold + re-anchor **races that publish**. If
the publish + merge fires on the proposed SHA before your fold lands, the fix is
**orphaned into a post-merge erratum** — the exact thing fold-ahead was meant to
avoid. Live: I folded a `11 §4` quote-fix after `dec_8ce3w6h1dm2b` was proposed
on `cdbf155`; it **won** the race (main landed my fixed `106a601`), but it was a
near-miss. **Rule:** a pre-merge fold-ahead is clean **only if it wins the
race** — so when folding after a Decision is proposed, announce the new SHA
**AND explicitly flag the Steward/publisher path to hold + merge the new tip**
(never assume the fold beats the publish); if the publisher has already published, treat it
as a post-merge erratum, don't chase the pipeline. A fold worth doing pre-merge
(a wrong **direct quote** gaining false authority on main — laundered-citation
hygiene) is still worth doing; just don't leave the win to chance.
