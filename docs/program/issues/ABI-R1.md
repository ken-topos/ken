---
id: ABI-R1
title: "correct stale filesystem capability prose — scoped roots, rights, symlink policy and no-follow resolution have landed"
status: draft
owner: foundation
size: S
gate: none
depends_on: [DOC-W2]
blocks: []
github: null
origin: docs/program/10-linux-abi-completion.md §4 (the ABI-completion program); node filed by the Steward 2026-07-25 on the operator's directive to frame the remaining program. Agents cannot create tracked work (COORDINATION §2).
---

> ## Authority: `10-linux-abi-completion.md` §4 — read that, not this
>
> ⛔ **This is a tracker/DAG node, NOT a shovel-ready WP frame.** A
> `docs/program/wp/` frame carrying deliverables, acceptance criteria, fixed
> inputs, negative controls, and a contention check **must be authored before
> release** (§2c front-load rule: the T1 enclave does the design judgment so the
> build ring executes mechanically). **Do not release this on the strength of
> this file.**

## Objective

`Capability/Filesystem/Errors.ken.md` still says filesystem authority is coarse
and not path-confined. **That is now false**: scoped roots, rights, symlink
policy, and no-follow resolution have landed.

## ⛔ SEQUENCED AFTER `DOC-W2` — a LEDGER-AXIS collision, not a shared-file one

**Caught at the contention check, 2026-07-25, before release.** This WP and
`DOC-W2` have **disjoint file lists** — and they still contend:

```
library/SOURCE-ATTESTATIONS:7
  59fbe76dde61a9ab3a1d4599088c60f04502ea89  catalog/packages/Capability/Filesystem/Errors.ken.md
```

★ **The ledger attests this WP's only target.** Editing `Errors.ken.md` **moves
its blob OID** — that is true even for a locator-only change, let alone a prose
correction — so ABI-R1 **must also update the ledger row**. `DOC-W2` is mid-flight
rewriting `library/SOURCE-ATTESTATIONS` (its Librarian fold added pack `sources`
and closed source currency). Two WPs writing different rows of a one-row-per-source
ledger **merge clean and wrong**: git sees disjoint hunks and unions them.

⇒ Recorded as a **`depends_on`, not a prose note, so the generator derives the
block.** ⚠ **After `DOC-W2` lands, RE-DERIVE the attestation row and the consumer
population before releasing this** — the ledger will have changed, which is the
whole point.

## ⚠ CORRECTION to "startable now"

I earlier reported this as one of two ABI nodes startable before PX8 closes.
**That was true on the DAG axis and false on the ledger axis.** `ABI-S3` is the
other, and it is owned by Runtime, which is held on the `RT-NATIVE-FNSPLIT`
priority. ⇒ **There is currently NO releasable ABI work.** The DAG-freeness of a
node is necessary, not sufficient.

## ⭐ Why this one is special — it is startable NOW

**`ABI-R1` and `ABI-S3` are the ONLY two nodes in this program with no
dependency on `PX8`.** Everything else in §5 descends from it. With the fleet
single-threaded on `RT-NATIVE-FNSPLIT`, these two are the only available parallel
ABI work.

⚠ **Documentation-only and `S`, but it is not busywork:** prose that contradicts
landed behavior is actively misleading, and this is the class of defect the
`DOC-W0` family and the withdrawn `ABI-R2` both came from — *a true statement
standing in for the property that mattered*. The judgment content is deciding
what the capability **now** guarantees, not find-and-replace.
