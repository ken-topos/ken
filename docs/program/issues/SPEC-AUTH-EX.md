---
id: SPEC-AUTH-EX
title: "62-authority §7 worked examples are written in a retired surface — retired `view` keyword, retired `Cap_FS` spelling, and `write_at` for the landed `write_file`"
status: draft
owner: spec-enclave
size: S
gate: none
depends_on: [SPEC-IDENT-BLESSED]
blocks: []
github: null
origin: "Measured by the Steward 2026-07-27 at origin/main e700b861 while discharging the CAT-CAPEX ordering question. Not routed by any ring."
---

⛔ **`status: draft` is deliberate** — the frame is not written and the enclave
is building `SPEC-IDENT-BLESSED`. ⚠ `depends_on` records a **scheduling**
dependency, not a technical one.

## The measurement

`spec/60-security/62-authority.md §7` ("Worked examples") is the spec's **only**
worked example of the authority discipline, and the section doc chapter 04
cites as the authoritative prose artifact. It is stale on three independent
axes against landed code:

| axis | `§7` writes | landed |
|---|---|---|
| definition keyword | `view` | `const`/`fn`/`proc` — `view` **retired** by operator SURF-1 |
| capability type | `Cap_FS` | `Cap a`, authority-indexed (`Cap : Auth -> Type0`) |
| FS operation | `write_at` | `write_file` |

`Cap_FS` appears **8 times** in the chapter; `view` **9 times**.

## Why it matters beyond tidiness

1. ⭐ **It is the copy source.** `CAT-CAPEX` sends Team Ergo to write the first
   checked capability exemplar, and `§7` is the obvious template. Its frame
   has to spend a banned-shape clause warning Ergo **not** to copy the spec —
   which is the wrong shape for a spec to be in.
2. **Doc chapter 04 teaches from it**, under a `62 §7`-labelled
   "unavailable in checked form" note (`evt_4b9pp185rmbpm`, correctly ruled).
   The label is about *checkedness*, not about the surface being retired — a
   reader takes the spelling as current.
3. ⚠ **The examples' semantic content is sound.** No-ambient confinement,
   least authority, non-amplifiable delegation, the order-dual `AC3` pair, the
   `AC6` authority-plus-flow composition, and the three `UnboundName`
   management names are all correct and worth preserving. ⇒ **This is a
   surface-currency repair, not a rewrite.** Whoever takes it should change
   spellings and keep the argument.

## ⛔ Scope note for whoever frames this

⛔ Do **not** fold this into `CAT-CAPEX` — that WP is `catalog/`-only and
Ergo does not edit `spec/`. ⭐ The better sequencing is the pattern chapter 03
validated: let `CAT-CAPEX` land the **checked** exemplar first, then have `§7`
cite it, so the spec's worked example points at something that provably
elaborates instead of at a code block nobody runs.
