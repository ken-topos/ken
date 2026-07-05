---
scope: roles/integrator
audience: integrator (and any reviewer approving a multi-artifact erratum:
  architect, spec-leader, conformance-validator)
source: merges former private memories `multi-piece-erratum-landing-integrity`
  and `multipiece-erratum-verify-all-on-main` (same finding, written up from
  the assembler/reviewer side and the approver side)
related: verify-field-order-arity-against-declaration-not-prose,
  wp-frame-stale-vs-landed-kernel
---

# A multi-piece erratum isn't landed until every piece is verified on `main`

A soundness erratum is often **three coordinated pieces** — spec chapter +
conformance corpus + kernel code — meant to land together. The federation can
ship only some of them: a squash-merge of a build branch that cherry-picked the
spec/conformance pieces can **silently drop them**, landing only the piece that
was native to the branch. Gate "done" on verifying **every piece is present on
`main`**, never on the merge notification.

**The concrete failure (§5.1 quotient-respect erratum, 2026-06-30).** Three
pieces — spec `da344a6`, conformance `f3ece75`, kernel `bb0b3ba` — were
Architect-approved and "shipped" as `ecbb279`; every retro/status treated it as
done. But on `main` only the **kernel** piece had landed. The spec still had the
reversed schema (normatively contradicting its own kernel), and the conformance
still had the reversed constant-motive cases with the direction-witnessing
dependent-motive case **deleted** — so the corpus guarded nothing, the exact
masking the erratum was meant to remove. `git branch --contains f3ece75` was
empty; the cherry-picks had dropped out of the squash. Not a soundness bug (the
kernel was right) — but the spec misdescribed its own kernel, and a build coding
from the spec would re-introduce the reversed/incomplete direction uncaught.

**How it was caught:** by grounding a new WP's base against the **landed corpus
files** (`git diff <prior-approved-commit> origin/main -- <file>`), not against
"verified/merged" notifications. The gap was invisible in every status line,
retro, and merge announcement — it surfaced only by diffing an already-approved
commit against `main` and finding `main` was its inverse. Notifications report
intent; only the landed bytes report truth.

**The standing gate this produced:** an Architect-approved N-piece erratum is
not landed until **every piece it was approved as** is verified present on
`main` — grep the normative sites, context-aware. **Grep caveat:** an erratum
often *intentionally* keeps the buggy form once, as a labeled negative example
(a "must be rejected" reject-witness) — so a naive "buggy form absent entirely"
grep **false-flags** a correct landing. State the gate precisely: *corrected
form present at the normative rule/schema sites; buggy form present only as the
labeled witness, never at a normative position.*

**Two structural checks, both required (the two axes of one gate) — run against
*current* `origin/main` immediately before merge, not just at assembly time:**
1. **Content drift** — `git diff <author-tip>:file <assembled-tip>:file` for
   every file must be empty against the author's actual branch tip, not just the
   commit you happened to read the content from.
2. **Diff scope** — `git diff --stat origin/main <tip>` must show **only** the
   intended files. If `main` advanced since you based your assembly, a
   stale-based diff silently grows to include **reverts of unrelated,
   already-landed WPs** — a correct-content tip on a stale base is as
   unmergeable as a wrong-content tip on a correct base. Re-base
   (`git rebase --onto <current-main> <old-base> <tip>`) and re-run both checks
   if `main` moved.

**The assembly-rebase wrinkle:** when assembling a single branch from an
author's spec-stage commit plus your own conformance/kernel commits, take the
author's **branch tip**, not just the commit you read the content from — a
follow-up commit on top of the body you read (a stale cross-ref fix, a
correction) is exactly the piece a content-only rebase drops. Enumerate every
commit since the merge-base (`git log <merge-base>..<author-branch>`) and fold
all of them; an empty drift-diff against the author's actual tip is the cheap
structural proof the fold is complete.

**The author-side race:** folding a correction into an **in-flight** merge
(votes already cast, Integrator poised) and declaring it "supersedes" the prior
tip interlocks nothing — decision-time state ("hasn't merged yet") is
perishable, and the author doesn't control merge timing. A squash can land the
pre-fold tip a beat before the fold commits, shipping the exact defect the fold
was meant to fix. Rule: a fold/erratum that races an in-flight merge must either
explicitly **hold the merge** (mention Integrator + leader *before* committing
the fold) or be authored as an erratum-on-current-`main` from the start — never
"fold and declare supersede" against a WP whose votes are already cast.

**Why it matters:** a partially-landed erratum is worse than an unlanded one —
the green-looking corpus and the authoritative-looking spec certify the bug as
the contract for the pieces that didn't land, while the one piece that did land
makes everyone believe it's done.

**How to apply.** (1) When cutting a new WP off `main`, diff the landed corpus
against your own recent approved commits in that area before authoring — confirm
prior work is actually present, not reverted by a parallel branch or dropped
from a squash. (2) For any multi-piece erratum you approve or assemble, track
**all** pieces to `main` — don't assume the others landed because one did, or
because a merge was announced. (3) Re-land a dropped piece verbatim
(`git cherry-pick` the original approved commit) so it needs no re-review. (4)
Keep trust-root errata on their own branch, never bundled into a lower-risk WP.
