---
scope: roles/steward
audience: (see scope README) — anyone framing a corpus-wide surface
  migration or a keyword reservation
source: 2026-07-12, two incidents same day — the `match-arm-glyph`
  migration and a keyword-reservation build WP (N4 Lane B)
---

# A migration sweep glob must enumerate every Ken-source root

When a WP does a **corpus-wide surface substitution** (glyph, keyword, or
exported-name migration), the sweep is only as complete as its **file
glob** — and the obvious roots (`catalog/**`, `crates/**/tests`,
`prelude.rs`) are **not the whole corpus.** Ken-source lives in more places
than that:

- `examples/rosetta/*.ken` (16 standalone example programs, confirmed
  still 16 in the current tree)
- `tooling/highlight-js/sample.ken` + the highlighter's own operator regex
  (`ken.js`) + its README table
- playbook/guide `.md` files with ` ```ken ` fences
  (`agent/playbooks/tools/write-ken.md`)
- any other `.ken` / `.ken.md` outside `catalog/`

**2026-07-12 (`match-arm-glyph` migration, `⇒`→`↦`):** PR1's sweep glob
missed `examples/rosetta/*.ken` (×16), a catalog fixture, and the
highlighter's `sample.ken` — they still held live old-glyph match arms
after PR1 "completed." PR1's own audit reported `files=0` because the
audit used the same under-covering glob. The gap only surfaced in **PR2
(the removal phase)**: once the old glyph stopped lexing as the retired
token, any missed live arm became a build/parse error, so PR2's
**precondition audit** ("zero retired tokens before I remove support") plus
a full green workspace build caught and forced the residual migration
(hundreds of separators across more files than PR1 touched).

**2026-07-12 recurrence — a keyword reservation hit the same trap.** A
build WP reserved several identifiers as keywords. QA's locked workspace
test suite was green, yet a rosetta `.ken` example used one of the reserved
words as a `const` identifier → unparseable after merge, latent because
rosetta `.ken` files aren't wired into any workspace test (the same missed
root as the glyph migration). The Architect caught it at terminal review by
sweeping every source root directly. Fix: fold the one-identifier rename
into the reserving WP's merge (atomic, self-documenting), re-confirmed by
the Architect. A raw grep massively over-counts here (comment/prose hits
like "this program" swamp real code collisions) — the sweep needs to be
**fence-aware and comment-stripped**, extracting `.ken` bodies and ` ```ken `
fences before matching identifiers at a word boundary.

**How to apply:**

- When framing a corpus-wide migration **or a keyword reservation**, put
  "enumerate every Ken-source root" in scope explicitly — list `examples/`,
  `tooling/`, playbook docs, not just `catalog`+`crates`. Don't trust a
  glob that "found them all" if it only globbed the obvious roots; an
  audit inherits its own glob's blind spots.
- **Design the migration as migrate-then-REMOVE where possible:** the
  removal phase's "must be zero before I remove" precondition plus a full
  workspace build is a *structural* completeness proof (a missed live site
  fails to compile) — far stronger than an audit grep that can share the
  sweep's blind spot. This is why the additive→migrate→remove staging is
  worth the extra PR.
- Generalizes
  [[exported-name-migration-needs-whole-harness-consumer-inventory]] — same
  root failure (under-scoped inventory), there on the consumer/test side
  rather than the producer/source side.
