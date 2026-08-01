---
id: DOC-CAP-ASBUILT
title: "The capability chapter tells readers the catalog has no checked authority exemplar; CAT-CAPEX adds one, falsifying that claim in two places"
status: ready
owner: doc
size: S
gate: none
depends_on: [CAT-CAPEX]
blocks: []
github: null
origin: "Measured by the Steward 2026-07-27 at origin/main d6df571e while scoping the doc ring's next WP. Not routed by any ring."
---

✅ **RELEASED 2026-08-01 — the gate is discharged and this is now `ready`.**
`CAT-CAPEX` is `merged`; both artifacts are present on `origin/main =
6de2a099`, verified by the Steward at release: the fragment
`catalog/packages/Capability/Filesystem/Authority.ken.md` and its elaborating
test `crates/ken-elaborator/tests/cat_capex_authority.rs`.

⇒ **The chapter's `:115`/`:125` claims are now FALSE and the repair is owed.**
The former `status: draft` was gating exactly on this and nothing else.

⚠ **Re-ground at pickup anyway.** This node's original measurement was taken
2026-07-27 at `d6df571e`, and the doc track then sat idle five days while the
runtime campaign moved `main`. Re-read both claim sites at the head you
actually work against before editing; ⛔ do not carry the line numbers in the
table below as current without rechecking them.

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
