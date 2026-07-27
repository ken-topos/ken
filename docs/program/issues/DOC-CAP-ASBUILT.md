---
id: DOC-CAP-ASBUILT
title: "The capability chapter tells readers the catalog has no checked authority exemplar; CAT-CAPEX adds one, falsifying that claim in two places"
status: draft
owner: doc
size: S
gate: none
depends_on: [CAT-CAPEX]
blocks: []
github: null
origin: "Measured by the Steward 2026-07-27 at origin/main d6df571e while scoping the doc ring's next WP. Not routed by any ring."
---

⛔ **`status: draft` is deliberate and the gate is real.** The chapter's claim
is **true today**. It becomes false only when `CAT-CAPEX` lands
`catalog/packages/Capability/Filesystem/Authority.ken.md` on `main`. ⛔ Editing
it before then would make the corpus wrong in the other direction.

⚠ **`CAT-CAPEX` is not yet merged** — its first candidate (`d52611f5`) failed
CI on the catalog formatter fixed-point gate and is being respun. Flip this to
`ready` only after the fragment is present on `origin/main`.

## The measurement

`library/learn/reading-ken/04-effects-capabilities-and-authority.md`:

| line | text |
|---|---|
| `:115` | `§7's worked examples (unavailable in checked form — spec pseudocode, not a catalog fragment)` |
| `:125` | `The authority specification is normative prose that the catalog has not instantiated as checked code.` |

`CAT-CAPEX` instantiates exactly that: a checked fragment taking an explicit
`(cap : Cap a)` over the landed `read_bytes`, elaborated by a named test with
paired positive/negative controls.

## ⭐ The repair PROMOTES the chapter's argument — it is not a deletion

`:126-128` already says the boundary *"is **not merely** 'no checked fragment
exists yet' — it is that `attenuate`/`revoke` are, by design, never going to be
something a Ken program calls at all."*

⭐ **That sentence is correct and `CAT-CAPEX` makes it *more* true.** The
chapter was hedging against a reader who might think the gap was just missing
work. The "no checked fragment" half is now discharged **by fact**, leaving the
designed part standing alone — which is what the chapter was pointing at.

⛔ Do not weaken, qualify, or restructure the `attenuate`/`revoke` argument.
It is landed truth (`38 §1.3.1` requires all three unbound; `62 §4` puts
narrowing in the trusted host), and it is why the section exists.

## ⛔ Do NOT fix the chapter's `Cap_FS`

It occurs once, in a refinement-type example paraphrasing `62 §2.2` — and
**`62` itself still writes `Cap_FS`**. Correcting the doc ahead of the chapter
it cites manufactures a doc/spec mismatch. It follows `SPEC-AUTH-EX`.

## Frame

`docs/program/wp/DOC-CAP-ASBUILT.md`, with fixed inputs pinned by blob at
`origin/main = d6df571e`. ⚠ The frame was written **before** the fragment
landed; the landed fragment is authority over it.
