---
id: SPEC-STATUS-RECONCILE
title: "the spec's two status vocabularies do not correspond — define the correspondence (or replace the ladder), then apply it"
status: ready
owner: spec-enclave
size: M
gate: none
depends_on: []
blocks: []
github: null
origin: Steward-filed 2026-07-27, measured at origin/main 94c2e67d. The backbone status file contradicts the chapters it indexes at scale, and it is a false input to the Steward's own sequencing — it already caused one mis-release (SEC1, released against a DRAFT row for a chapter that was in fact elaborated and seeded). Steward owns the frame and AC/control placement.
---

> ## ▶ THE INSTRUMENT THE SEQUENCING PASS READS IS WRONG
>
> Frame: [`SPEC-STATUS-RECONCILE.md`][f], under `docs/program/wp/`.
> The frame is the executable artifact; this node carries the measurement and
> the program bookkeeping.

## What was measured, at `94c2e67d`

| measurement | value |
|---|---|
| parseable status rows in `spec/SPEC-PROGRESS.md` | **48** |
| rows reading `DRAFT` | **47** |
| rows reading anything else | **1** (`_notes/analysis-digest.md` = `DONE`) |
| rows reading `REVISED` | **0** — despite a standing instruction at line 147 to use it |
| `spec/**/*.md` files total | **63** |
| files carrying a `> Status:` self-declaration | **60** |

## ⛔ WHY THIS IS NOT A ROW SWEEP

The chapters do **not** record a maturity rung. They record **provenance**
(`K1 elaborated`, `impl-ready (L6)`, `V4 elaborated`) and **binding force with
scope** (`Normative` for X). `SPEC-PROGRESS`'s ladder — `TODO · DRAFT ·
REVISED · DONE` — is a **third** quantity: drafting-effort maturity, from a
bootstrap phase the file's own header says is over.

⇒ **There is no principled mapping from `K1 elaborated` onto `DRAFT|REVISED|
DONE`.** Defining the correspondence, or ruling that there is none and
replacing the instrument, **is the deliverable**. The row sweep falls out of it.

⭐ **`REVISED`'s zero uses across 48 rows is the tell.** A rung nobody reaches
despite an explicit instruction to reach it is evidence the ladder is wrong —
not evidence that everything is a draft.

## ⛔ THE BANNED SHAPE

Operator test policy (2026-07-26): *"Test oracles that assert facts about source
code, catalog, or documentation lines are an invitation for failure and delay.
Tests should focus on behavior."*

⇒ ⛔ **No CI checker greping rows against chapter headers.** The deliverable is
a **corrected artifact**, not a new gate. This includes the weaker version — a
"just reports drift" script wired into CI is a gate if it can go red.

## ⭐ WHY THE ENCLAVE AND NOT THE DOC RING — a corrected premise

This was initially scoped to the doc ring, on the reasoning that the doc ring
was idle and this is an as-built accuracy problem. **That was wrong on two
counts, both measured:**

1. **Contention.** `spec/SPEC-PROGRESS.md` is written by the **spec enclave on
   spec-WP landings** — `c631841d` (today), `a97b4304`, `d69819ca`, `30bc5dfd`.
   The doc track's concurrency exemption rests on **contention-free-ness**
   (operator, 2026-07-21) — `library/` and `agent/`, not `spec/`. Handing the
   doc ring this file breaks the premise the exemption stands on.
2. **Authority.** Whether `K1 elaborated` implies normative force is a
   **spec-authority** question. The doc ring cannot rule it.

⚠ The doc ring's idleness is real and remains unaddressed by this node.

## Sequencing

⭐ **This goes before the incoming `§2` proposal.** Team Ergo is routing the
enclave a blessed-Unicode-identifier completeness proposal against
`spec/30-surface/31-lexical.md`. Different file — but if it lands a chapter
status change, this node's input moves. Define the vocabulary first.

**Contention-free with all three active build rings** (Ergo, Language, Runtime
are in `crates/`).

[f]: ../wp/SPEC-STATUS-RECONCILE.md
