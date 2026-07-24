---
id: LOADER-CITE-ANCHOR
title: "LOADER-STALE-PREMISE cites the spec by line number (:147-158) — rots silently in the one catalog file outside the currency gate"
status: merged
owner: doc
size: XS
gate: none
depends_on: [LOADER-STALE-PREMISE]
blocks: []
github: null
origin: adversary post-merge hunt on LOADER-STALE-PREMISE @ 2024d3eb (evt_yn2235fjvswe, 2026-07-23)
---

**Low severity — durability, not a live error.** `:147-158` is correct
*today* (read and confirmed). This is queued to ride with the next doc batch,
**not** a standalone merge. The adversary's own sizing.

## The gap — the repair reintroduces its own defect class, subtler

`LOADER-STALE-PREMISE`'s thesis was *"each repaired claim now cites the
normative spec directly."* But all three prose sites cite by **line number** —
`spec/30-surface/33-declarations.md:147-158` — and line numbers are the most
drift-prone locator: any re-flow of the spec moves them **with nothing red.**
That is the *same silent-rot mechanism* as the `no disk loader yet` premise it
replaced — a true statement that quietly becomes false in place.

**Whether the rot is caught splits by file (all verified against `2024d3eb`):**

| site | currency status |
|---|---|
| `library/learn/reading-ken/fragments.md:91` | manifest doc citing 33-declarations → **backed** (a spec edit trips currency → Librarian re-reads) |
| `catalog/packages/README.md:48` | not tracked; cited by `05-packages` which also cites the spec → **indirectly backed** (weak) |
| `catalog/guide/README.md:88` | ⛔ **neither a tracked doc NOR a cited source** — manifest count 0. Its spec pointer is entirely outside the currency mechanism. Spec drift = **silent**. |

⇒ `catalog/guide/README.md:88` is the load-bearing case: nothing re-validates
its `:147-158` when the spec moves. The WP replaced a premise that rotted
silently with a citation that rots silently, **in the one file the currency
gate cannot see.**

## The fix is nearly free and already used one file over

The **manifest** cites this exact section by **anchor** —
`#32-importing-and-exporting` (verified: `library/manifest.toml:106,257`) —
which is drift-proof: re-flowing the spec does not move a heading anchor.

**Swap the three prose `:147-158` line-number cites for the
`#32-importing-and-exporting` anchor.** Uniform — the anchor is strictly better
than a line range everywhere, and it closes the rot without depending on
currency coverage the catalog files do not have.

## Scope

Exactly three edits:
- `catalog/guide/README.md:88` — the load-bearing one
- `catalog/packages/README.md:48`
- `library/learn/reading-ken/fragments.md:91` (a `library/` currency-bearing
  file — the anchor swap is a citation-locator change, not a source change, but
  run the ledger check on the result)

## Acceptance

- No `spec/…:NNN-NNN` line-number citation remains in the three repaired sites;
  each uses `#32-importing-and-exporting` (or the correct current anchor if the
  section heading has since changed — verify the anchor resolves).
- The anchor **resolves** in `33-declarations.md` today (grep the heading).
  ⛔ Do not swap a live-but-fragile line cite for a dead anchor.
- §8a split: `catalog/**` → Architect; `library/**` → Librarian. `fragments.md`
  rides the Librarian's ledger check.

★ **This is `hunt-the-correction-inherits-the-defect-class` landing on a doc
WP** — the fix for a silent-rot premise chose a silent-rot locator. Sibling of
[[dont-publish-derived-code-measurements-state-the-property-let-the-ring-measure]]:
a citation is only "current" if something re-validates it; a line number in an
un-tracked file is validated by nothing.
