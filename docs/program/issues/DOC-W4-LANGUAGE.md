---
id: DOC-W4-LANGUAGE
title: "Wave 4 slice 2 — the language reference, scoped to whatever survives a residual measurement against the 625-line page already named `surface-reference`"
status: ready
owner: doc
size: S
gate: none
depends_on: [DOC-W4-TOOLCHAIN]
blocks: []
github: null
origin: "Steward 2026-08-02, under section 2a-bis, so the doc ring has a framed successor when DOC-W4-TOOLCHAIN merges. The depends_on edge is a GENUINE content dependency, not ring-capacity sequencing: this slice consumes D0's generation-capability answer to decide whether a syntax fact is labelled generated or authored. Measured at origin/main = 09931340."
---

# Wave 4, slice 2 — the language reference

Wave 4 produces `library/reference/` across language, verification, toolchain,
runtime, platform, and diagnostics. Slice 1 (`DOC-W4-TOOLCHAIN`) took the one
surface that needs no generator and produced `D0`, the durable
generation-capability report every later slice rests on. This slice takes the
language surface.

## The duplicate hazard, measured before framing

`library/guide/surface-reference.ken.md` is **625 lines** and its numbered
sections are exactly the language's declaration forms:

| section | form |
|---|---|
| 1 | purity keywords `const` / `fn` / `proc` |
| 2 | `def`, transparent definitions |
| 3 | `data` and `match` |
| 4 | refinement types |
| 5 | `class` and `instance` |
| 6 | effect rows (`visits`) |
| 7 | named proof claims `prop` / `theorem` / `proof` |
| 8 | local `let` binding groups |
| 9 | the `.ken.md` literate format |

**That page is titled "Surface reference", it is organised per form, and it is
classified `kind = "explanatory"`.** So the question this slice opens with is
not "what should the language reference contain" but **"is there a residual at
all, and if there is, what is it?"**

Wave 3 asked that question and it retired six of seven planned chapters —
`DOC-W3-DEPDATA` shipped one page where the program's list implied seven. The
same measurement is owed here, and it is `D0` of this slice.

## Audience is a real discriminator; kind is the one in question

`library/agents/core/read-ken.md` is already `kind = "reference"`,
`authority = "derived-reference"` — but `audience = ["agent-reader"]`. A
human-audience language reference does not duplicate it, on the same basis that
`library/reference/toolchain/` does not duplicate
`library/agents/core/toolchain.md`. **That axis is settled and is not this
slice's question.**

The open axis is `surface-reference.ken.md`: same audience, same subject,
adjacent kind. Slice 1's judgment 1 discriminates a reference (answers a lookup)
from a how-to (directs a task); it does not discriminate a reference from an
explanatory page that is organised per form and calls itself a reference.
**Settling that is this slice's first deliverable, not an assumption it may
make.**

The frame is `docs/program/wp/DOC-W4-LANGUAGE.md`.
