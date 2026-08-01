# WP frame — `DOC-ASBUILT-FRAGMENTS` (as-built phase A, slice 1)

**Owner:** Team Doc (`doc-leader` + `doc-author`, Librarian as QA).
**Branch:** `wp/DOC-ASBUILT-FRAGMENTS`. **Size:** M.
**Node:** `docs/program/issues/DOC-ASBUILT-FRAGMENTS.md`.

> ## ⛔⛔ READ FIRST — THE GATE IS RED AND STAYS RED
>
> `scripts/gen-doc-status.sh --check` **exits 1 today**, before you touch
> anything, with **27 drifted cited sources**. ⭐ **Your slice does not fix
> that, and is not expected to.** The ledger is re-stamped once, terminally, in
> phase B — see [[DOC-ASBUILT-AUDIT]].
>
> ⛔ **A red `--check` is NOT a rejection of your candidate.** ⚠ QA: do not
> treat gate colour as this slice's acceptance signal; ⭐ the drift block must
> come back **unchanged** (see `AC-5`), which is a different and much stronger
> check.

## Fixed inputs

| input | value |
|---|---|
| **base** | ⭐ **`origin/main` as of when you cut the branch — whatever SHA that is, provided it contains this frame.** ⛔ Not a SHA copied from this table. |
| the page | `library/learn/reading-ken/fragments.md` |
| its drifted sources | **9**, tabulated in the node — ⚠ **re-derive at your base** |
| the campaign law | [[DOC-ASBUILT-AUDIT]] — ⭐ **read it before this frame** |
| ledger | `library/SOURCE-ATTESTATIONS` — ⛔ **read-only in this slice** |

---

## ⭐ Why this page is first

`fragments.md` is cited **as a source** by 7 other documents. ⛔ Editing it moves
its own blob OID and drifts every one of them. Reconciling it first means those
pages absorb its final content inside this campaign, instead of discovering
fresh drift after they are declared done.

⚠ **This holds for a locator-only edit too** — an anchor rename moves the blob
exactly as prose does.

---

## Deliverables

**D1 — per-source claim reconciliation.** For **each** of the 9 drifted sources:
read its **current blob**, find every claim this page derives from it, and state
whether that claim is still true. ⭐ Cite the anchor you actually read.

**D2 — repair what is false.** Correct the claims `D1` finds false, in this page
only.

**D3 — the whole-page sweep.** For each claim class you repair, sweep the
**entire page** for the same class. ⚠ **The prior DOC-CAP-ASBUILT rejection was
exactly this**: the targeted lines were fixed while the same claim survived
elsewhere on the page.

**D4 — a closed report.** For each of the 9: still-true / repaired / **routed**.
⛔ An empty routed list is fine; an *unstated* one is not.

⛔ **`07-catalog-style-guide.md` is cited at TWO anchors** — both owe a `D1`
entry.

---

## Acceptance criteria

**AC-1 — all 9 sources are addressed**, each against its current blob, ⛔ not
against this frame's OIDs and ⛔ not from memory. **Control:** quote what you
read.

**AC-2 — every claim the page states about a drifted source is true at your
base.** ⛔ A claim you did not check is not a claim you may leave.

**AC-3 — whole-page closure per repaired class.** **Control:** name the class
and show the sweep — ⛔ not just the lines you edited.

**AC-4 — scope is this one page.** **Control:** `git diff --name-only` shows
exactly `library/learn/reading-ken/fragments.md`.
⛔ **No `library/manifest.toml` edit, no `library/SOURCE-ATTESTATIONS` edit, no
`library/STATUS.md` regeneration.** ⛔ **No `spec/`, `catalog/`, or `crates/`
edit** — if a source is itself wrong, that is `D4`-routed, not repaired here.

**AC-5 — the drift population is unchanged.** Run `scripts/gen-doc-status.sh
--check` before and after. It exits 1 both times, and **the listed drifted paths
are the same 27, byte-for-byte.**
⭐ **This is the real control, and it is stronger than it looks.** The ledger's
sortedness and exact-set checks run **before** the drift check, so any stray
manifest or ledger edit changes *which error the script reports*. An unchanged
27-path block is positive evidence that you stayed inside `AC-4`.
⚠ **`fragments.md` is itself attested.** Editing it moves its blob, so ⭐ **the
28th entry that would appear is `fragments.md` itself** — ⛔ if your after-run
lists 28, that is expected **only** for `fragments.md` and for nothing else.
**Report which of the two you got and why.**

**AC-6 — no broken link or anchor.** ⚠ This page is dense with
`../../../spec/...#anchor` and catalog links.

**AC-7 — the page's own anchors are preserved.** ⛔ **Do not rename or remove a
heading/anchor unless a repair genuinely requires it** — 7 documents cite into
this page, and a moved anchor breaks their citations. ⚠ If one must move, say so
in `D4` and name the citing pages.

---

## ⛔ Banned scope

- ⛔ **Any ledger or manifest write.** Phase A writes none, for any reason,
  including "the row is obviously fine."
- ⛔ **`scripts/gen-source-attestations.sh`** — that is phase B and running it
  here launders 27 unreviewed claims.
- ⛔ **A new CI gate or test asserting facts about source, catalog, or doc
  lines** (operator test policy).
- ⛔ **Reconciling any other page.** They are their own slices.

---

## Contention

**None.** The doc track runs concurrently with build work by standing operator
exception and touches `library/` and `agent/`, never `crates/`.
⚠ `DOC-CAP-ASBUILT` is in flight on
`library/learn/reading-ken/04-effects-capabilities-and-authority.md` — a
**different** page. ⛔ Do not touch it, and ⛔ do not wait for it.

---

## Hard stops

⭐ **Route a hard stop; do not push through one.**

1. **A source's current content makes the page's claim false in a way that needs
   a spec or catalog decision.** ⇒ `D4`-route it. ⛔ Do not repair the source.
2. **A repair requires moving an anchor 7 documents cite.** ⇒ Say so before
   doing it.

⏱ **Target: complete or hard-stop inside one turn.** A Steward sizing target,
⛔ not an acceptance criterion and ⛔ not something QA checks. **9 sources is
the largest phase-A slice bar two** — if it overruns, the remaining slices get
cut smaller.
