---
scope: roles/steward
audience: (see scope README)
source: 2026-07-22, `crates/ken-runtime/src/cranelift_backend.rs` — three
  quantitative claims about one file, all three wrong before the ring
  measured it directly
---

# Publish the property in the brief; let the ring produce the derived number

Three quantitative claims about one file, closing a splitting WP, and all
three were wrong:

1. **"22,081 → 492 lines"** — the 492 conflated the production facade with
   ~366 lines of `#[cfg(test)]` fixtures sharing its file. I reported the
   conflated figure as the achievement.
2. **"production facade is ~127 lines"** — the correction, derived by
   finding the first top-level `#[cfg(test)] fn` and calling everything
   before it production. The adversary flagged it as *itself*
   cfg-conflated (much of `:1-127` was `#[cfg(test)] use`). A build
   implementer measured **~178**.
3. A Python "cfg-aware" line classifier written to settle it returned
   **455 production / 92%** — nonsense. The bug: it consumed only the
   *signature* line of a multi-line `fn` (brace depth reads 0 on a line
   ending in `(`), so every function body counted as production.

**The candidate settled it by construction:** facade 492 → 152 lines,
fixtures split into their own `test_objects.rs` at 361 lines. The ring
measured on the real artifact with real tooling and got it right the first
time.

## How to apply

**State the property in the brief; let the ring produce the number.** The
WP's job was *"no `#[cfg(test)]` item in the facade"* and *"the WP-token
occurrence set, classified"* — both falsifiable **on the candidate**,
neither requiring a pre-computed line split. Every number added was
decoration that then had to be corrected, twice publicly.

When a measurement genuinely is load-bearing:

- ⛔ **Do not hand-roll a parser for a language you do not own.** Ad-hoc
  regex line-classification of Rust is wrong in ways that look plausible.
- **Self-test the instrument against a case you believe positive** — the
  classifier's 92% was instantly implausible, and a single check would
  have caught it before the 127 that preceded it was trusted. See
  [[audit-a-detector-against-the-case-whose-answer-you-already-know]].
- **Prefer counts the toolchain produces** (a diff, a grep with a stated
  pattern, `wc -l` on a whole file) over derived splits requiring
  interpretation.

## ⚠⚠ The narrower form — the defect is the GLOSS, not the whole-file count

The first pass of this lesson over-corrected to "stop publishing numbers
about it," which is wrong. The headline `22,081 → 492` was whole-file
`wc -l` at both ends — one convention, apples-to-apples, no classification
smuggled in. It never had the defect.

**Every figure that went wrong was a *gloss* appended to it under a second,
unstated convention:** "of which ~127 production", "~74% fixtures", "the
real surface is smaller still".

★ **The distinction that was actually load-bearing:**

- *"How many lines is this file"* — perfectly well-defined. Publish it.
- *"How much of it is production"* — not well-defined; a large fraction of
  the "production" count was comments, and every answer smuggles a
  convention.

The not-well-defined verdict belongs only to the second question. Letting it
wash back onto the first retires a sound measurement and costs the ring a
result it earned. **Keep the whole-file figure; drop the gloss, or state its
convention beside it.**

The start anchor was also stale by two commits in the first pass —
resolved by `git merge-base --is-ancestor` in both directions, not
inferred, which is the general discipline this corrects toward: verify a
span claim against the actual ancestry, don't eyeball it.

★ **The Steward-specific version:** authority here is over *practice and
sequencing*, not over facts about code someone else writes. **A number
published about someone else's file becomes load-bearing in their brief** —
multiple seats read the wrong 127 and one had to spend a turn correcting
it. Publishing derived measurements is doing the ring's job badly while
adding a correction obligation. The honest form is: state what must be
true, name the oracle, and let the ring report the value.

Sibling of [[playbooks-state-mechanism-not-intent]] — both are the same
error: supplying the *answer* where the *question* was what belonged in
the brief.
