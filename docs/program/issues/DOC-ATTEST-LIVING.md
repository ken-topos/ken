---
id: DOC-ATTEST-LIVING
title: "attesting living tracker files makes every routine WP status flip redden the currency gate"
status: ready
owner: doc
size: S
gate: none
depends_on: []
blocks: []
github: null
origin: hit in production by the Steward 2026-07-25 on PR #938 (test shard 1/4, registered_record_validation_gates_run). Steward-filed; agents cannot create tracked work (COORDINATION §2). Not a DOC-W2 defect — DOC-W2's citations are correct; the problem is the CLASS of file it cited.
---

> ## The gate is right. The choice of cited source is what needs deciding.
>
> `DOC-W2` added three **issue files** to `library/SOURCE-ATTESTATIONS`:
>
> ```
> docs/program/issues/CAT-CAPEX.md
> docs/program/issues/DOC-W1.md
> docs/program/issues/DOC-W2.md
> ```
>
> Issue files are **living tracker state**, not stable prose. They take a
> `status:` flip on every lifecycle transition, a `github:` number at publish, a
> merge block at close, and retro pointers after. **Every one of those edits
> moves the file's blob OID and reddens
> `registered_record_validation_gates_run`.**

## How it presented

Flipping `DOC-W2` from `active` to `merged` — pure tracker bookkeeping, no claim
touched — failed CI:

```
gen-doc-status: cited source(s) changed since their last attestation —
  the currency claim is no longer backed by evidence for:
  - docs/program/issues/DOC-W2.md
      (attested bb9a7d3a…, actual 5ffe02bd…)
```

⇒ **A WP cannot be marked merged without a library re-attestation fold.** That
coupling is new, it is not written down anywhere, and it will fire on
`CAT-CAPEX` next.

## ⚠ Why this is not just friction — the failure mode is a rubber-stamp

The remediation is `gen-source-attestations.sh`, which deliberately refuses to
install its own output and asks the operator to *"record which changed
sources/pages were revalidated."* That refusal is load-bearing: **bumping the
OID re-asserts that the library's currency claim is still backed.**

★ **The danger is that the recurring case is almost always benign.** A status
flip genuinely does not affect a cited anchor — so the tenth time an agent hits
this, it bumps the row without looking, and the check has become a ritual. The
one time it matters will look identical to the nine before it.

⚠ **It already nearly did.** On the PR #938 fold, the cited anchors were:

| anchor | verdict |
|---|---|
| `DOC-W2.md#1-objective` | byte-identical |
| `DOC-W2.md#5-exit-property-and-the-evaluation-suite` | **CHANGED** — `tt` → `Proved` |

The reasoning *"I only edited frontmatter and prepended a merge block, so the
cited body is untouched"* was **half wrong**, and it is exactly the reasoning the
benign case trains. (The re-attestation was still correct — the citing corpus,
`library/agents/core/proof-and-trust.md`, already rules `Proved` the writable
surface name — but that was established by *checking*, not by assuming.)

## Deliverables

**D1 — decide what should actually be cited.** Three candidate shapes; the
Librarian owns the call:

1. **Cite a stable extract, not the living file.** If the library needs DOC-W2's
   objective and exit property, those could live in a doc the tracker does not
   rewrite. Strongest option if the cited content is genuinely stable prose.
2. **Attest a content subset.** Attest the cited *anchors'* bytes rather than the
   whole-file OID, so frontmatter and appended merge blocks do not perturb it.
   Removes the false positives without weakening the real check. ⚠ Costs a
   generator change — price it before choosing.
3. **Accept the coupling and make it explicit.** Keep whole-file attestation and
   document that closing an attested WP owes a re-attestation fold. ⛔ If this is
   the choice, **D2 is mandatory** — otherwise the ritual failure above is
   guaranteed rather than merely likely.

**D2 — if the coupling stays, put the per-anchor check in the tooling.** The
generator should report *which cited anchors changed*, not just that the file's
OID moved. An agent shown "OID moved" bumps the row; an agent shown "anchor
`#5-exit-property` changed" has to read it. **Move the judgment from discipline
into output.**

**D3 — state the rule wherever WP closure is described** (`COORDINATION.md` §10
or the doc-team playbook): closing a WP whose issue file is attested owes a
ledger fold in the same change.

## Acceptance criteria

- **AC-1 — a status flip on an attested issue file no longer reddens CI
  spuriously**, *or* the owed fold is documented at the point of closure. State
  which of D1's three shapes was chosen **and why the other two were rejected.**
- **AC-2 — the real check still bites.** Whatever replaces the whole-file OID
  must still redden when a **cited anchor's content** genuinely changes.
  ⛔ **Demonstrate this with the `tt` → `Proved` case as the positive control** —
  it is a real, measured instance of the thing that must not slip through.
  *A negative check passes for any reason; without this control, "no more false
  positives" is indistinguishable from "no more checking."*
- **AC-3 — `CAT-CAPEX` is covered.** It is the next attested issue file due a
  lifecycle edit; whatever is decided must handle it without a manual fold, or
  must say plainly that a manual fold is owed.
- **AC-4 — no regression in CI.** `scripts/ken-cargo` scoped to
  `-p ken-cli --test library_documentation_gates`. ⛔ Never `--workspace` or
  `--locked` locally (COORDINATION §12).

## Guardrails

- ⛔ **Do not delete the three rows to make the red go away.** The attestation
  mechanism is correct and DOC-W2's citations are accurate; the question is
  which *class* of file belongs under whole-file currency.
- ⛔ **Do not hand-edit `library/SOURCE-ATTESTATIONS` or `library/STATUS.md`.**
  Both are generated; regenerate and install deliberately.
- This WP does **not** revisit the recursion, `$ref`, or unknown-keyword
  fail-closed behaviour of the schema validator. Unrelated; and the assertion
  defect in that gate is `DOC-GATE-NEEDLE`, tracked separately.

## ⛔ HELD — the fleet is FNSPLIT-only (operator, 2026-07-25)

Filed `ready`, **not released.** The operator has ruled the fleet strictly
single-threaded on `RT-NATIVE-FNSPLIT`, and the doc-track concurrency exception
is **doc-only in the sense of where review attention goes** — proving a disjoint
file set does not earn a slot. ⛔ **Do not re-ask.**

⚠ **Meanwhile the coupling is live**, so any Steward closing an attested WP owes
the fold by hand. The recipe that worked on PR #938: `comm` the changed paths
against the ledger's path column to find *every* affected row, diff each
manifest-cited anchor across both blobs, then install `.proposed` and record the
per-anchor verdict in the commit message.
