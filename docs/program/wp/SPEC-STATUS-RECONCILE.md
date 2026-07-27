# `SPEC-STATUS-RECONCILE` — two status vocabularies that do not correspond

**Owner:** spec enclave (spec-leader + spec-author + conformance-validator)
**Size:** M
**Node:** `docs/program/issues/SPEC-STATUS-RECONCILE.md`
**Fixed inputs measured at `origin/main = 94c2e67d`.** Re-derive before cutting;
report any drift rather than assuming.

## 1. The problem, as measured — not as characterized

`spec/SPEC-PROGRESS.md` is the spec's status backbone. It is what a sequencing
pass reads to decide what is releasable. **It disagrees with the chapters it
indexes, at scale, and the disagreement is not staleness.**

### 1a. The backbone's own table

| measurement at `94c2e67d` | value |
|---|---|
| parseable status rows in the outline table | **48** |
| rows reading `DRAFT` | **47** |
| rows reading anything else | **1** (`_notes/analysis-digest.md` = `DONE`) |
| rows reading `REVISED` | **0** |

The declared ladder is at line 41:

> `Status: TODO · DRAFT (first pass written) · REVISED (refined) · DONE`

⭐ **`REVISED` has zero uses across 48 rows** — its only two occurrences in the
whole file are that legend and an explicit *instruction* at line 147 to use it
(*"Raise chapters from DRAFT → REVISED as the enclave/teams validate them"*).
⚠ **A rung that no one reaches despite a standing instruction to reach it is
evidence the ladder is wrong, not evidence that everything is a draft.**

### 1b. What the chapters actually say about themselves

**60 of 63 `spec/**/*.md` files carry a `> Status:` self-declaration.** That
vocabulary is **not a maturity ladder at all**:

| shape | approx. count | example |
|---|---|---|
| `DRAFT v0` (+ parenthetical scope) | ~25 | `DRAFT v0 (CAT-2).` |
| **`<WP-ID> elaborated`** | ~16 | `K1 elaborated`, `V4 elaborated`, `L5 elaborated`, `Sec2 elaborated` |
| **`impl-ready (<WP-ID>)`** | ~8 | `impl-ready (L6)`, `impl-ready (B3)` |
| **`Normative`**, often scope-qualified | ~6 | `Normative for the policy shape and the binding guarantee` |
| `Elaborated (X2 contract)` | 2 | `40-runtime/41-values.md` |
| one-offs | 3 | `living document — for the operator`, `Phase 1 — registry audit`, `DRAFT v3 — CAPSTONE COMPLETE` |

## 2. ⭐⭐ THE DESIGN JUDGMENT, FRONT-LOADED — this is not a row sweep

**The two files are not tracking the same quantity, and no amount of row-editing
fixes that.** The chapters record **two orthogonal axes**:

1. **Provenance** — *which work package last elaborated this chapter*
   (`K1 elaborated`, `impl-ready (L6)`, `V4 elaborated`).
2. **Binding force, with scope** — *how far this text binds, and over what*
   (`Normative for the built-in/prelude/package line and the …`).

`SPEC-PROGRESS`'s ladder is a **third** quantity: **drafting-effort maturity**,
from the bootstrap phase. The file's own header says so — it describes itself as
the resume protocol for *"a long-running spec-drafting effort"* and states
*"this is the spec-author bootstrap the Opus Spec enclave would do; the real
enclave later refines it."*

⇒ ⛔ **A reconcile is UNDEFINED until someone defines the correspondence.**
There is no principled mapping from `K1 elaborated` onto `DRAFT | REVISED |
DONE`, because provenance and maturity are different questions. **Defining that
correspondence — or ruling that there is none and replacing the instrument — is
the deliverable.** The row sweep falls out of it.

⚠ **Do not resolve this by inventing a mapping to make the sweep possible.**
Inventing a rung assignment for 47 rows produces a file that is *precise and
unfounded*, which is worse than the current one because it reads as maintained.

### 2a. Candidate shapes — you choose and justify; ⛔ I am not mandating one

- **(A) Keep the ladder, define the mapping.** Requires a defensible rule taking
  provenance + binding force → one rung, and an answer for what `REVISED` means
  such that it is reachable.
- **(B) Replace the ladder with the axes the chapters already use.** The status
  column becomes provenance + binding force, i.e. the backbone stops maintaining
  a vocabulary nothing else speaks.
- **(C) Stop dual-maintaining.** The chapters are the source of truth for their
  own status; the backbone's table becomes a derived view.

⛔ **(C) has a trap and you must address it if you pick it:** a derived table is
a mechanism, and 60-of-63 coverage means **3 files have no declaration at all**.
A derivation silently drops them. Name them and say what they get.

## 3. ⛔ BANNED SHAPE — do not build a checker

**Operator test policy, 2026-07-26, verbatim:** *"Test oracles that assert facts
about source code, catalog, or documentation lines are an invitation for failure
and delay. Tests should focus on behavior."*

⇒ ⛔ **Do NOT frame, propose, or build a CI check that greps `SPEC-PROGRESS`
rows against chapter headers.** That is exactly the banned shape. **The
deliverable is a corrected, coherent artifact — not a new gate.**

⚠ The same rule forbids the tempting weaker version: a script that "just
reports" drift and is wired into CI. If it can go red, it is a gate.

## 4. Deliverables

- **`D1`** — the **correspondence ruling**: which shape (2a) is adopted and why,
  written into `SPEC-PROGRESS.md` as its operative convention, replacing the
  line-41 legend. ⛔ Edit the legend; do not append a second one beside it.
- **`D2`** — the **per-chapter inventory**: for each of the 63 spec files, its
  declared status (or its absence) and the rung/axis value `D1` assigns it.
- **`D3`** — the **corrected table**, applying `D1`.
- **`D4`** — the **unclassifiable report**: every chapter that `D1` could not
  classify, **by name, with the reason.** ⭐ This is a required deliverable, not
  an exception list — see `AC-4`.
- **`D5`** — the stale **purpose statement**: the header describes a bootstrap
  phase that is over. Correct it to what the file is *for now*, or state
  explicitly that it is retained as historical preamble and mark it as such.

## 5. Acceptance criteria and their controls

- **`AC-1`** — `D1` is stated as a rule that a third party can apply, and is
  applied uniformly.
  **Control:** pick **three** chapters with *different* declaration shapes —
  one `<WP> elaborated`, one `impl-ready (<WP>)`, one scope-qualified
  `Normative` — and show `D1` assigns each its value **by the stated rule**.
  ⛔ Three chapters of the same shape do not exercise the rule.
- **`AC-2`** — every row in the corrected table is **derived from a measured
  chapter declaration**, not from the previous row value.
  **Control:** name at least one row whose value **changed** and one that did
  **not**, and give the chapter text each was read from.
- **`AC-3`** — the `REVISED` problem is answered, not inherited.
  **Control:** either the adopted vocabulary has no unreachable rung, or the
  report states which rung is unreachable **and why that is correct**.
- **`AC-4`** — ⭐ **the unclassifiable set is reported, and reporting zero
  requires evidence.**
  **Control:** if `D4` is empty, show the **3 files that carry no `> Status:`
  declaration** and how `D1` classified them. An empty `D4` with 3 known
  declaration-less files is a **failed** measurement, not a clean one.
- **`AC-5`** — no gate is added.
  **Control:** the diff touches no CI configuration, no `scripts/`, and adds no
  test asserting documentation content. State this as measured, not intended.

## 6. Scope — in and out

**In:** `spec/SPEC-PROGRESS.md`.

⛔ **OUT — and this is the boundary that keeps the WP safe:** **do not edit any
chapter's own `> Status:` line.** The chapters' self-declarations are this WP's
**input**, and rewriting an input to match your output is not a reconcile. If
`D1` shows a chapter's own declaration is wrong, **report it** — do not fix it
here.

⛔ Out: conformance corpus, `90-open-decisions.md`, `docs/adr/`, any crate.

## 7. Contention — and why this is the enclave's, not the doc ring's

`spec/SPEC-PROGRESS.md` is **written by the spec enclave on spec-WP landings** —
most recently `c631841d` (`SPEC-STORE-SPLIT`, 2026-07-27, 2 insertions / 2
deletions), and before that `a97b4304`, `d69819ca`, `30bc5dfd`.

⇒ ⭐ **This was initially scoped to the doc ring and that was wrong.** The doc
track's concurrency exemption rests on **contention-free-ness** (operator,
2026-07-21) — it touches `library/` and `agent/`, not `spec/`. Handing the doc
ring a file the enclave writes on every spec landing would break the exact
premise the exemption stands on. It is also not the doc ring's call: **whether
`K1 elaborated` implies normative force is a spec-authority question.**

⚠ **Live sequencing note:** Team Ergo is routing the enclave a `§2`
blessed-Unicode-identifier completeness proposal. It touches
`spec/30-surface/31-lexical.md`, **not** this file — but if it lands a chapter
status change, this WP's input moves. ⇒ **This WP goes first**; define the
vocabulary before more chapters acquire statuses under the old one.

**No build ring touches `spec/`.** Ergo, Language and Runtime are in
`crates/` and are contention-free with this.

## 8. What to do on a hard stop

Stop before edits and route to me with the concrete fork. ⛔ Do not pick a shape
in 2a by default because the measurement was ambiguous — an ambiguous
measurement is itself the finding, and `D4` is where it belongs.
