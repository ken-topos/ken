---
id: DOC-W4-RESIDUAL
title: "Wave 4 slice 3 — the terminal residual measurement across the four remaining reference surfaces and the four indexes, authoring only what survives it"
status: ready
owner: doc
size: S
gate: none
depends_on: [DOC-W4-LANGUAGE]
blocks: []
github: null
origin: "Steward 2026-08-02, under section 2a-bis. Folds the remaining Wave 4 surfaces (verification, runtime, platform, diagnostics) and the four indexes into ONE residual measurement rather than four authoring nodes, because slices 1 and 2 both measured their subject already covered and the same instrument answers all four in one pass. Measured at origin/main = 3b873896."
---

# Wave 4, slice 3 — what is actually left

Wave 4 committed to `library/reference/` across six surfaces plus four
indexes. Two slices have landed and both changed the picture:

- **Slice 1 (`DOC-W4-TOOLCHAIN`)** produced the durable generation-capability
  report. No generator extracts a declaration, keyword, syntax production, or
  CLI surface; the CLI emits no machine-readable output; Ken has no diagnostic
  registry.
- **Slice 2 (`DOC-W4-LANGUAGE`)** measured nine language forms against the
  existing 625-line guide and found the named-gap set **empty**. Zero pages
  were authored, and that was the correct outcome.

Four surfaces remain — verification, runtime, platform, diagnostics — plus the
symbol, keyword, diagnostic, and glossary indexes.

## Why one node and not four

Two of the four are already suspect on **mechanism**, not on residual:

- **diagnostics** — measured at `3b873896`: there is no diagnostics crate and
  no diagnostic registry. A generated diagnostic index is not producible, and
  an authored one would be a hand-maintained list of strings that rots at the
  next error-message edit.
- **platform** — `12-documentation-program.md` §4b already warns that this is
  where the wave's authoring commitment is hardest to hold honestly:
  cross-platform is indefinitely deferred (operator, L2-1), and a page
  describing a deferred lane in the present tense is aspirational syntax by
  another name.

The other two — verification and runtime — face the identical question slice 2
just answered: the human-audience explanatory corpus may already deliver them.

⇒ Four separate authoring nodes would each open with the same measurement over
the same ~2,400-line corpus, and at least two would predictably return "no
subject." That is proliferation without a grounded constraint demanding it.
**One measurement answers all four**, and the authoring that survives it is
whatever the measurement licenses.

## The corpus being measured against

| surface | the human-audience material that may already deliver it |
|---|---|
| verification | `library/guide/proof-techniques.ken.md` (474 lines), `learn/reading-ken/02-types-contracts-and-proofs.md` (142), `03-assurance-and-trust.md` (221) |
| runtime | `learn/reading-ken/06-execution.md` (239), `04-effects-capabilities-and-authority.md` (171) |
| platform | no obvious human-audience page; the question is whether an honest one is writable at all today |
| diagnostics | no registry, no page; `library/agents/tasks/diagnose.md` is `agent-reader` |

The frame is `docs/program/wp/DOC-W4-RESIDUAL.md`.
