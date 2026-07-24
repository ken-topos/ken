---
id: STR-BIJ
title: "the String/List Char 'bijection' over-claim (adversary A1 + A2)"
status: ready
owner: spec-enclave
size: S
gate: none
depends_on: []
blocks: []
github: null
origin: adversary findings A1 + A2
---

> ⛔ **HELD behind [[DOC-VALIDATION-BINDING]] (Steward, 2026-07-24).** That WP
> acquired the `library/` ledger axis (`manifest.toml` + generated `STATUS.md`)
> on 2026-07-24 and is the live holder. **Do not release this WP concurrently** —
> same axis, and the collision shape is a SILENT union in the ledger plus a red
> currency gate after landing, not a merge conflict. Re-derive the consumer
> population after that WP lands; it will have changed. Steward ruling
> `evt_tmv40vgtg63k`.

Honesty erratum on landed prose: spec normative over-claim (A1) and a catalog
title over-claim (A2), both CONFIRMED — the adversary itself supplied the
refutation that keeps this out of soundness (see the brief §4). Wording-only,
zero semantic change.

Frame committed on `wp/STR-BIJ-frame` (`e135fa32` per the tracker's held-
branch note as of RT-PARITY's close); ready to kick to the Spec enclave
(Handoff Gate first). Pulls a CV vote on merge (touches `conformance/`).

Full brief: [`docs/program/wp/str-bij-overclaim-erratum.md`](../wp/str-bij-overclaim-erratum.md).

## ⛔ Sequencing — do NOT release concurrently with `LOADER-STALE-PREMISE`

**These two WPs share no files and still contend, through the citation
ledger.** A file-level conflict check says they are disjoint, and that check
is not the binding one here.

- This WP **edits** `spec/30-surface/33-declarations.md`, which is an
  **attested source** (`library/SOURCE-ATTESTATIONS`, OID `8b817ffb…`).
  Changing its bytes drifts the ledger row for **every `library/` page that
  cites it**.
- `LOADER-STALE-PREMISE` **adds new citations to that same file** — its
  acceptance requires each repaired claim to cite the normative spec rather
  than another doc that cites it. So it *widens* the population this WP
  disturbs, while this WP is in flight.

⇒ Run concurrently and whichever lands second meets a **red currency gate**,
needing a ledger fold and a fresh semantic re-validation of every affected
consumer. That is not a merge conflict and no `merge-tree` or touched-path
intersection will predict it — it surfaces only as a failing
`gen-doc-status --check` after the fact.

**Sequence after `LOADER-STALE-PREMISE` merges**, then re-derive the
consumer population before kicking — it will have changed, which is the whole
point.

★ **General rule this instance establishes:** WP contention has a second
axis besides shared files. Two WPs contend if one **mutates a source the
other's domain attests**. Check the ledger, not just the diff.
