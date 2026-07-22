---
id: DOC-CURRENCY-ANCHOR
title: "library/REVISION certifies nothing about the corpus — currency is unchecked"
status: closed
owner: doc
size: S
gate: none
depends_on: [DOC-W0]
blocks: []
github: null
origin: adversary finding evt_6c9mhr3tg9pfg (2026-07-22), post-merge on DOC-W0 @ 6be9754b
---

**⛔ This BLOCKS Wave 1.** The exposure is entirely forward, and Wave 1 is
where it bites.

> **Why `blocks:` is empty in the frontmatter.** Wave 1 is *deliberately not
> framed* (`12-documentation-program.md` §4: *"Each wave is framed as its own
> issue when its predecessor's exit condition is met — I am not pre-committing
> the fleet to seven waves sight-unseen"*), so there is no `DOC-W1` id to
> point at, and minting a stub to satisfy a schema field would be inventing
> scope. **The constraint is instead bound where it will actually be read:
> the Wave 1 row of that document's waves table.** When Wave 1 is framed, its
> issue takes `depends_on: [DOC-CURRENCY-ANCHOR]` and this note comes out.

## The defect

`docs/program/12-documentation-program.md:121` is normative:

> **A date is not evidence of currency.** Currency is a **source revision**,
> recorded by generated `STATUS.md`.

`revision_resolved()` (landed in DOC-W0, `scripts/gen-doc-status.sh`)
establishes exactly one thing: **`REVISION` names a real commit in this
repository's history.** It never reads `library/`, never reads a manifest
`sources` entry, and never compares anything at `REVISION` to anything at
`HEAD`.

**A revision that certifies nothing about the corpus is a date with extra
steps** — the precise failure that clause exists to forbid.

## Grounded on `origin/main @ 6be9754b`, no mutation required

```
library/STATUS.md:7  **Validated revision:** `e5a400c7...`
$ git ls-tree --name-only e5a400c7 -- library/ | wc -l
0
$ ./scripts/gen-doc-status.sh --check
gen-doc-status --check: library/STATUS.md is current.   # exit 0
```

The corpus is stamped "validated at" a revision where `library/` has **zero
entries**. `STATUS.md`'s own declared `sources` (`library/manifest.toml`,
`library/REVISION`) **do not exist at the revision it cites**. Gate green.

**Forward repro (adversary, reverted):** injecting
`"THE TCB IS NOW UNBOUNDED AND UNAUDITED"` under a live `sources` anchor of
`library/introduction.md` — `docs/PRINCIPLES.md#5-keep-the-tcb-small…` —
leaving every heading intact, yields `--check: current`, exit 0, validated
revision unchanged.

**Structural, not merely empirical.** The only four files referencing
`library/REVISION` at `6be9754b` are the script, `STATUS.md`,
`manifest.toml`, and the gate test — no CI step, no hook. Gate 3 checks source
path+anchor **existence in the current tree**, so it catches *structural*
drift (a renamed heading goes red) and is blind to *content* drift. **No code
path anywhere reads a source's bytes at `REVISION`.**

## Why this is a mechanism defect, not a discipline lapse

The obligation to bump `REVISION` on validation was not missed — it was
**correctly reasoned away, twice, from the predicate**:

- doc-author: *"no bump needed since it's still further back than the new
  merge-base, and the gate only requires ancestry, not exact-merge-base
  equality."*
- librarian QA: *"resolves and remains an ancestor, as contracted; it need not
  equal the merge-base."*

**Both are right about the predicate.** Two careful independent T1-grade seats
read the gate, correctly concluded nothing needed to change, and shipped a
stamp asserting validation against a revision where the corpus does not exist.
**When the predicate tells both the author and QA that the stale case is fine,
the predicate is the defect.**

## Honest bounding — severity is GAP, not correctness-now

All five cited sources are **byte-unchanged** across the 3-commit / 13-file
window between `REVISION` and the merge (`12-documentation-program.md`,
`PRINCIPLES.md`, `spec/00-overview.md`, `spec/60-security/64-trust-model.md`,
`spec/20-verification/21-spec-syntax.md`). **Today's stamp is substantively
true — but by the ring's diligence, not by anything the mechanism
established.** The evidence is absent, not yet wrong.

**Bootstrap caveat, in fairness:** for the *introducing* commit the corpus
genuinely cannot exist at its own parent, so `library/`-absent-at-`REVISION`
is unavoidable exactly once. It resolves on the first post-merge bump — but
that bump is the same unenforced human step the two quotes above show being
skipped.

**Why Wave 1 is the trigger:** Wave 0 registers three documents whose sources
are stable charter files. Wave 1+ adds **derived-reference pages over live
spec chapters**, and nothing forces a `REVISION` bump when one moves. The first
time a cited chapter's body changes under a stable heading, every derived page
claims a currency it does not have, with every gate green — and the ledger's
`derived-reference` framing is what makes that claim load-bearing.

## The checkable property already has its data

`sources` names real repo paths, so the property is expressible per entry:

```sh
git diff --quiet $REVISION HEAD -- <source>
```

**Not prescribing the mechanism** — whether this becomes a hard gate, a
warning, or a standing Librarian as-built duty is the doc ring's design call.

## ✅ CLOSED — 2026-07-22 (Steward), acceptance re-derived on `origin/main`

**Wave 1 is UNBLOCKED.** The `⛔ This BLOCKS Wave 1` banner above is
**discharged**, not withdrawn — it was correct when written.

Closure was verified by reading the landed `scripts/gen-doc-status.sh` **on
`origin/main` through the git object store**, not from a working tree and not
from the Steward's own open-item list:

| AC | how it is met on `origin/main` |
|---|---|
| 1 — property, not mechanism | the content-currency gate compares **each cited source's bytes at `REVISION` against `HEAD`** and fails on drift, so the currency claim is backed rather than asserted |
| 2 — bootstrap explicit | the introducing-commit case is handled and **messaged distinctly** from a stale one (*"nothing was there to validate… this is distinct from a cited source"*) |
| 3 — two-sided | body-drift-under-unchanged-heading is **detected**; an unchanged corpus stays green. Both arms mutation-proofed against the real tree |
| 4 — grep the emission | the check reads source bytes **at `REVISION`** via the object store — the exact operation that was absent |
| 5 — CI | green in CI on the merge that landed it; **no local `--workspace` run** |

**Three folds were needed, and each fixed a genuinely different defect** —
worth keeping, because the first two both looked like the whole fix:

1. `REVISION` named a **pre-squash branch commit**, unreachable once `main`
   squash-merged the branch. Now checked **on the branch, against the
   publication topology.**
2. The regression drove a **synthetic fixture** and never consumed the
   repository's **real** `library/REVISION` — so a branch-local bad value
   passed every existing check.
3. **The check silently skipped itself** when `origin/main` was unresolvable
   (shallow CI checkout). A gate that no-ops on an environment condition
   reports success for *"I did not run"* — the failure mode the adversary's
   sharpening below predicts exactly.

⛔ **Fold 3 is the transferable one:** the first two folds were found by review,
the third only by **running the probe that had been offered as skippable.** An
escape clause is a fallback, never an equal option.

## Acceptance criteria

1. **State the property, not a mechanism.** `STATUS.md`'s currency claim is
   backed by evidence that **every cited source is unchanged between
   `REVISION` and `HEAD`**, or the claim is visibly weakened to what is
   actually established. A green gate must not assert more than it checked.
2. **The bootstrap case is handled explicitly**, not by accident — state what
   `REVISION` means for the introducing commit and make that case
   distinguishable from a stale one.
3. **Two-sided, in the consumer topology** (DOC-W0's own hard-won carry):
   a cited source whose *body* changes under an unchanged heading must be
   **detected**; an unchanged corpus must stay green. Mutation-proof both arms
   against the real tree.
4. **Grep the emission, not the name** — demonstrate the check reads a
   source's bytes *at `REVISION`*, since that is the exact operation absent
   today.
5. Green in **CI**, never a local `--workspace` run (`COORDINATION §12`).

## Notes — the record on DOC-W0 should not overclaim

DOC-W0's AC-1 required *"a new page cannot land without declaring what it is,
what grounds it, and **how its currency is checked**."* That was satisfied
**structurally** (a revision is recorded and validated as a real ancestor) and
**not substantively** (the recorded revision certifies nothing about the
corpus). DOC-W0 is closed on its deliverables; this issue carries the unmet
half, and the closure record says so rather than pretending the wave's whole
purpose was met.

This is the **eighth** finding in the DOC-W0 family and the same class as the
other seven — with one sharpening from the adversary worth preserving:

> Object-present was true. Ancestry-provable is true. `REVISION`-is-an-ancestor
> is true. **A false proxy gets caught in review; a _true_ one is what ships.**
> Nine rounds of careful review kept converging on better and better *true
> statements about the anchor* without anyone asking what the anchor was *for*.
