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
| the page | `library/learn/exercises/solutions.md` |
| its drifted sources | **11 distinct paths / 12 citations** — ⚠ **re-derive at your base** |
| campaign law | [[DOC-ASBUILT-AUDIT]] |

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

**D2 — repair what is false**, in this page only.

**D3 — whole-page sweep per repaired claim class.** ⚠ This is what sank two
earlier candidates in this campaign: the named lines were fixed while the same
claim survived elsewhere on the page.
⭐ **On this page the class is often a *solution shape*, not a phrase** — if one
worked answer leaned on a stale signature, sweep for **every** answer that leans
on the same package, not merely for the same words.

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

**AC-2 — every claim this page makes about a drifted source is true at your
base**, including the implicit claim that a worked solution is *a correct
answer*.

**AC-3 — whole-page closure per repaired class.** **Control:** name the class
and show the sweep, ⛔ not only the edited lines.

**AC-4 — scope is one page, and the drift population is unchanged at 28.**
**Control:** `git diff --name-only` shows exactly
`library/learn/exercises/solutions.md`; `scripts/gen-doc-status.sh --check`
before and after both exit 1 with **byte-identical** 28-path output.
⭐ The ledger's sortedness and exact-set checks run **before** the drift check,
so any stray manifest or ledger write changes *which error* the script reports —
an unchanged block is positive evidence you stayed in scope.

**AC-5 — no broken link or anchor**, and every exercise's identity is preserved.
⛔ **Do not renumber or retitle an exercise** — `exercises.md` and
`exercises/README.md` refer to them, and a renumber silently breaks a cross-page
reference that no gate checks. ⚠ If one must change, say so in `D4` and name
what refers to it.

---

## ⛔ Banned scope

- ⛔ **Any ledger, manifest, or `STATUS.md` write**, and
  ⛔ `scripts/gen-source-attestations.sh`.
- ⛔ **No `spec/`, `catalog/`, or `crates/` edit.** ⚠ If a source is itself
  wrong, that is `D4`-**routed**, not repaired here.
- ⛔ **No new CI gate or test asserting facts about source, catalog, or doc
  lines** (operator test policy) — ⭐ including any harness that runs the
  exercises.
- ⛔ **No other page**, and that explicitly includes `exercises.md`. ⚠ If a
  solution is wrong because the **exercise** is wrong, `D4`-route it — the
  exercise page is its own slice.

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
3. **A solution is wrong because its exercise is wrong** ⇒ route it; the
   exercise page is out of scope here.

⏱ **Target: complete or hard-stop inside one turn.** ⛔ Not an AC and ⛔ not
something QA checks; if it overruns, that is my sizing error and the remainder
gets cut per-source-group rather than per-page.
