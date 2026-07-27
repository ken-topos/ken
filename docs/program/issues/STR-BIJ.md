---
id: STR-BIJ
title: "the String/List Char 'bijection' over-claim (adversary A1 + A2)"
status: merged
owner: spec-enclave
size: S
gate: none
depends_on: []
blocks: []
github: https://github.com/swe-toolkit/ken/pull/1096
origin: adversary findings A1 + A2
---

> ✅ **HOLD DISCHARGED 2026-07-27 (Steward).** The `DOC-VALIDATION-BINDING`
> hold (`evt_tmv40vgtg63k`, 2026-07-24) is spent: that WP is **`merged`**, so it
> is no longer the live holder of the `library/` ledger axis. The re-derivation
> it required has been done — see *Sequencing* below for the measured population
> and the current attested OID. **This node is releasable to the spec enclave**
> on the Handoff Gate.

Honesty erratum on landed prose: spec normative over-claim (A1) and a catalog
title over-claim (A2), both CONFIRMED — the adversary itself supplied the
refutation that keeps this out of soundness (see the brief §4). Wording-only,
zero semantic change.

**The frame and the brief are both on `main`** — read them there, not from a
branch. ⚠ The earlier note pointing at `wp/STR-BIJ-frame` (`e135fa32`) is
withdrawn: that branch's tip `f030cedd` **predates this node file, which does
not exist on it at all**, and its copy of the brief is byte-identical to
`main`'s. The branch is spent and carries nothing unlanded. Pulls a CV vote on
merge (touches `conformance/`).

Full brief: [`docs/program/wp/str-bij-overclaim-erratum.md`](../wp/str-bij-overclaim-erratum.md).

## ✅ Sequencing — DISCHARGED 2026-07-27, with the re-derivation done

`LOADER-STALE-PREMISE` is **`merged`**, so the "do not run concurrently"
condition is met and the required re-derivation has been performed. Measured at
`origin/main = 13004a63`:

| what the old text asserted | measured 2026-07-27 |
|---|---|
| attested OID of `33-declarations.md` is `8b817ffb…` | **`7daeac5b`** — the cited value was stale |
| the ledger row may already be drifted | **current**: attested OID **equals** the live blob, so the node starts from a clean row |
| landing second trips a red currency gate in CI | **no such CI gate exists** — `gen-doc-status` appears in **zero** `.github/workflows/` files |

**Re-derived consumer population — 10 `library/` pages cite
`spec/30-surface/33-declarations.md`** (excluding `SOURCE-ATTESTATIONS` itself,
which is the ledger, not a consumer):
`agents/core/read-ken.md`, `agents/core/write-ken.md`,
`agents/evaluations/results-2026-07-24.toml`, `agents/manifest.toml`,
`learn/reading-ken/01-anatomy.md`,
`learn/reading-ken/02-types-contracts-and-proofs.md`,
`learn/reading-ken/05-packages-and-provenance.md`,
`learn/reading-ken/fragments.md`, `manifest.toml`, `quickstart.md`.

⚠ **The old text's remedy is superseded by operator policy, and this is the part
to read before planning a fold.** The ledger is generated **at version release
points, not enforced per merge** (recorded in the merged `LIB-GATE-DECOUPLE`,
and `LIB-GATE-DECOUPLE` removed the CI coupling). ⇒ Editing an attested source
**does** move its OID — including for a wording-only change, which is exactly
what this node is — but that drift **does not redden a merge**. It is owed at
the next release point. So: **do not plan a blocking ledger fold; do record the
10 rows this node will drift** so the release-point regeneration is not a
surprise.

★ **The general rule still holds, and is worth more than the gate that
prompted it:** WP contention has a second axis besides shared files — two WPs
contend if one **mutates a source the other's domain attests**. Check the
ledger, not just the diff. ⚠ But state the axis separately from the *detector*:
here the axis was real and the predicted symptom (a red per-merge gate) was
decoupled out from under it. **A contention argument that names only its symptom
expires when the symptom is retired, even though the contention is unchanged.**
