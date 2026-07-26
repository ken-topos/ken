---
id: ABI-R1
title: "correct stale filesystem capability prose — scoped roots, rights, symlink policy and no-follow resolution have landed"
status: merged
owner: foundation
size: S
gate: none
depends_on: [DOC-W2]
blocks: []
github: null
origin: docs/program/10-linux-abi-completion.md §4 (the ABI-completion program); node filed by the Steward 2026-07-25 on the operator's directive to frame the remaining program. Agents cannot create tracked work (COORDINATION §2).
---

> ## ✅ MERGED — 2026-07-26 ~09:33Z, PR #1006, `origin/main` = `499ff468`
>
> Kicked ~09:0xZ, QA-approved candidate ~20 minutes later, merged on the third
> candidate. ⚠ **Retros are NOT yet in** — the WP is merged, not closed
> (COORDINATION §10). Chased from the Steward at ~09:3xZ.
>
> **Merged exact `d265172624b19afcaef636c07aee5b37207b2416`.** Decision
> `dec_7ayj9fy85kjhw`, verified `resolved` **from the object** with a non-null
> Architect `resolved_by` — not from the channel report.
>
> **Landed, verified by BLOB IDENTITY on `origin/main` after the merge, with a
> negative control:**
>
> | object | blob | check |
> |---|---|---|
> | `catalog/…/Filesystem/Errors.ken.md` @ `499ff468` | `a9977831` | ✅ the new prose |
> | the same path @ the previous main `afbd1542` | `59fbe76d` | ✅ control — the checker *can* tell them apart |
>
> ⛔ **THE CANDIDATE PREDATED THE PREVIOUS MERGE, SO ITS DIFF LOOKED LIKE A
> REVERT — IT WAS NOT.** `git diff origin/main <candidate>` listed `#1005`'s three
> `scripts/` files, because the candidate branched before `#1005` landed. ⭐ **A
> diff against a moved base cannot distinguish "reverts it" from "predates it".**
> The question was settled by computing the *merge result* rather than reading the
> diff: `git merge-tree --write-tree` gave tree **`62b1a194`** with all three
> `scripts/` blobs preserved, and the publisher's own post-merge verification then
> reported the landed tree as **`62b1a194`** — the same tree, arrived at
> independently. All three blobs re-confirmed present on `main` afterwards.
>
> **Verified at kickoff time (`7eaa42a3`), one command per object:** frame blob
> `0a28c7df`, this node `a2297870`, target `59fbe76d`,
> `crates/ken-host/src/capability.rs` `5c03ed32`, plus a deliberately-absent path
> as a control.
>
> ⭐ **The frame's `d3b9f36c` anchor was six `main`-SHAs stale and it did not
> matter:** both load-bearing files were **blob-identical** at `7eaa42a3`. ⇒
> Staleness is a question about **content**, not SHA distance.
>
> ⛔ **The target IS a cited source** — `library/SOURCE-ATTESTATIONS` **row 9**
> held that exact OID, so the ledger row moved in the same commit as the prose.
> Frame `D4`'s *"row may have drifted"* caveat was **DISCHARGED** before the kick,
> and the row was re-checked against the new blob byte-for-byte before publishing
> (with a positive control at the old base, to confirm the coherence checker
> reports coherence at all rather than passing on a lookup miss).
>
> ### ⛔ THREE CANDIDATES, AND MY OVERCLAIM CAUSED ONE OF THE BLOCKS
>
> Candidate 1 (`0c8b77fc`, preserved at `preserved/abi-r1-0c8b77fc`) was blocked
> by the Architect for asserting the resolver **enforces** the scope's
> `SymlinkPolicy`. Candidate 2 (`f93a81bd`) was blocked by QA for asserting
> resolution **does not consult** it. ⛔ **Neither universal is true, and I supplied
> the second one.** I reported to the live ring that no production consumer
> branches on the policy, from a `grep … | head -20` whose window cut
> `ken-interp/src/eval.rs:4040` off the bottom; there are six production reads and
> four branch sites. The implementer adopted my universal and wrote its inverse.
>
> ⇒ The landed prose stops at the **true** universal: `SymlinkPolicy` is a
> carried, per-scope, two-state mechanism. The **lane divergence** — native
> rejects unconditionally while the interpreter and virtual lanes honour the
> policy — is a **code** question, tracked separately and deliberately not
> described here as intended behaviour.

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

⇒ **That assessment was for the release decision and it is now SPENT: `ABI-R1`
is MERGED (`499ff468`, 2026-07-26).** ⛔ Do not read the line above as naming a
currently-available node. `ABI-S3` remains Runtime-owned and held behind the
`RT-NATIVE-FNSPLIT` priority, so **no ABI node is releasable right now** — and the
next one to assess is `ABI-S3`, whose frame does not exist yet.

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
