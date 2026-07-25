---
id: ABI-R1
title: "correct stale filesystem capability prose — scoped roots, rights, symlink policy and no-follow resolution have landed"
status: ready
owner: foundation
size: S
gate: none
depends_on: [DOC-W2]
blocks: []
github: null
origin: docs/program/10-linux-abi-completion.md §4 (the ABI-completion program); node filed by the Steward 2026-07-25 on the operator's directive to frame the remaining program. Agents cannot create tracked work (COORDINATION §2).
---

> ## ✅ FRAMED AND READY — 2026-07-25
>
> **The shovel-ready frame is
> `docs/program/wp/ABI-R1-capability-prose-currency.md`.** Read that, not this
> file. It pins the landed surface from `crates/ken-host/src/capability.rs`
> clause by clause, and it carries the finding that makes this WP non-trivial:
>
> ★ **`check_fs_capability` enforces RIGHTS and AUTHORITY only.** It returns
> `ScopeEscape` solely for an *empty* scope and **never returns `SymlinkDenied`
> at all** — it hands the scope back to its caller. So confinement and symlink
> policy are **carried** by the capability and **enforced at resolution**, not at
> the gate. ⇒ "path-confined" is a claim about the resolver; citing the gate as
> evidence for it is an overclaim.
>
> ★★ **And the `AFull` sentence conflates two axes.**
> `rights_for_authority(AUTH_FULL) == RightSet::ALL`, so `Full` **still holds
> `WRITE` and `DELETE`** — the rights axis is unchanged. Only the *reach* axis
> changed. The correction must change **"anywhere"** and leave **"writes and
> deletes"** alone. Getting this backwards is the likeliest way to ship a new
> false statement.

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

## ✅ THE LEDGER BLOCKER IS DISCHARGED — 2026-07-25

`DOC-W2` merged (`origin/main` = `d3b9f36c`) and the attestation row has been
**re-derived**, which was the whole point of sequencing rather than trusting a
pre-merge read:

```
library/SOURCE-ATTESTATIONS  row 9   (was row 7 — DOC-W2 added three rows)
  59fbe76dde61a9ab3a1d4599088c60f04502ea89  catalog/packages/Capability/Filesystem/Errors.ken.md
```

⚠ **Re-derive again at pickup.** The row number is a function of everything
merged so far; treat the line above as evidence the *check* was run, not as a
durable fact.

## ⚠ SUPERSEDED: my two earlier "startable" claims, both wrong, in opposite ways

Recorded rather than deleted, because the pair is the lesson:

1. **"`ABI-R1` and `ABI-S3` are startable now."** True on the **DAG** axis, false
   on the **LEDGER** axis — `SOURCE-ATTESTATIONS` attested this WP's only target
   while `DOC-W2` was rewriting that ledger. Disjoint file lists, real collision.
2. **"There is currently NO releasable ABI work."** Correct when written, **stale
   the moment `DOC-W2` landed.**

★ **DAG-freeness is necessary, not sufficient — and a blocker note goes stale as
soon as its blocker clears.** Both errors came from writing a *conclusion*
("startable") where the durable thing was the *check* ("does anything in flight
attest my target?").

⇒ **Current status: `ABI-R1` is the one genuinely releasable ABI node.**
`ABI-S3` remains Runtime-owned and held behind the `RT-NATIVE-FNSPLIT` priority.

## ⛔ OPERATOR RULING 2026-07-25 — HELD. Fleet stays FNSPLIT-only. SETTLED.

Release requires a slot, and the operator has ruled there is none: **the fleet
remains strictly single-threaded on `RT-NATIVE-FNSPLIT`.** I raised this WP and
`DOC-GATE-NEEDLE` together, with idle rings and verified-disjoint file sets, and
the ruling was **hold both**.

★ **The durable part — the doc-track exception is DOC-ONLY.** Its stated basis
(contention-free-ness) explains *why doc received the exception*; it is **not** a
general rule that any contention-free WP may run concurrently. ⇒ **Demonstrating
disjoint file sets does not earn a slot.** This resolves an ambiguity I had been
reading the other way.

⛔ **Do not re-ask.** A settled operator ruling is a fixed input. This node stays
`ready`, fully framed, until the FNSPLIT chain closes.

⚠ **Documentation-only and `S`, but it is not busywork:** prose that contradicts
landed behavior is actively misleading, and this is the class of defect the
`DOC-W0` family and the withdrawn `ABI-R2` both came from — *a true statement
standing in for the property that mattered*. The judgment content is deciding
what the capability **now** guarantees, not find-and-replace.
