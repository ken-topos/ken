# Reading-Ken exercises

These exercises practice the reading discipline chapters
[01](../reading-ken/01-anatomy.md)–[06](../reading-ken/06-execution.md) teach,
against the same registered fragment set
([`fragments.md`](../reading-ken/fragments.md)) those chapters already
grounded — no new fragment, spec section, or producer test is introduced
here. Each exercise names an explicit **learning objective**: the specific
reading skill it practices, not just the chapter it follows.

## What "checked" means for a reading exercise

This curriculum's chapters are checked against real, `ken check`-passing
code. These exercises are about a different thing — reading comprehension of
already-checked material — so "checked" here does not mean "run through the
`ken` compiler" (there is no new Ken code in an exercise to check that way).
It means what every other page in this library already means by it: **every
expected answer is grounded in an explicit citation** to the exact fragment
passage or spec section that decides it, verified current by the same
**content-currency** mechanism as every other page
(`scripts/gen-doc-status.sh`, `library/REVISION`). A reader can verify any
answer by opening the cited source directly, rather than trusting this
page's authority for it — the same discipline every chapter in this
curriculum already follows.

## Structure

- [`exercises.md`](exercises.md) — the questions, organized by chapter,
  each with its explicit learning objective and a real fragment/spec
  citation to work from.
- [`solutions.md`](solutions.md) — the matching answers, in a **separate
  file** so a reader working the exercises does not see an answer by
  scrolling past it. Each solution states the exact citation that decides
  it, so it is checkable, not just assertable.

Work the exercises against the fragment or spec section itself, not against
memory of the chapter's prose — several of these questions exist precisely
because a fragment's own casual wording and the normative section that
actually governs it are not always the same claim (chapter
[05](../reading-ken/05-packages-and-provenance.md) §3 is the worked example
of exactly this trap).

---

**Grounds this page:** `library/learn/reading-ken/fragments.md`. Authority
class: `tutorial` — this page organizes practice against material chapters
01–06 already ground; it introduces no new source claim. Content-currency
predicate, as elsewhere.
