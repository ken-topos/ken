# WP frame — `DOC-ASBUILT-EXECUTION` (as-built phase A, slice 2)

**Owner:** Team Doc (`doc-leader` + `doc-author`, Librarian as QA).
**Branch:** `wp/DOC-ASBUILT-EXECUTION`. **Size:** M/L.
**Node:** `docs/program/issues/DOC-ASBUILT-EXECUTION.md`.

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
| the page | `library/learn/reading-ken/06-execution.md` |
| its drifted sources | **15 distinct paths / 24 citations** — ⚠ **re-derive at your base** |
| campaign law | [[DOC-ASBUILT-AUDIT]] |

---

## ⭐⭐ THE EXPECTED DRIFT COUNT IS **28**, NOT 27 — and that is not a defect

`library/learn/reading-ken/fragments.md` is the **only** `library/` page in the
ledger. Slice 1 edited it, so its blob moved and it joined the drift population.

⇒ ⭐ **From slice 1 onward the baseline is 28**, and it **stays 28** for every
remaining phase-A slice, because ⛔ **no other consuming page is attested** —
editing them adds no row.

**So for this slice specifically:** `AC-4` expects the before-and-after drift
blocks to be **identical at 28**. ⚠ ⛔ A candidate that reports 29 has written
something it should not have; ⛔ one that reports 27 is measuring against a base
that predates slice 1.

---

## Deliverables

**D1 — per-source claim reconciliation, all 15 paths.** Read each at its
**current blob**; for every claim this page derives from it, state whether it is
still true. ⭐ Cite the anchor you actually read. ⚠ Several sources are cited at
**multiple anchors** (`42-evaluation` at 5, `45-native-backend` at 4,
`43-termination` and `44-capacity` at 2 each) — **each anchor owes its own
entry.**

**D2 — repair what is false**, in this page only.

**D3 — whole-page sweep per repaired claim class.** ⚠ This is what sank two
earlier candidates: the named lines were fixed while the same claim survived
elsewhere on the page.

**D4 — a closed report:** still-true / repaired / **routed**, per path.

> ### ⭐ Two claim classes on this page have real oracles — lead with them
>
> **Code sources.** `crates/ken-interp/src/eval.rs`,
> `crates/ken-runtime/src/cranelift_backend.rs`,
> `crates/ken-cli/tests/px4b_native_production.rs` and `.github/workflows/ci.yml`
> are **checked artifacts**. A claim about what the interpreter or backend *does*
> is verifiable, ⛔ not a matter of taste.
>
> **⚠ `spec/90-open-decisions.md` is the highest-yield source here.** Open
> decisions **get settled**. ⇒ A passage describing a decision as *open* may now
> be describing a **settled** one — and that reads as current prose, so nothing
> flags it. ⭐ Check the disposition of every open decision this page mentions,
> ⛔ not merely that the anchor still resolves.
>
> ⚠ `docs/program/issues/CAT-CAPEX.md` also drifted — it is the node whose
> exemplar landed, and `04-effects-capabilities-and-authority.md` was already
> repaired against it. ⭐ Check whether this page carries the same stale
> "no checked exemplar" family.

---

## Acceptance criteria

**AC-1 — all 15 paths and every cited anchor addressed** against current blobs.
**Control:** quote what you read. ⛔ Not from this frame, ⛔ not from memory.

**AC-2 — every claim this page makes about a drifted source is true at your
base.**

**AC-3 — whole-page closure per repaired class.** **Control:** name the class
and show the sweep, ⛔ not only the edited lines.

**AC-4 — scope is one page, and the drift population is unchanged at 28.**
**Control:** `git diff --name-only` shows exactly
`library/learn/reading-ken/06-execution.md`; `scripts/gen-doc-status.sh --check`
before and after both exit 1 with **byte-identical** 28-path output.
⭐ The ledger's sortedness and exact-set checks run **before** the drift check,
so any stray manifest or ledger write changes *which error* the script reports —
an unchanged block is positive evidence you stayed in scope.

**AC-5 — no broken link or anchor**, and the page's own heading inventory is
unchanged unless a repair requires it (⚠ say so in `D4` if it does).

---

## ⛔ Banned scope

- ⛔ **Any ledger, manifest, or `STATUS.md` write**, and
  ⛔ `scripts/gen-source-attestations.sh`.
- ⛔ **No `spec/`, `catalog/`, or `crates/` edit.** ⚠ If a source is itself
  wrong, that is `D4`-**routed**, not repaired here.
- ⛔ **No new CI gate or test asserting facts about source, catalog, or doc
  lines** (operator test policy).
- ⛔ **No other page.** They are their own slices.

---

## Contention

**None.** The doc track runs concurrently with build work by standing operator
exception and touches `library/` and `agent/`, never `crates/`.

---

## Hard stops

⭐ **Route a hard stop; do not push through one.**

1. **A source's current content makes a claim false in a way that needs a spec
   or catalog decision** ⇒ `D4`-route it.
2. **An open decision this page describes has been settled in a way that changes
   the section's argument, not just its wording** ⇒ ⛔ do not rewrite the
   argument on your own authority; route it.

⏱ **Target: complete or hard-stop inside one turn.** ⚠ **This is the largest
phase-A slice — 15 paths against slice 1's 9.** ⛔ Not an AC and ⛔ not something
QA checks; if it overruns, that is my sizing error and the remainder gets cut
per-source-group rather than per-page.
