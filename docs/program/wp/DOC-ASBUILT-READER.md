# WP frame — `DOC-ASBUILT-READER` (as-built phase A, slice 5)

**Owner:** Team Doc (`doc-leader` + `doc-author`, Librarian as QA).
**Branch:** `wp/DOC-ASBUILT-READER`. **Size:** M.
**Node:** `docs/program/issues/DOC-ASBUILT-READER.md`.

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
| the pages | ⭐ **four** (amended, see below): `library/quickstart.md`, `library/introduction.md`, `library/learn/exercises/README.md`, `library/learn/reading-ken/04-effects-capabilities-and-authority.md` |
| their drifted sources | ⭐ **6 distinct paths / 11 citations** (amended) — ⚠ **re-derive at your base** |
| campaign law | [[DOC-ASBUILT-AUDIT]] |

---

> ## ⭐⭐ AMENDED 2026-08-01 — `exercises.md` MOVED TO [[DOC-ASBUILT-SOLUTIONS]]
>
> ⚠ **This slice was five pages / 9 sources / 14 citations. It is now four / 6 /
> 11.** ⛔ `library/learn/exercises/exercises.md` is **NOT yours** — do not touch
> it and do not count it.
>
> **Why:** slice 3 hit a frame hard stop before any edit (`evt_41th5chexqwv`).
> `exercises.md` **04.1 asks what `AFull` "does not yet confine"**, a premise
> `catalog/packages/Capability/Filesystem/Errors.ken.md` has retired. ⇒ ⭐ **An
> exercise and its solution are one artifact split across two files** —
> repairing the solution alone writes a correct answer to a wrong question.
>
> ⭐ **The move was free and it un-shares three sources.** `exercises.md`'s
> drifted sources (`EmptyDec`, `Combinators`, `Property`) were a **strict
> subset** of `solutions.md`'s 11, so slice 3's distinct population did not move;
> ⇒ those three leave this slice entirely and **no source is now read twice
> across the two.**
>
> ⚠ **`library/learn/exercises/README.md` STAYS HERE** — ⛔ it did not move. If a
> slice-3 repair implies one there, it arrives as a `D4` route naming the pair.

## ⭐⭐ THE GENRE: THESE ARE THE PAGES A PERSON MEETS FIRST

An introduction, a quickstart, and an exercise set make **promises about what
Ken is and what a reader can do next**. ⇒ ⭐ A stale claim here is not one wrong
sentence — it is a **wrong first impression**, and it is the least likely to be
caught later, because nobody re-reads the introduction.

⚠ **They share claim classes with each other.** ⇒ ⭐ **`D3`'s sweep is
cross-page**: a promise repaired in `introduction.md` very likely appears in
`quickstart.md` too. ⛔ Repairing one and leaving the sibling is the defect that
sank two earlier candidates in this campaign.

> ### ⚠⚠ `04-effects…md` IS IN SCOPE — `DOC-CAP-ASBUILT` DID NOT CLEAR IT
>
> That WP repaired this page's **capability-exemplar** claims and merged. ⛔ It
> did **not** address its other two drifted sources —
> `spec/30-surface/36-effects.md` (cited **3×** here) and `fragments.md`, whose
> blob slice 1 then moved.
>
> ⭐ **A merged neighbour WP is a clearance only for the axes it named.** ⛔ Do
> not treat this page as already reconciled, and ⛔ do not re-open the
> capability-exemplar claims it did settle — if you believe one is now wrong,
> that is a `D4` route, not a repair.

---

## ⭐ The expected drift count is **28**, and this slice cannot move it

⛔ None of these four pages is attested, so editing them adds no row.
`AC-4` expects the before-and-after drift blocks **identical at 28**.
⛔ 29 means something was written that should not have been; ⛔ 27 means the base
predates slice 1.

---

## Deliverables

**D1 — per-source claim reconciliation, all 6 sources, across all four pages.**
Read each source **once** at its current blob, then per page state every claim
derived from it and whether it is still true. ⭐ **Organize `D1` BY SOURCE**, not
by page. ⭐ Cite the anchor you actually read.
⚠ **`spec/30-surface/36-effects.md` is cited 4× (3 in `04-effects` alone) and
`spec/00-overview.md` 2× — each anchor owes its own entry.**

**D2 — repair what is false**, in these four pages only.

**D3 — cross-page sweep per repaired claim class**, across all four.

**D4 — a closed report:** still-true / repaired / **routed**, ⭐ per (source,
page) pair.

> ### ⭐ Where the oracles are
>
> ⚠ **AMENDED — the three catalog packages left with `exercises.md`.**
> `EmptyDec`, `Combinators` and `Property` are now [[DOC-ASBUILT-SOLUTIONS]]'s
> alone. ⭐ Your one checked-code source is
> `catalog/guide/decomposition-abstraction.ken.md`, cited by `quickstart.md` — a
> claim about what it provides is verifiable, ⛔ not a matter of taste.
>
> ⚠ **`fragments.md` is cited by `04-effects` and `exercises/README.md`, and
> slice 1 just repaired it.** Read it at its **current** blob — a claim
> inherited from the **old** text is now false while reading as current.
>
> ⚠ **`spec/00-overview.md` is the sharpest risk for `introduction.md`.** An
> overview is exactly where a project's self-description drifts, and
> `introduction.md` is downstream of it. ⭐ Check what the overview now *claims
> Ken is*, ⛔ not merely that the anchor resolves.

---

## Acceptance criteria

**AC-1 — all 6 sources addressed against current blobs, for each page that
cites them**, every multi-anchor citation at each anchor. **Control:** quote
what you read. ⛔ Not from this frame, ⛔ not from memory.

**AC-2 — every claim any of these four pages makes about a drifted source is
true at your base.**

**AC-3 — cross-page closure per repaired class.** **Control:** name the class
and show the sweep across all four, ⛔ not only the edited lines.

**AC-4 — scope is exactly these four pages, drift population unchanged at 28.**
**Control:** `git diff --name-only` shows only those four paths;
`scripts/gen-doc-status.sh --check` before and after both exit 1 with
**byte-identical** 28-path output. ⭐ The ledger's sortedness and exact-set
checks run **before** the drift check, so an unchanged block is positive
evidence you stayed in scope.

**AC-5 — no broken link or anchor**, and ⛔ no heading/anchor renamed or removed
unless a repair requires it. ⚠ `quickstart.md` and `introduction.md` are portal
pages that other documents link into.

---

## ⛔ Banned scope

- ⛔ **Any ledger, manifest, or `STATUS.md` write**, and
  ⛔ `scripts/gen-source-attestations.sh`.
- ⛔ **No `spec/`, `catalog/`, or `crates/` edit** — a wrong source is
  `D4`-routed.
- ⛔ **No new CI gate or test asserting facts about source, catalog, or doc
  lines** (operator test policy).
- ⛔ **No fifth page**, and specifically ⛔ **neither** `exercises/solutions.md`
  **nor** `exercises/exercises.md` — ⚠ **both are [[DOC-ASBUILT-SOLUTIONS]]'s**
  as of the amendment above — and ⛔ none of the four chapters in
  [[DOC-ASBUILT-CHAPTERS]]. ⭐ `exercises/README.md` **is** yours; if a repair to
  it implies one in the exercise/solution pair, `D4`-route it and name the pair.

---

## Contention

**None.** The doc track runs concurrently with build work by standing operator
exception and touches `library/` and `agent/`, never `crates/`.
⚠ Other phase-A slices may be in flight on **different** pages. ⛔ Do not touch
them and ⛔ do not wait for them.

---

## Hard stops

⭐ **Route a hard stop; do not push through one.**

1. **A source's current content makes a claim false in a way that needs a spec
   or catalog decision** ⇒ `D4`-route it.
2. **A repair to `04-effects` would re-open a capability claim `DOC-CAP-ASBUILT`
   settled** ⇒ ⛔ do not; route it.
3. **A repaired class extends to a page outside these four** ⇒ ⛔ do not follow
   it there; record it in `D4` and name the page. ⚠ **That is valuable** — it
   tells me the remaining slices are not independent.

⏱ **Target: complete or hard-stop inside one turn.** ⛔ Not an AC and ⛔ not
something QA checks. ⚠ If it overruns, the recut is **by source group across
all four pages**, ⛔ not back into one-WP-per-page.
