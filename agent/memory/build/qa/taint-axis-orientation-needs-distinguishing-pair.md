---
scope: build/qa
audience: (see scope README)
source: private memory `taint-axis-orientation-needs-distinguishing-pair`
---

# Any classification discriminator needs a non-degenerate distinguishing pair

Conformance for a **taint / order-dual lattice axis** (integrity, `@ct`
constant-time, any future taint factor where `⊥` is safe / `⊤` is the taint /
the **sink demands `⊥`**): the load-bearing latent bug is the **orientation
being backwards** (sink mis-built to demand `⊤`, confidentiality-style), which
**silently inverts accept↔reject**. A **single reject case cannot net it** —
under the flip it either still rejects (for the wrong reason, green-vs-green) or
flips to accept with no contrast to reveal it. The net is the **non-degenerate
distinguishing PAIR on the SAME sink shape**: the taint value (`⊤`) **rejects**
*while* the safe value (`⊥`) **accepts** — because a flipped order inverts
**both**, the pair (not either case alone) pins the orientation.

This is the `[Sec1-dual]` discipline. It recurred across **two axes** —
integrity (Sec1, `Untrusted ⋢ Trusted`) and `@ct` (Sec1ct: A1 `cmp` rejects on
`k:ct⊤` WHILE B1 `route` accepts on `Secret`-but-`ct⊥`, same `BranchGuard`
shape). Author it as a pair and say so in the cross-case sweep ("orientation pin
{A1,B1}"). Doubly load-bearing under N1: the labels are **erased before the
kernel**, so the orientation flip emits a kernel-valid core — the discriminating
pair is the **sole** net (see untrusted layer backstop hole for omissions).

**Generalized (B1, 3rd domain — `behavioral/export/seed-export.md`,
`5808e59`).** The same trap-class is NOT specific to lattice axes — it is **any
classification/projection discriminator** whose latent bug silently swaps which
bucket a value lands in. B1's no-over-claim invariant (I1) turns on a
discriminator that reads **kernel state** (`trusted_base()` membership +
certificate-presence) vs a **self-reported status string**: the net is the
same-postcondition PAIR (EX-A1 `proved`→`Q` WHILE EX-A2 open-hole→`P`/`unknown`,
one proposition under two kernel states). A single "proved→`Q`" case is
green-vs-green under the two realistic bugs (trust the untrusted layer's
`proved` string; or bucket everything-with-an-`ensures`-clause into `Q`). So the
rule is: **when a spec pins a classification map, the conformance net is one
non-degenerate pair per discriminator boundary, on a shared input.** Three
domains now: integrity (Sec1) · `@ct` (Sec1ct) · export `proved`↔`assumed` (B1).
Strongest form — the **source spec itself mandates the pin** (`21 §5.4`: "a
structural discriminator that flips, not a status string compared for equality")
— which reconcile-don't-cite surfaces. (B1 corollary: trusted-by-typing is NOT
kernel-`proved` → routes to `P`/`tested`, never `Q`; the safe direction is
under-claim — the Sec1ct CT-D1 erratum.)

Generalizes cast direction test at nondegenerate endpoints / transport schema
degenerate endpoint trap (direction tested at non-degenerate endpoints) from
cast/transport **direction** to lattice **axis orientation**; a specialization
of discriminating conformance verdict must flip (the flip must be a real
contrast, not green-vs-green) for the taint case. The author-side spec mirror:
pin **both** order directions in prose (`61 §5a.1`) so the orientation is
stated, not just implied.
