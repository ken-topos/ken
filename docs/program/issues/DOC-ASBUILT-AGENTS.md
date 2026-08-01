---
id: DOC-ASBUILT-AGENTS
title: "As-built slice 6 — reconcile the thirteen-page agents corpus against its 7 shared drifted sources; it is instructions machines follow, not prose people skim"
status: merged
owner: doc
size: M
gate: none
depends_on: [DOC-ASBUILT-FRAGMENTS]
blocks: [DOC-ASBUILT-AUDIT, DOC-ASBUILT-LEDGER]
github: null
origin: "Steward 2026-08-01, phase A slice 6 of DOC-ASBUILT-AUDIT, measured at origin/main 4ab1c23e. Cut by audience, per the campaign's shared-source-set rule. Final phase-A slice."
---

> # ✅ MERGED 2026-08-01 — PR #1304, `origin/main = 3cc4fa19`
>
> ## ⭐⭐⭐ PHASE A IS COMPLETE.
>
> Exact candidate `239c5e117afae4d7249be933a31ad1799f741fa5`, tree
> `da17d2b16ca8968d2cb818b08dbb65346eccea80`, base `ac187ada`. Librarian QA
> `evt_4g9ghc5kt4rws`; Decision `dec_2dtksy0ebwk4n` resolved. All post-conditions
> exact — `library/README.md` blob `9e2f597bcd6e3a1ced6e0a95b500adda778786a5`,
> the only path changed, and the anti-revert check confirmed slice 2's two
> runtime blobs survived the merge.
>
> **Repaired:** `library/README.md` only (+22/−19) — the inherited
> stale-curriculum-progress table row from [[DOC-ASBUILT-READER]], and a portal
> paragraph. The other twelve pages were reconciled and found still true.
>
> ## ⚠⚠ THE LIBRARIAN REJECTED THE FIRST CANDIDATE FOR AN INVENTED CLAIM
>
> `236b39df` was rejected (`evt_76va5srcc7yw`): the repaired portal paragraph at
> `library/README.md:40–42` promised a **future live-corpus gate that does not
> exist**. ⇒ ⭐⭐ **An as-built repair can introduce a NEW false claim while
> removing an old one** — the failure mode is not only "missed a survivor", and a
> QA pass that only re-checks the routed class would not have caught it.
>
> ## ⛔ TWO SOURCE DEFECTS FOUND AND ROUTED, NOT REPAIRED
>
> Both are `docs/program/` files this slice may not touch, and both were found by
> the **corpus-wide `D3` class sweep** that [[DOC-ASBUILT-READER]]'s retro added:
>
> 1. `07-catalog-style-guide.md` — §5 retires the Findings section while line 31
>    and §2 item 6 still require it. Steward ruling `evt_1q488s8hnah1s`: **§5
>    governs**, the other two sites are void.
> 2. `12-documentation-program.md:342`/`:374` — Wave 1 and Wave 2 headings say
>    `(FRAMED)` while line 184 says `✅ LANDED`. Steward ruling
>    `evt_34revdy2cdyc4`. ⛔⛔ **`library/manifest.toml:322` cites the Wave 2
>    heading BY ANCHOR** (`…-framed`), so repairing the word renames the anchor
>    and breaks a live citation — the fix is **coupled to a manifest write**.
>    ⚠ Wave 1's heading is cited by nothing; the two are **not** symmetric.
>
> ⇒ Both sequenced **after** this merge, because editing either moves a source
> OID that phase A's pinned drift control depends on.

> # ▶ SLICE 6 OF PHASE A — THE AGENTS CORPUS, AND THE LAST PHASE-A SLICE
>
> `library/README.md` + the twelve `library/agents/**` pages.
>
> ⭐ **13 pages · only 7 distinct drifted sources · 18 citations.** Twelve of
> the thirteen pages cite **exactly one** drifted source each.
>
> ⛔ **When this merges, phase A is complete and phase B is releasable.**

## ▶ THE FRAME IS WRITTEN

`docs/program/wp/DOC-ASBUILT-AGENTS.md`. ⛔ **Read the frame, not this node**,
and read [[DOC-ASBUILT-AUDIT]] first — its two-phase law binds this slice.

## ⭐⭐ Thirteen pages is not thirteen units of work — SEVEN sources are

| page | its drifted source(s) |
|---|---|
| `library/agents/core/write-ken.md` | `proof-techniques`, `surface-reference`, `07-catalog-style-guide` **×2** |
| `library/agents/tasks/author-package.md` | `07-catalog-style-guide` **×3** |
| `library/README.md` · `library/agents/README.md` · `library/agents/evaluations/README.md` | `docs/program/12-documentation-program.md` |
| `library/agents/core/read-ken.md` · `library/agents/tasks/write-program.md` · `library/agents/tasks/effects-and-capabilities.md` | `spec/30-surface/36-effects.md` |
| `library/agents/core/toolchain.md` · `library/agents/tasks/diagnose.md` | `spec/40-runtime/42-evaluation.md` |
| `library/agents/core/proof-and-trust.md` | `spec/60-security/64-trust-model.md` |
| `library/agents/tasks/prove-or-repair.md` | `catalog/guide/proof-techniques.ken.md` |
| `library/agents/tasks/read-review.md` | `docs/program/07-catalog-style-guide.md` |

⭐ **The clustering is the argument.** Three pages hang off
`12-documentation-program.md`, three off `36-effects.md`, two off
`42-evaluation.md`. ⇒ **One read of a source settles three pages at once**, and
⛔ splitting them would read the same seven sources up to three times each.

## ⭐⭐ Why this corpus is the highest-stakes page-set in phase A

⚠ **These pages are instructions that AGENTS follow to write Ken.** They are not
prose a person skims and forgives.

⇒ ⭐ **A stale instruction here propagates into authored code and proofs** —
`write-ken.md` tells an agent how to write a package, `author-package.md` how to
lay one out, `prove-or-repair.md` how to discharge an obligation. A drifted
style guide or a changed effects rule makes those instructions **actively
wrong**, and the failure surfaces later as a rejected candidate whose author was
following the documentation.

⚠ **`docs/program/07-catalog-style-guide.md` is the most-cited drifted source in
the corpus** (9 consumers) and appears **6×** in this slice alone. ⭐ It is the
single highest-yield read in the campaign's tail.

## ⭐ The drift baseline is **28** and does not move

⛔ None of these thirteen is attested, so editing them adds no row. **29 means
something was written that should not have been; 27 means a pre-slice-1 base.**
