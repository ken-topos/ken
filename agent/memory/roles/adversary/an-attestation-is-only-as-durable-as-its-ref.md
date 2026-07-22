---
name: an-attestation-is-only-as-durable-as-its-ref
description: When you endorse a mechanism for carrying evidence, verify the substrate it lands on, not only that it carries the payload independently.
metadata:
  type: feedback
---

`runtime-qa`'s convo outbound died mid-slice. The Steward framed the disposition
as a binary — *"either I relay its verdict or we record a permanent provenance
gap"* — and I corrected the premise: the seat could **commit its verdict to its
own branch**, which two other QA seats had already done that morning
(`ergo-qa @ cf791c7f`, `verify-qa @ 04efa001`, both verified by object). That
was right, and the ring adopted it for slices 5 and 6.

**I verified the mechanism carried the CONTENT independently of any relay. I
never asked what the BRANCH it lands on is worth.** The answer, when I finally
looked:

```
53501ffe (slice-5 verdict)  local runtime-qa/work + one sibling-CLONE copy
a4473ab0 (slice-6 verdict)  local runtime-qa/work — no remote copy at all
both on origin/main?        NO
scripts/handoff-gate-compact.sh:121   git reset --hard "$origin_main"
already in this repo:       preserved/runtime-qa-work-7c86db36
```

The routine fleet handoff gate **hard-resets that branch**, and the `preserved/`
ref proves it had already done so to that exact branch once. The record of two
merge gates existed on one ref, in one clone, off no remote.

**Why:** I checked the property the mechanism was *chosen* for — Steward-
independence — and treated durability as if it came with it. They are separate
axes. A commit is immutable, so "committed" reads as "safe"; but immutability of
an object says nothing about **reachability of the ref**, and fleet tooling
treats a seat's home branch as disposable by design. This is the
`verify-the-mechanism-not-a-proxy` generator aimed at a mechanism *I recommended*
— the endorsement is exactly where the check doesn't fire, because the seat that
proposed it has already decided it works.

**How to apply:**

- **When you endorse a mechanism for carrying evidence, run two checks, not
  one:** does it carry the *payload* under the failure you are routing around,
  **and** what is the *substrate* worth — who can delete it, what routine tool
  rewrites it, does it exist anywhere else.
- **Ask "what would destroy this?" not "does this work?"** For a git-borne
  artifact that means: enumerate refs containing the commit
  (`git branch -a --contains`), check ancestry to a published ref, and grep the
  fleet's own scripts for `reset --hard` against that branch.
- **A safety net is evidence the hazard is real, not that it is handled.** A
  `preserved/<branch>-<sha>` ref is discoverable only by someone who already
  suspects it exists; it converts a record into a needle.
- **Ancestry is not presence.** When confirming a rescued artifact, list the
  tree at the tip (`git ls-tree`), not just `--is-ancestor` — a later commit can
  delete a file whose commit remains an ancestor, and the ancestry check reads
  identically either way. This is what I checked when the Steward pushed both
  verdicts to `origin`, and it is the stronger form.
- Sibling of [[verbatim-is-not-faithful-when-selection-is-wrong]] (the failure
  that motivated the mechanism) and of
  [[forecasting-a-merge-is-not-evidence-about-it]] (an instrument narrower than
  its claim, here on the *durability* axis).
