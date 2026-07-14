---
scope: fleet
audience: (see scope README) — every implementer and QA; anyone whose candidate
  is green but whose base/corpus gate is red
source: CC8 implementer + QA §10 retros (evt_4ztyyac93z3fv, evt_6a6t29gntm05b),
  2026-07-14 — and the red was the Steward's
---

# A red gate on files you don't own is NOT your bug — hold your green candidate and route the red

**CC8 rebased onto a fresh `main` and found:**

```
its own gates          → GREEN  (cc8 6/6, cc7 6/6, targeted fmt/diff-check)
the corpus-wide gate   → RED    (two catalog/guide/ strands non-canonical)
```

**The red had nothing to do with CC8.** An upstream WP (LET-2) had landed two
`catalog/guide/*.ken.md` files **unformatted**, and the frozen-corpus formatter
gate enumerates that directory. **CC8's implementer had not touched either
file.**

## ★ The trap

**The corpus gate is a MANDATED gate for the WP.** So the implementer is staring
at a red required check, with a green candidate, and the obvious move — **"make
the gate pass"** — means **editing another team's files**.

> **That move produces a diff that is scope-creep on its face, silently absorbs
> someone else's defect into your WP, and HIDES the real bug** — the upstream
> owner never learns their merge went out broken.

**The implementer named it exactly:** *"Treating either as a CC8 code problem
would have produced **dishonest compensating edits**."*

## The rule

**When a gate fires on files outside your WP's authorized scope:**

1. **STOP. Do not repair it.** Your scope is your scope; a green diff bought by
   editing an upstream owner's files is not a green diff.
2. **HOLD the candidate.** It is green on its own gates — that fact is evidence,
   and it is the thing that proves the red is not yours.
3. **SEPARATE the two verdicts explicitly in the handoff:** *"candidate-local
   gates GREEN; base-wide corpus gate RED, from `<file>`, owned by `<WP>`, which
   I did not touch."* **Both numbers, both attributions.**
4. **ROUTE the red to its owner** (the Steward, or the owning ring). It is their
   defect and their fix.

**⇒ Validate CANDIDATE-LOCAL gates separately from BASE-WIDE gates, and report
them separately.** Collapsing them into one "is the build green?" question is
what creates the pressure to fix what isn't yours.

## And the sibling, for whoever LANDED the red

**The Steward caused this one** by merging LET-2 with `--doc-only` — a flag that
**merges immediately WITHOUT WAITING FOR CI**. LET-2 edited
`catalog/guide/*.ken.md`.

> **A `.md` extension under `catalog/` is LITERATE PACKAGING, not a statement
> about what the file IS.** Those are **Ken sources** that a Rust corpus test
> parses, formats, and elaborates. **`catalog/` · `examples/` · `conformance/`
> ⇒ FULL CI, always, whatever the file is named.**

**And a red corpus gate on `main` blocks the publisher for EVERY subsequent
non-doc merge** — so a fast-path merge that skips CI can stall the entire fleet
behind it. See
[[an-oracle-that-greps-a-name-fires-on-prose-that-denies-it]] for the
corpus-oracle enumeration audit that is supposed to catch this at frame time.
