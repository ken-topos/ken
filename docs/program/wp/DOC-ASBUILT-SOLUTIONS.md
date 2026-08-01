# WP frame — `DOC-ASBUILT-SOLUTIONS` (as-built phase A, slice 3)

**Owner:** Team Doc (`doc-leader` + `doc-author`, Librarian as QA).
**Branch:** `wp/DOC-ASBUILT-SOLUTIONS`. **Size:** M/L.
**Node:** `docs/program/issues/DOC-ASBUILT-SOLUTIONS.md`.

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
| the pages | ⭐ **BOTH** — `library/learn/exercises/solutions.md` **and** `library/learn/exercises/exercises.md` (amended, see below) |
| its drifted sources | **11 distinct paths / 15 citations** — ⭐ **the union is still 11**; ⚠ **re-derive at your base** |
| campaign law | [[DOC-ASBUILT-AUDIT]] |

---

> ## ⭐⭐ AMENDED 2026-08-01 — `exercises.md` IS IN SCOPE; HARD STOP 3 WITHDRAWN
>
> ⚠ **The original frame put the exercise page out of scope and made "a solution
> is wrong because its exercise is wrong" a hard stop. doc-author fired it
> correctly, before any edit** (`evt_41th5chexqwv`): `exercises.md` **04.1 asks
> what `AFull` "does not yet confine"**, while
> `catalog/packages/Capability/Filesystem/Errors.ken.md` now says `Full` retains
> all seven rights and exercises them **only within its `FsScope`**. ⇒ **The
> question's premise is retired, not just its answer.**
>
> ⭐⭐ **AN EXERCISE AND ITS SOLUTION ARE ONE ARTIFACT SPLIT ACROSS TWO FILES.**
> Repairing only the solution means **writing a correct answer to a wrong
> question** — a worse artifact than the stale pair, because it reads as
> reconciled. ⛔ There is no solution-only repair of 04.1 that is not a fiction.
>
> ⭐ **And the union is free.** Measured at `efa8c5e8`: `exercises.md` cites
> **3** drifted sources — `EmptyDec`, `Combinators`, `Property` — and **all three
> are already among `solutions.md`'s 11.** ⇒ `exercises \ solutions = ∅`. **The
> distinct-source population does not move: 11 before, 11 after.** Only the
> citation count rises, 12 → 15.
>
> ⛔ **This is NOT a new node**, and the alternative was rejected on the node
> gate: the only constraint arguing for one was **this frame's own prose**, which
> the gate names explicitly as ungrounded — *including prose the Steward wrote.*
>
> ⇒ **`exercises.md` is REMOVED from [[DOC-ASBUILT-READER]]**, which drops to
> four pages and 6 distinct sources. ⭐ After this move **no source is read twice
> across the two slices** — the three catalog packages belong wholly to this one.
>
> ⚠ **04.2 comes with it.** doc-author found the same stale family makes 04.2's
> *"no capability-typed catalog fragment"* answer false; it is answerable as
> **False** once the pair is in one scope. ⭐ Sweep for the family, ⛔ do not stop
> at 04.1 and 04.2 because they are the two that were named.

---

## ⭐⭐ THE GENRE IS THE POINT — A STALE CLAIM HERE IS A BROKEN ANSWER

Every other phase-A page *describes* Ken. **This one hands the reader a worked
solution and implies it works.**

⇒ ⭐ **The `D1` question is sharper here than elsewhere.** Not merely *"is this
sentence still accurate?"* but **"would this solution still be a correct answer
against the current package?"** A solution that names a constructor, law, or
combinator the catalog has since renamed or re-signed is **wrong**, ⛔ not merely
dated — and it is wrong in the way most likely to waste a reader's afternoon,
because a worked answer is the last thing anyone thinks to doubt.

⚠ **This does NOT authorize running the exercises as a test suite, or adding
one.** ⛔ New CI gates and tests asserting facts about source, catalog, or doc
lines are banned (operator test policy). The oracle is **reading the current
blob**, not executing anything.

---

## ⭐ The expected drift count is **28**, and this slice cannot move it

`fragments.md` was the only attested `library/` page and slice 1 already edited
it. ⛔ **`solutions.md` is not attested**, so editing it adds no row.

⇒ `AC-4` expects the before-and-after drift blocks **identical at 28**.
⛔ 29 means something was written that should not have been; ⛔ 27 means the base
predates slice 1.

---

## Deliverables

**D1 — per-source claim reconciliation, all 11 paths.** Read each at its
**current blob**; for every claim this page derives from it, state whether it is
still true. ⭐ Cite the anchor you actually read.
⚠ **`spec/30-surface/30-taxonomy.md` is cited at two anchors — each owes its own
entry.**

**D2 — repair what is false**, in these two pages only. ⭐ **Repair an exercise
and its solution as one unit** — a corrected answer under a retired question is
not a repair.

**D3 — sweep per repaired claim class, ACROSS BOTH PAGES.** ⚠ This is what sank
two earlier candidates in this campaign: the named lines were fixed while the
same claim survived elsewhere on the page.
⭐ **Here the class is often a *solution shape*, not a phrase** — if one worked
answer leaned on a stale signature, sweep for **every** answer that leans on the
same package, not merely for the same words.
⭐⭐ **And the sweep is now explicitly cross-page**: a class repaired in a
solution must be checked against **its exercise's premise**, and vice versa. The
04.1/04.2 pair is the worked example, ⛔ not the boundary.

**D4 — a closed report:** still-true / repaired / **routed**, per path.

> ### ⭐ Where the real oracles are
>
> **Five sources are checked catalog code** — `EmptyDec`, `Combinators`,
> `Transport`, `Property`, and (via slice 2's neighbourhood) the runtime pair
> `cranelift_backend.rs` / `px4b_native_production.rs`. A claim about what a
> package *provides* is verifiable against its current blob, ⛔ not a matter of
> taste.
>
> **⚠ `spec/90-open-decisions.md` is the highest-yield source here**, exactly as
> in slice 2. Open decisions **get settled**. ⇒ A passage treating one as *open*
> may now be describing a **settled** one, and that reads as current prose, so
> ⛔ nothing flags it. ⭐ Check the **disposition** of every open decision this
> page relies on, ⛔ not merely that the anchor resolves.
>
> ⚠ **`docs/program/issues/CAT-CAPEX.md` drifted** — its exemplar landed, and
> both `04-effects-capabilities-and-authority.md` (in `DOC-CAP-ASBUILT`) and
> slice 2 were checked for the stale **"no checked exemplar"** family. ⭐ Check
> whether an exercise or its solution carries the same family.

---

## Acceptance criteria

**AC-1 — all 11 paths and both `30-taxonomy` anchors addressed** against current
blobs. **Control:** quote what you read. ⛔ Not from this frame, ⛔ not from
memory.

**AC-2 — every claim either page makes about a drifted source is true at your
base**, including the implicit claim that a worked solution is *a correct
answer* ⭐ **and that its exercise asks an answerable question.** ⚠ An exercise
whose premise a source has retired fails this AC even if its solution is
internally consistent with it.

**AC-3 — closure per repaired class, ACROSS BOTH PAGES.** **Control:** name the
class and show the sweep over exercises **and** solutions, ⛔ not only the edited
lines. ⭐ For every repaired solution, state whether its exercise still holds;
for every repaired exercise, state whether its solution followed.

**AC-4 — scope is exactly those two pages, and the drift population is unchanged
at 28.** **Control:** `git diff --name-only` shows exactly
`library/learn/exercises/solutions.md` and
`library/learn/exercises/exercises.md`; `scripts/gen-doc-status.sh --check`
before and after both exit 1 with **byte-identical** 28-path output.
⭐ Neither page is attested, so ⛔ **neither edit can move the count** —
doc-author's pre-edit capture is the anchor: exit 1, 32 lines, 28 rows,
SHA-256 `349d5452…`.
⭐ The ledger's sortedness and exact-set checks run **before** the drift check,
so any stray manifest or ledger write changes *which error* the script reports —
an unchanged block is positive evidence you stayed in scope.

**AC-5 — no broken link or anchor**, and every exercise's identity is preserved.
⛔ **Do not renumber, retitle, add, or remove an exercise** — `exercises/README.md`
(⚠ **still out of scope, in [[DOC-ASBUILT-READER]]**) refers to them, and a
renumber silently breaks a cross-page reference that no gate checks. ⚠ If one
must change, say so in `D4` and name what refers to it.

⭐⭐ **AC-5 IS THE BOUNDARY THAT SURVIVES THE AMENDMENT.** Editing an exercise's
**premise** is now in scope; editing the exercise **set** is not. ⇒ Repair what
04.1 *asks*; ⛔ do not decide that 04.1 should no longer exist, or renumber
around it. **If the honest repair is deletion, that is a `D4` route, not an
edit.**

---

## ⛔ Banned scope

- ⛔ **Any ledger, manifest, or `STATUS.md` write**, and
  ⛔ `scripts/gen-source-attestations.sh`.
- ⛔ **No `spec/`, `catalog/`, or `crates/` edit.** ⚠ If a source is itself
  wrong, that is `D4`-**routed**, not repaired here.
- ⛔ **No new CI gate or test asserting facts about source, catalog, or doc
  lines** (operator test policy) — ⭐ including any harness that runs the
  exercises.
- ⛔ **No third page.** ⚠ `exercises.md` is now **IN** scope (amended above);
  ⛔ `library/learn/exercises/README.md` is **NOT** — it stays in
  [[DOC-ASBUILT-READER]]. ⭐ If a repair here implies one there, `D4`-route it
  and name the pair.

---

## Contention

**None.** The doc track runs concurrently with build work by standing operator
exception and touches `library/` and `agent/`, never `crates/`.
⚠ [[DOC-ASBUILT-EXECUTION]] may still be in flight on
`library/learn/reading-ken/06-execution.md` — a **different** page. ⛔ Do not
touch it and ⛔ do not wait for it. ⭐ They share sources but not scope: the
shared rows are re-stamped in phase B, after both have landed.

---

## Hard stops

⭐ **Route a hard stop; do not push through one.**

1. **A source's current content makes a solution wrong in a way that needs a
   spec or catalog decision** ⇒ `D4`-route it. ⛔ Do not invent a replacement
   answer on your own authority.
2. **An open decision this page relies on has been settled in a way that changes
   what the correct answer *is*** ⇒ route it.
3. ⛔ ~~**A solution is wrong because its exercise is wrong** ⇒ route it; the
   exercise page is out of scope here.~~ **WITHDRAWN 2026-08-01 — this stop
   fired correctly (`evt_41th5chexqwv`) and the frame was wrong, not the
   author.** ⭐ **Repair the pair.** The exercise page is in scope; the only
   surviving boundary is `AC-5` (premise yes, exercise **set** no).
4. **A repair reaches `library/learn/exercises/README.md`** ⇒ ⛔ do not follow it
   there; `D4`-route it and name the pair. ⚠ That page is [[DOC-ASBUILT-READER]].

⏱ **Target: complete or hard-stop inside one turn.** ⛔ Not an AC and ⛔ not
something QA checks; if it overruns, that is my sizing error and the remainder
gets cut per-source-group rather than per-page.
