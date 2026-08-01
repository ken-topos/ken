# WP frame — `DOC-ASBUILT-AGENTS` (as-built phase A, slice 6 — the last)

**Owner:** Team Doc (`doc-leader` + `doc-author`, Librarian as QA).
**Branch:** `wp/DOC-ASBUILT-AGENTS`. **Size:** M.
**Node:** `docs/program/issues/DOC-ASBUILT-AGENTS.md`.

> ## ⛔⛔ READ [[DOC-ASBUILT-AUDIT]] FIRST — its two-phase law binds this slice
>
> ⭐ **Phase A writes NO ledger, and the gate stays red.** ⛔ No
> `library/manifest.toml`, no `library/SOURCE-ATTESTATIONS`, no
> `library/STATUS.md`, ⛔ no `gen-source-attestations.sh` — for any reason.
> ⚠ A red `--check` is **not** a rejection of this candidate.
>
> ⭐⭐ **This is the final phase-A slice. When it merges, phase B is releasable
> and the ledger is re-stamped once, terminally.** ⛔ That does not make it
> permissible to re-stamp anything here.

## Fixed inputs

| input | value |
|---|---|
| **base** | ⭐ **`origin/main` as of when you cut the branch, provided it contains this frame AND [[DOC-ASBUILT-FRAGMENTS]] has landed.** ⛔ Not a SHA copied from this table. |
| the pages | **thirteen**: `library/README.md`, `library/agents/README.md`, `library/agents/evaluations/README.md`, `library/agents/core/{write-ken,read-ken,proof-and-trust,toolchain}.md`, `library/agents/tasks/{author-package,diagnose,effects-and-capabilities,prove-or-repair,read-review,write-program}.md` |
| their drifted sources | ⭐ **7 distinct paths / 18 citations** — ⚠ **re-derive at your base** |
| campaign law | [[DOC-ASBUILT-AUDIT]] |

---

## ⭐⭐ THIRTEEN PAGES, SEVEN SOURCES — WORK THE SOURCES, NOT THE PAGES

⛔ **Do not read "thirteen pages" as a large slice.** Twelve of the thirteen cite
**exactly one** drifted source. The clusters:

| source | pages it settles | cites |
|---|---|---|
| `docs/program/07-catalog-style-guide.md` | `write-ken` (×2), `author-package` (×3), `read-review` | **6** |
| `spec/30-surface/36-effects.md` | `read-ken`, `write-program`, `effects-and-capabilities` | 3 |
| `docs/program/12-documentation-program.md` | `library/README`, `agents/README`, `evaluations/README` | 3 |
| `catalog/guide/proof-techniques.ken.md` | `write-ken`, `prove-or-repair` | 2 |
| `spec/40-runtime/42-evaluation.md` | `toolchain`, `diagnose` | 2 |
| `catalog/guide/surface-reference.ken.md` | `write-ken` | 1 |
| `spec/60-security/64-trust-model.md` | `proof-and-trust` | 1 |

⇒ ⭐ **One read settles up to three pages.** Organize the whole WP this way.

---

## ⭐⭐ THE STAKES: THESE ARE INSTRUCTIONS AGENTS FOLLOW TO WRITE KEN

⚠ **This corpus is not prose a person skims and forgives.** `write-ken.md` tells
an agent how to write a package; `author-package.md` how to lay one out;
`prove-or-repair.md` how to discharge an obligation; `diagnose.md` how to read a
failure.

⇒ ⭐ **A stale instruction here propagates into authored code and proofs**, and
the failure surfaces much later as a rejected candidate whose author was
faithfully following the documentation. ⛔ That makes "close enough" a worse
outcome here than on any explanatory page.

⇒ ⭐ **The `D1` question is therefore: would an agent following this instruction
today produce something the current catalog and spec accept?** ⛔ Not merely
*"does this sentence still parse as true."*

> ### ⚠ THE STYLE GUIDE IS THE HIGHEST-YIELD READ IN THE CAMPAIGN'S TAIL
>
> It is the **most-cited drifted source in the whole corpus** (9 consumers) and
> appears **6×** in this slice. A changed style rule reads as ordinary normative
> prose about how Ken code is written — ⛔ nothing flags it, and every agent
> authoring a package inherits it.

---

## ⭐ The expected drift count is **28**, and this slice cannot move it

⛔ None of these thirteen is attested, so editing them adds no row.
`AC-4` expects the before-and-after drift blocks **identical at 28**.
⛔ 29 means something was written that should not have been; ⛔ 27 means the base
predates slice 1.

---

## Deliverables

**D1 — per-source claim reconciliation, all 7 sources, across every page that
cites them.** Read each source **once** at its current blob, then for **each
citing page** state every instruction or claim derived from it and whether it is
still true. ⭐ **Organize `D1` BY SOURCE.** ⭐ Cite the anchor you actually read.
⚠ `07-catalog-style-guide.md` is cited **6×** — each occurrence owes its own
entry.

**D2 — repair what is false**, in these thirteen pages only.

**D3 — cross-page sweep per repaired claim class**, across all thirteen. ⚠ The
three-page clusters make this concrete: if `12-documentation-program.md`
falsifies a claim in `library/README.md`, ⭐ **check the other two READMEs
before closing it.**

**D4 — a closed report:** still-true / repaired / **routed**, ⭐ per (source,
page) pair.

---

## Acceptance criteria

**AC-1 — all 7 sources addressed against current blobs, for each page that
cites them**, every repeated citation at each occurrence. **Control:** quote
what you read. ⛔ Not from this frame, ⛔ not from memory.

**AC-2 — every instruction or claim these thirteen pages make about a drifted
source is true at your base**, in the operative sense above: an agent following
it today would produce something the current catalog and spec accept.

**AC-3 — cross-page closure per repaired class**, across all thirteen.
**Control:** name the class and show the sweep, ⛔ not only the edited lines.

**AC-4 — scope is exactly these thirteen pages, drift population unchanged at
28.** **Control:** `git diff --name-only` shows only those thirteen paths;
`scripts/gen-doc-status.sh --check` before and after both exit 1 with
**byte-identical** 28-path output. ⭐ The ledger's sortedness and exact-set
checks run **before** the drift check, so an unchanged block is positive
evidence you stayed in scope.

**AC-5 — no broken link or anchor.** ⚠ These pages cross-link heavily
(`agents/README.md` indexes `core/` and `tasks/`). ⛔ Do not rename or remove a
heading unless a repair requires it.

---

## ⛔ Banned scope

- ⛔ **Any ledger, manifest, or `STATUS.md` write**, and
  ⛔ `scripts/gen-source-attestations.sh`. ⚠ **Especially here** — this is the
  last phase-A slice, and "we're about to re-stamp anyway" is ⛔ **not** a
  licence to start.
- ⛔ **No `spec/`, `catalog/`, or `crates/` edit** — a wrong source is
  `D4`-routed. ⚠ This includes `docs/program/07-catalog-style-guide.md` itself:
  if the **style guide** is wrong, ⛔ that is a route, not a repair.
- ⛔ **No `agent/` edit.** ⚠ `library/agents/**` is documentation *for* agents;
  `agent/**` is the federation's own playbook corpus. ⭐ **Different tree, not
  in this WP.**
- ⛔ **No new CI gate or test asserting facts about source, catalog, or doc
  lines** (operator test policy).
- ⛔ **No fourteenth page.**

---

## Contention

**None.** The doc track runs concurrently with build work by standing operator
exception and touches `library/` and `agent/`, never `crates/`.
⚠ Other phase-A slices may be in flight on **different** pages. ⛔ Do not touch
them and ⛔ do not wait for them.

---

## Hard stops

⭐ **Route a hard stop; do not push through one.**

1. **A source's current content makes an instruction wrong in a way that needs a
   spec or catalog decision** ⇒ `D4`-route it. ⛔ Do not invent the corrected
   instruction on your own authority — ⚠ **an invented instruction here is worse
   than a stale one**, because it is confidently wrong and nothing upstream
   backs it.
2. **The style guide itself appears wrong** ⇒ route it.
3. **A repaired class extends to a page outside these thirteen** ⇒ ⛔ do not
   follow it there; record it in `D4` and name the page.

⏱ **Target: complete or hard-stop inside one turn.** ⛔ Not an AC and ⛔ not
something QA checks. ⚠ If it overruns, the recut is **by source cluster** — the
table above is already the cut list.
