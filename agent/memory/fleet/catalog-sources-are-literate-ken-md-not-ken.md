---
scope: fleet
audience: (see scope README) — anyone grepping `catalog/` for a proof-vocab
  completion check, a rename-completeness sweep, or any "should be zero"
  survey over catalog sources
source: private memory `catalog-sources-are-literate-ken-md-not-ken`
---

# catalog/ sources are literate `.ken.md`, not `.ken` — a vacuous glob returns zero

The `catalog/` corpus (packages + guide) is stored almost entirely as
**literate `.ken.md`** files — Ken code lives in fenced blocks inside
markdown — NOT as `.ken` files.

**The trap:** `git grep -E '\btt\b' <sha> -- 'catalog/**/*.ken'` matches
ZERO files and returns nothing. Read as "no surface `tt` in the catalog"
that is a **false negative** — nearly cast a proof-vocab-completion vote on
a rename-completeness check that grepped a file glob with (almost) no
members. A vacuous grep and a genuinely-clean grep both print nothing;
distinguish them.

**How to apply:** grep `catalog/**` (all files) or `catalog/**/*.ken.md`,
never `catalog/**/*.ken` alone. When a "should be zero" grep returns zero,
first confirm the glob actually matches files (`git grep -l -- '<glob>'` or
`git diff --name-only` shows the real extensions). This is the catalog-side
twin of the prelude-emission trap (Rust-emitted prelude code vs. `.ken`
sources) — the shared lesson: **ground the grep's file set before trusting
a zero result.**

**Currency check (2026-07-28):** as of this writing `catalog/` holds 43
`.ken.md` files and exactly **one** `.ken` file
(`catalog/packages/Tooling/Verification/ProofErasureBoundaryChecker.ken`),
so `catalog/**/*.ken` is no longer *strictly* vacuous — it now silently
returns a near-empty, misleadingly-partial result instead of zero, which is
the same trap in a subtler form. The rule is unchanged: always widen the
glob or verify file-set membership before trusting the grep's silence.
