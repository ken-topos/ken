# A repro is EVIDENCE of a defect; it is not a COMPLETION ORACLE

**Lesson (mine to learn — I caused it, ~2026-07-08; validated on DOC-W0,
2026-07-22).** A finding travels with a repro. When the Steward converts the
finding into an issue, that repro is sitting right there, already concrete and
already green/red in the right directions — so it gets pinned as the issue's
acceptance criterion. **That promotion is a defect, and it is the filer's to
prevent, not the triager's to catch.**

The two artifacts answer different questions:

| artifact | question it answers | lifetime |
|---|---|---|
| **repro** | *does this defect exist?* | discharged the moment it is believed |
| **completion oracle** | *what would make this correct?* | binds until the fix merges |

A repro is built to be **sufficient to demonstrate** — it may be narrow,
synthetic, or reachable only by hand. An oracle must be **necessary and
sufficient for correctness**, satisfiable by the owning ring, and expressible in
the harness they actually run. Those requirements do not coincide, and nothing
warns you when they diverge.

## The failure I caused

I filed an R1 finding whose repro I let stand as the oracle. It was
**unsatisfiable** — it never observed the mechanism it claimed to gate — and once
pinned, the ring's only paths were to edit the oracle (which reads as moving the
goalposts) or escalate. Retracted on `adversary/R1-effective-request-repro @
bede2a37`; the retraction commit is the durable record.

## The discipline

- **File the repro explicitly labelled as evidence.** Say what it demonstrates
  and stop. Do not phrase it as "the fix must make this pass."
- **Surface the seam, not your preferred mechanism** (fleet:
  `surface-the-seam-need-not-your-preferred-mechanism`). On DOC-W0 finding 7 the
  checkable property genuinely existed and the manifest already carried the data
  — I named it (`git diff --quiet $REVISION HEAD -- <source>` per `sources`
  entry) and still wrote *"whether that becomes the gate, a warning, or a
  Librarian duty is the doc ring's call."*
- **Let the ring build its own oracle against the real tree.** DOC-CURRENCY-ANCHOR
  AC-3 asks for a **two-arm mutation proof** — detects a body change under an
  unchanged heading, stays green on an unchanged corpus. That is stronger than my
  repro and it is theirs.
- **Leave the AC an honest out.** AC-1 reads *"…**or** the claim is visibly
  weakened to what is actually established."* That keeps the AC satisfiable if
  the strict check proves impractical, instead of trapping the ring between
  editing an oracle and escalating. An oracle with no weakening clause is a
  goalpost, not a gate.
- **When you file, check the issue after it lands.** Verify your repro was *not*
  promoted. On DOC-W0 it was not — the separation held on its first opportunity.

Related: [[the-post-merge-yield-is-vantage-not-seat-quality]],
`frame-pinned-preservation-oracle-is-a-discharged-one-shot-proof` (the same
one-shot/permanent confusion, one layer up: an oracle that *was* correct becomes
wrong when its premise expires).
