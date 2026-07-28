---
scope: fleet
audience: (see scope README) — anyone framing, reviewing, or scoping a
  preservation/regression oracle pinned to a frozen frame-time golden
source: kenfmt capstone C follow-on, 2026-07-13/14
---

# A frame-pinned preservation oracle is a discharged one-shot proof

A preservation oracle pinned to a point-in-time frame (proving operation X
preserved meaning) is a **one-shot obligation, discharged when X merges**
— at the first authorized evolution of the artifact, retire it, don't
re-baseline it.

kenfmt capstone C shipped an AC2 oracle,
`actual_frozen_reformat_matches_frame_semantics_and_literate_bytes`, that
hashed each corpus file's AST/elaboration/literate-prose and compared it to
a **frozen golden taken from the pre-reformat frame**. It was deliberately
non-circular (golden from the frame, not from the formatter's own output)
— a genuinely good design *for what it was: a one-shot proof that C's
whole-catalog reformat preserved meaning*, gating the C merge. (Currency
note, 2026-07-28: this oracle no longer exists in the tree — consistent
with the disposition below having been carried out.)

The first post-C WP that *legitimately* changed the corpus — a reviewed ABI
migration adding an entrypoint wrapper to 19 files — made AC2 fail on those
files (AST fingerprint grew: real added AST, a true positive). AC2 was
working as designed; its **premise** ("the corpus's only mutation is the
meaning-preserving reformat") was simply violated by authorized evolution.
A frame-pinned golden cannot distinguish authorized evolution from
unauthorized drift — that is the review process's job, not a snapshot's.

**The ruling: retire the frame oracle wholesale, don't re-baseline it.**
- A frame-anchored preservation oracle is a **discharged one-shot proof
  obligation** the moment the operation it guards (the reformat) merges.
  Keeping it live as a permanent regression gate conflates "operation X
  preserved meaning" (a fact about a past event) with "the artifact must
  forever match its pre-X state" (which forbids all future legitimate
  change).
- **Re-baselining the touched goldens per WP is an exception-ledger
  treadmill**: every future artifact-touching WP owes a manual golden
  refresh, which reintroduces exactly the rubber-stamp-the-refresh risk — a
  gate everyone routinely refreshes catches nothing. It also usually
  **can't stay non-circular**: the intermediate "post-migration-but-pre-
  operation" input is ephemeral (not a committed baseline), so the
  refreshed golden degrades to being derived from the operation's own
  output. Subsume-don't-proliferate says retire, not ledger.
- **Verify no durable coverage is lost first.** The go-forward invariants
  are *current-anchored* and typically already present and *stronger*.
  Here a byte fixed-point test (`format(file)==file` over the live corpus)
  **strictly dominates** the frame AST/elab oracle as a formatter-
  regression guard: any meaning change alters bytes → fails the fixed
  point, and a byte fixed point trivially preserves AST *and* elaboration.
  The retired oracle's only distinctive value was the one-time
  non-circular proof, already consumed.
- Retiring a merged, soundness-labelled test is a real scope call — rule
  the *shape* (retire), let the scope-owner authorize the *size* of the
  delete, keep it test-only (no production/artifact byte moves), and
  re-confirm the durable gates (fixed-point, strict CLI gate) stay present
  and non-vacuous.

**Reversal discipline noted at the time:** the first ruling was to
*normalize* the oracle (fixing only the first, false-positive symptom). The
second symptom proved the test's **shape**, not just its ID-sensitivity,
was wrong for a living corpus — a legitimate flip on new evidence, made
transparently.

**How to apply:** when a preservation oracle is proposed or reviewed, ask
*"is this oracle proving a fact about a past operation, or a permanent
invariant of the artifact?"* If the former, plan its retirement into the
same WP that discharges it — name the go-forward invariant that survives it
before deleting it, and confirm that invariant strictly dominates.
