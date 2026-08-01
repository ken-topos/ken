# WP frame — `DOC-ASBUILT-CHAPTERS` (as-built phase A, slice 4)

**Owner:** Team Doc (`doc-leader` + `doc-author`, Librarian as QA).
**Branch:** `wp/DOC-ASBUILT-CHAPTERS`. **Size:** L.
**Node:** `docs/program/issues/DOC-ASBUILT-CHAPTERS.md`.

> ## ⛔⛔ READ [[DOC-ASBUILT-AUDIT]] FIRST — its two-phase law binds this slice
>
> ⭐ **Phase A writes NO ledger, and the gate stays red.** ⛔ No
> `library/manifest.toml`, no `library/SOURCE-ATTESTATIONS`, no
> `library/STATUS.md`, ⛔ no `gen-source-attestations.sh` — for any reason.
> ⚠ A red `--check` is **not** a rejection of this candidate.

## Fixed inputs

| input | value |
|---|---|
| **base** | ⭐ **`origin/main` as of when you cut the branch, provided it contains this frame AND [[DOC-ASBUILT-FRAGMENTS]] has landed.** ⛔ Not a SHA copied from this table. |
| the pages | **four**, all in `library/learn/reading-ken/`: `01-anatomy.md`, `02-types-contracts-and-proofs.md`, `03-assurance-and-trust.md`, `05-packages-and-provenance.md` |
| their drifted sources | ⭐ **9 distinct paths / 30 citations** — ⚠ **re-derive at your base** |
| campaign law | [[DOC-ASBUILT-AUDIT]] |

---

## ⭐⭐ THIS IS FOUR PAGES AND IT IS STILL SMALLER THAN SLICE 2

⛔ **Do not read "four pages" as "four times the work."** The expensive act in
this campaign is **reading a source at its current blob and deciding what it
falsifies**, and there are **9** such reads here against slice 2's **16**.

Measured: `01-anatomy`'s sources are a **strict subset** of `05-packages`'s;
`02-types` adds exactly one source over `05-packages`, and `03-assurance` adds
exactly one. ⇒ ⭐ **Six of the nine sources serve three or four of the pages at
once.**

> ### ⭐⭐ THE GROUPING IS A CORRECTNESS REQUIREMENT, NOT A CONVENIENCE
>
> These four chapters are one genre and **share claim classes**. When you find
> a drifted source falsifies a claim, ⭐ **the same claim is likely to appear in
> more than one of these pages.**
>
> ⇒ **`D3`'s sweep is explicitly CROSS-PAGE here.** ⛔ Repairing the class in
> `02-types` and leaving it standing in `05-packages` is the **same defect** that
> sank two earlier candidates in this campaign — the named site fixed, the class
> surviving elsewhere. Cutting these pages into separate WPs would have
> reproduced that defect *across* WPs, where no single QA pass could see it.

---

## ⭐ The expected drift count is **28**, and this slice cannot move it

`fragments.md` was the only attested `library/` page and slice 1 already edited
it. ⛔ **None of these four pages is attested**, so editing them adds no row.

⇒ `AC-4` expects the before-and-after drift blocks **identical at 28**.
⛔ 29 means something was written that should not have been; ⛔ 27 means the base
predates slice 1.

---

## Deliverables

**D1 — per-source claim reconciliation, all 9 sources, ACROSS ALL FOUR PAGES.**
Read each source **once** at its current blob, then for **each of the four
pages** state every claim that page derives from it and whether it is still
true. ⭐ Cite the anchor you actually read.

⭐ **Organize `D1` BY SOURCE, not by page** — that is the whole point of the
grouping, and it is what makes the cross-page sweep fall out for free.

⚠ **Three sources are cited at multiple anchors and each anchor owes its own
entry:** `07-catalog-style-guide.md` (**3** anchors in `01-anatomy` alone),
`spec/60-security/64-trust-model.md` (**4** in `03-assurance`),
`spec/30-surface/30-taxonomy.md` (**4** in `05-packages`).

**D2 — repair what is false**, in these four pages only.

**D3 — cross-page sweep per repaired claim class.** For each class you repair,
sweep **all four pages**, ⛔ not only the one where you found it.

**D4 — a closed report:** still-true / repaired / **routed**, ⭐ **per (source,
page) pair** — a source may be still-true in one chapter and false in another.

> ### ⭐ Where the oracles are
>
> **Four sources are checked catalog code** — `EmptyDec`, `Combinators`,
> `Transport`, `Property`. A claim about what a package *provides* is verifiable
> against its current blob, ⛔ not a matter of taste.
>
> ⚠ **`library/learn/reading-ken/fragments.md` is cited by all four**, and
> ⭐ **slice 1 just repaired it.** Read it at its **current** blob — a claim any
> of these chapters inherited from the **old** `fragments.md` is exactly the
> kind that is now false while reading as perfectly current. ⛔ This is the
> highest-yield single source in the slice.
>
> ⚠ **`docs/program/07-catalog-style-guide.md` is the most-shared source in the
> corpus** (9 consumers) and is cited **3 times in `01-anatomy` alone**. A
> style-guide rule that changed shows up as *normative prose about how Ken code
> is written* — ⛔ easy to read past.

---

## Acceptance criteria

**AC-1 — all 9 sources addressed against current blobs, for each of the four
pages that cites them**, and every multi-anchor citation addressed at each
anchor. **Control:** quote what you read. ⛔ Not from this frame, ⛔ not from
memory.

**AC-2 — every claim any of these four pages makes about a drifted source is
true at your base.**

**AC-3 — cross-page closure per repaired class.** **Control:** name the class
and show the sweep **across all four pages**, ⛔ not only the page where it was
found and ⛔ not only the edited lines.

**AC-4 — scope is exactly these four pages, and the drift population is
unchanged at 28.** **Control:** `git diff --name-only` shows **only** the four
`library/learn/reading-ken/` paths listed above; `scripts/gen-doc-status.sh
--check` before and after both exit 1 with **byte-identical** 28-path output.
⭐ The ledger's sortedness and exact-set checks run **before** the drift check,
so any stray manifest or ledger write changes *which error* the script reports —
an unchanged block is positive evidence you stayed in scope.

**AC-5 — no broken link or anchor**, and ⛔ **no heading/anchor of any of the
four is renamed or removed** unless a repair genuinely requires it. ⚠ These
chapters cite **each other** as well as being cited from outside the slice; a
moved anchor breaks a citation no gate checks. If one must move, say so in `D4`
and name what refers to it.

---

## ⛔ Banned scope

- ⛔ **Any ledger, manifest, or `STATUS.md` write**, and
  ⛔ `scripts/gen-source-attestations.sh`.
- ⛔ **No `spec/`, `catalog/`, or `crates/` edit.** ⚠ If a source is itself
  wrong, that is `D4`-**routed**, not repaired here.
- ⛔ **No new CI gate or test asserting facts about source, catalog, or doc
  lines** (operator test policy).
- ⛔ **No fifth page.** ⚠ Specifically ⛔ **not**
  `04-effects-capabilities-and-authority.md`
  (reconciled in `DOC-CAP-ASBUILT`, and its residual is its own slice),
  ⛔ not `06-execution.md` ([[DOC-ASBUILT-EXECUTION]]), ⛔ not `fragments.md`
  ([[DOC-ASBUILT-FRAGMENTS]], merged — ⭐ it is a **source** here, not a target).

---

## Contention

**None.** The doc track runs concurrently with build work by standing operator
exception and touches `library/` and `agent/`, never `crates/`.
⚠ [[DOC-ASBUILT-EXECUTION]] and [[DOC-ASBUILT-SOLUTIONS]] may be in flight on
**different** pages. ⛔ Do not touch them and ⛔ do not wait for them. ⭐ They
share sources but not scope — shared rows are re-stamped in phase B, after all
of them land.

---

## Hard stops

⭐ **Route a hard stop; do not push through one.**

1. **A source's current content makes a claim false in a way that needs a spec
   or catalog decision** ⇒ `D4`-route it.
2. **A repair requires moving an anchor another page cites** ⇒ say so before
   doing it.
3. ⭐ **A repaired class turns out to extend to a page OUTSIDE these four** ⇒
   ⛔ do not follow it there. Record it in `D4` as routed and name the page —
   that is a finding for its own slice, and ⚠ it is **valuable**: it tells me
   the remaining slices are not independent.

⏱ **Target: complete or hard-stop inside one turn.** ⛔ Not an AC and ⛔ not
something QA checks. ⚠ **If it overruns, the natural recut is BY SOURCE GROUP
across all four pages** — ⛔ not back into one-WP-per-page, which is what this
grouping exists to prevent.
