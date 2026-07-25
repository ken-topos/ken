---
id: DOC-GATE-WIRE-BINDING
title: "validation-gate registry: bind the kind-vocabulary RULE to its GATE by registering it as a VALIDATION_GATES row"
status: merged
owner: verify
size: XS
gate: none
depends_on: [DOC-GATE-CONTROL-BINDING]
blocks: []
github: https://github.com/swe-toolkit/ken/pull/933
origin: adversary finding I1 on DOC-GATE-CONTROL-BINDING (f0ceb702), side thread thr_2seh2bm1kr5mh evt_5ezj67aakm4he, 2026-07-24. Steward-filed (agents cannot create tracked work per COORDINATION §2); Steward triage = CONFIRMED by measurement, and the overclaim is a Steward defect for the THIRD time this session.
---

> ## The fix binds the RULE to a test. It does not bind the RULE to the GATE.
>
> `DOC-GATE-CONTROL-BINDING` made deleting a detector's **rule** loud. Deleting
> its **invocation** is still silent. Two of two wires cut clean, measured.

## The measurement

From a clean tree at `f0ceb702`, each mutation restored byte-identical:

| mutation | result |
|---|---|
| delete `bad.extend(invalid_kind_violations(…))` from `check_manifest_completeness` (`:471`) | **24 passed, 0 failed** |
| replace `status_record_population_violations(&entries)` at `:638` with `Vec::new()` | **24 passed, 0 failed** |

In both runs the detector's own committed test still **passes**, because it
calls the detector directly (`:516`, `:697`). The rule survives; its enforcement
against the real manifest does not.

★ **No observational test can catch this**, and that is the whole point. Both
production call sites are **vacuously satisfied today** — the real manifest has
no out-of-vocabulary `kind` and exactly one `status` record — so a wired check
and an unwired one produce **identical output on every input the suite will ever
see**. This needs a *structural* binding, not a better assertion.

⚠ **The obvious close does not exist.** A committed test running the real gate
against a fixture manifest would bind the wire directly, but `repo_root()`
(`:63-71`) is `env!("CARGO_MANIFEST_DIR")` — a **compile-time** constant — so
every gate is hard-bound to the actual repository manifest and cannot be pointed
at a fixture without a refactor. Verified.

## Deliverable — the file's own mechanism, already executed once

Register the vocabulary check as a `VALIDATION_GATES` row:

- token `document-kind`, applying to every record, `run: check_document_kinds`
  (a two-line runner over the existing `invalid_kind_violations` detector);
- add `document-kind` to all **14** `library/manifest.toml` records.

Exactly what `96ab2b4b` did for `transport-delimiter` — verified present as
**1** registry row and **14** manifest occurrences.

That makes deletion loud **twice**:

1. **Delete the runner** while the row names it ⇒ `E0425` **at the registry
   line** — the parent WP's proven reverse-dependency binding.
2. **Delete the row** ⇒ all 14 records declare a token absent from `known`, so
   `gate_validation_tokens_are_closed_and_match_applicable_checks` fails on
   *both* the unknown-token arm **and** the `declared != required` arm.

## ⛔ Scope — the status-population rule is deliberately EXCLUDED

`status_record_population_violations` is a **global invariant over the record
set**, not a per-record validation claim. A registry row for it would be a
category error, and the Adversary explicitly declined to ask for one.

⇒ **Accept that residual and state the claim accurately.** Do not refactor
`repo_root()` for it. ⛔ Do not fold in the `run: fn(&DocEntry)` refactor —
declined three times now.

## Acceptance criteria

- **AC-1 — the row is registered** and `check_document_kinds` runs the existing
  detector against the real manifest. Behaviour unchanged: today's 14 records
  produce no rejection.
- **AC-2 — ⭐ BOTH deletion modes are loud, each demonstrated and LOCATED.**
  (a) delete the runner ⇒ `E0425` **at the registry line**, quoted;
  (b) delete the row ⇒ name the failing test **and which arm** fired.
  Restore byte-for-byte and re-verify green after each.
- **AC-3 — the invocation wire is now bound.** Repeat the adversary's exact
  mutation — remove the vocabulary check's production call — and show it now
  **reddens**. This is the finding; closing it without this check is not closing
  it.
- **AC-4 — no vacuity.** State which line you removed for each proof.

## ⚠ Fair attribution — this is a strict improvement, not a failed fix

**Every gate body in this file shares this residual** — emptying `check_links`
reddens nothing either. The pre-`f0ceb702` state had **both** deletion modes
silent; `DOC-GATE-CONTROL-BINDING` converted one of two to loud. ⛔ Do not read
the parent WP as having failed. It did exactly what was asked.

## ⚠ The Steward overclaim — THIRD instance this session, same direction

My merge summary said *"the remedy for the orphaning defect is no longer itself
orphanable"* and *"deleting either rule now reddens at a named artifact instead
of vanishing silently."* **The first clause is false and the second is true only
for the rule, not the invocation.**

★ The **PR description was accurate** — *"removing each detector's rule makes its
named committed test fail."* **It was the summary generalization that outran the
measurement**, which is a distinct and easier failure than the earlier one: the
narrow sentence was right there and I broadened it while restating it.

⇒ Prior instances today: claiming a positive control that did not exist
(`DOC-GATE-RECORD-AXIS`), and a re-anchor guard testing path overlap as a proxy
for content replay. See
`agent/memory/roles/steward/the-publish-description-is-the-one-artifact-no-reviewer-reviews.md`
and `agent/memory/build/a-check-that-measures-a-proxy-passes-for-the-wrong-reason.md`.
