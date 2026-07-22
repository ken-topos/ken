---
id: LOADER-STALE-PREMISE
title: "\"no disk loader yet\" is stale in 9 places — including already-landed library/ content"
status: ready
owner: doc
size: S
gate: none
depends_on: []
blocks: []
github: null
origin: librarian evt_4sf7y15s0jbpx (DOC-W1-4 ch05 review)
---

**Surfaced by @librarian reviewing DOC-W1-4 ch05, and correctly scoped OUT of
that WP.** The chapter fix belongs to ch05; **the upstream premise is a
separate, cross-cutting fact** that will keep re-infecting chapters until it is
repaired at the source.

## The contradiction

**Current normative spec — `spec/30-surface/33-declarations.md:147-158`:**

> The loader discovers compilation units lazily, following `import` and facade
> `export M (…)` edges from units already being compiled… Each unit is loaded
> and elaborated at most once in a compilation run.

**The real producer exists and passes:** `crates/ken-elaborator/tests/n2_in_repo_loader.rs`
writes separate `A.ken.md`/`B.ken.md` and proves `A` resolves `B.value`.
@librarian ran it independently on the ch05 candidate; green.

**Nine locations still assert the opposite:**

```
catalog/guide/README.md:88                  "does not resolve yet (no disk loader)"
catalog/packages/README.md:47               "does not resolve yet (no disk loader)"
docs/program/07-catalog-style-guide.md:476  "There is no disk loader yet"       <- THE ROOT CITATION
docs/program/diary/2026/Jul/09.md:101       (diary — historical, leave)
docs/program/wp/cc3-parsing-cursor-decoder.md:119
docs/program/wp/cc4-diagnostic-core.md:96
docs/program/wp/cc5-pretty-doc.md:75
docs/program/wp/cc9-test-property.md:97
library/learn/reading-ken/fragments.md:90   ⛔ ALREADY LANDED IN library/
```

## ⛔ Why this is `ready` and not a note

**`library/learn/reading-ken/fragments.md:90` is live, published, reader-facing
content asserting a capability limit that does not exist.** It cites
`07-catalog-style-guide.md §13` as its authority — so the stale root has
already propagated into the library once, unaided.

★ **And it is a *laundering* path, not just a wrong sentence.** ch05 inherited
it from the same root and turned a **corpus-usage gap** into a **language/
tooling capability gap** — those are different claims with different
consequences for a reader deciding whether Ken can do something. The honest
fact is narrow: *today's catalog entries do not exercise the live loader.* The
stale premise inflates that into *the loader does not exist.*

⇒ Each new chapter that touches imports meets the same root citation and can
make the same upgrade. **One repair at the source is cheaper than N chapter
reviews catching it** — and the ch05 review is evidence the catch is not
automatic: the branch was **mechanically green** (21/21 gates, `gen-doc-status
--check` clean) while carrying the wrong claim.

★ **That green-vs-wrong result is the point:** source-currency proves the cited
bytes are unchanged. **It cannot prove a stale-but-unchanged source still
carries the semantic claim.** This is precisely the DOC-W0 defect class, one
level up — the citation is current *and* wrong.

## Scope

1. **Repair the root** — `docs/program/07-catalog-style-guide.md:476` §13.
   Restate as a **corpus-coverage** fact: the loader resolves cross-file
   `import`; today's catalog entries inline helpers rather than importing them.
2. **Repair the two catalog READMEs** (`catalog/guide/README.md:88`,
   `catalog/packages/README.md:47`).
3. **Repair the landed library content** —
   `library/learn/reading-ken/fragments.md:90`. ⚠ This is `library/`, so it
   carries the wave's currency requirements; re-anchor its citation to the
   normative spec section, not to the style guide.
4. **The four `docs/program/wp/cc*.md` frames are CLOSED WP records** — they
   were true when written. ⛔ **Do not rewrite history.** Leave them; if
   anything, they are evidence of when the premise was correct.
5. **`docs/program/diary/` is a diary.** ⛔ Leave it.

⇒ **Four files change, five are deliberate leaves.** Classify and make the
leaves visible, same discipline as CB-HYGIENE's Class C.

## Acceptance

- **No non-historical source asserts the absence of a disk loader.** Emit the
  occurrence set against the exact candidate and classify every row
  changed / left-with-reason.
- **Each repaired claim cites the normative spec** (`33-declarations.md`), not
  another doc that cites it. ★ The failure mode here was a **citation chain**
  where only the root was ever checked.
- ⛔ **Do not overcorrect.** The narrow true fact — *catalog entries do not
  currently exercise cross-file import* — must survive. Deleting it would
  replace a false limitation with a false capability claim.

**Owner: doc.** Sequence **after DOC-W1-4 merges** — it touches
`library/learn/reading-ken/`, and ch05 is live in that directory now.
