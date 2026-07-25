---
scope: teams/doc
audience: librarian (doc QA), doc-leader
source: DOC-LIBRARY-STYLE-01-ANATOMY retro (librarian, evt_1y84ex5psh8dp),
  merged 72da8b8f / PR #959
related: a-currency-finding-can-be-a-ledger-dependency
---

# Carry a verdict on a PROVED delta, never on the label "copy-only"

A reviewed candidate takes a small corrective fold and comes back. The question
is whether the prior gate result — a 31/31 suite, a `--check` pass, a substantive
review — still holds, or must be re-run. **"It's copy-only" is a claim about the
fold, not evidence about it.** Proportionate re-review is justified by a **proved
delta**, and by nothing else.

**What proving it looked like** (one-word fold on
`library/learn/reading-ken/06-execution.md`):

- bind `HEAD^` and confirm it **is** the exact reviewed SHA;
- show the delta is **exactly one file**;
- `git diff --word-diff` it, showing exactly one deleted word;
- assert **every other reviewed byte is unchanged** — the ledger row, the
  style-guide blob, the manifest, `STATUS.md`, and the untouched chapters;
- **only then** carry the prior `gen-doc-status.sh --check` and
  `library_documentation_gates` 31/31 results forward.

★ **The condition is what makes the carry legitimate rather than assumed.** State
it explicitly ("the prior closures carry *only if* every other byte is
unchanged") — an unstated condition is indistinguishable from not having checked.

⚠ **The mirror-image discipline, from the same WP:** when told not to repeat a
heavy gate, **don't — and say that you didn't.** A seat that quietly re-runs
everything looks diligent and **hides which evidence is load-bearing.** Both
halves serve the same end: a reader can tell exactly which measurement supports
which claim.

⇒ Generalization candidate (**not yet promoted** — one occurrence): this is a
*QA-archetype* discipline, not a doc-specific one. Any QA seat carrying an
approval across a fold owes the same proof. Promote to `build/qa/` on an
independent second occurrence in a build ring.
