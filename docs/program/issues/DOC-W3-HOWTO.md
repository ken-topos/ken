---
id: DOC-W3-HOWTO
title: "Wave 3 slice 2 — library/how-to/ recipes scoped by the CLI's seven-subcommand task surface, each grounded in a real diagnostic or checked artifact"
status: ready
owner: doc
size: M
gate: none
depends_on: [DOC-W3-GUIDE]
blocks: []
github: null
origin: "Steward 2026-08-01, under section 2a-bis (stay one release ahead of the frontier). DOC-W3-GUIDE was the doc ring's only node and nothing succeeded it. Measured at origin/main = 0cde815f."
---

# Wave 3, slice 2 — the how-to recipes

Wave 3 produces `library/guide/`, `library/how-to/`, and the `catalog/guide/`
migration. Slice 1 (`DOC-W3-GUIDE`) takes the migration and the conceptual
pages that sit directly on it. **This slice is `library/how-to/`.**

## Why this is a separate slice, and why it is releasable

The program says the recipes are driven by **actual diagnostics and recurring
failures, not by an imagined task list**. That is a different input from slice
1's, and it is the reason the two were not framed together: slice 1's input is
material that already exists in `catalog/guide/`, and this slice's input is the
behaviour of the toolchain.

That input is now measured, so the ring does not have to stop for a research
act. The task surface is small and closed — the CLI offers exactly seven
subcommands — while the refusal population behind it is large enough that
per-diagnostic coverage is not a coherent goal. The frame settles how to scope
against that asymmetry.

## The shape, stated up front: recipes DIRECT WORK; they do not explain

Wave 3's exit property is that *tutorials teach, how-tos direct work, and
conceptual pages explain; no single page is forced to do all three.* This slice
is the only one of the three that has no existing pages at all — `library/`
holds zero `how-to` records today, though the manifest's closed vocabulary has
admitted `kind = "how-to"` and `authority = "how-to"` since Wave 0.

The pressure to explain will be real, because the Wave 1 spine
(`library/learn/reading-ken/`) already holds six chapters covering six of Wave
3's seven guide subjects, and slice 1 adds conceptual pages on top. A recipe
that restates any of that is the failure this slice is designed against. Link
and move on.

## The one judgment that decides the page set

A recipe is scoped by **a task the toolchain actually offers**, not by a topic
someone might want covered. The frame fixes the enumeration to the seven
subcommands and requires every recipe to name the real diagnostic or checked
artifact it resolves. A task with no grounding is a Wave 3 gap to report, not
prose to invent.

The frame is `docs/program/wp/DOC-W3-HOWTO.md`.
