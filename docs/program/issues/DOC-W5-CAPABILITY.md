---
id: DOC-W5-CAPABILITY
title: "Wave 5 precondition — the Librarian's format-capability report: which of Wave 5's nine fact classes the checked artifact format can express today, and therefore whether Wave 5 is authorable or blocked on a generator"
status: ready
owner: doc
size: S
gate: none
depends_on: [DOC-W4-RESIDUAL]
blocks: []
github: null
origin: "Steward 2026-08-02, under section 2a-bis. The documentation program itself requires this report BEFORE Wave 5 is framed ('Before this wave is framed, the Librarian reports which of those facts the checked artifact format can express today and which cannot'). DOC-W4-RESIDUAL's eight-row D0 made it urgent rather than procedural: six of its eight rows closed `not-producible` for want of exactly this kind of extraction path. Measured at origin/main = 40f8757d."
---

# Wave 5 precondition — can the format express what Wave 5 promised?

Wave 5 commits to *"one generated reference page or card per live package,
plus subject, declaration/type, law, effect/capability, assurance, platform,
maturity, dependency, and reverse-dependency indexes."*
`12-documentation-program.md` §4b attaches a precondition to that commitment,
in its own words:

> **D4 is a commitment the toolchain must actually be able to keep, and Wave 0
> did not establish that it can.** Before this wave is framed, the Librarian
> reports which of those facts the checked artifact format can express **today**
> and which cannot. **A fact we cannot generate gets authored and labelled as
> authored — never generated-looking prose.**

This node is that report, and nothing else.

## Why it stopped being procedural

`DOC-W4-RESIDUAL` measured eight Wave 4 rows and **six closed
`not-producible`** — symbol index, keyword index, diagnostic index, glossary,
platform, diagnostics — every one of them for want of an inventory, a registry,
or a generator. Wave 5's index set is larger and more derived than Wave 4's.

⇒ **Running Wave 5's precondition now is the difference between discovering
that in a framing pass and discovering it three slices in.** Wave 4 discovered
its emptiness incrementally across three nodes. Wave 5 gets one measurement up
front.

## Unlike Wave 4, the subject is real

Measured at `40f8757d`: `catalog/packages/` holds **39 leaf packages**, every
one a literate `.ken.md` the toolchain checks, across four sections
(Application, Capability, Core, Data, Tooling).

That is a substantial, current, checked corpus — **not** the "the guide already
covers this" situation that closed Wave 4's language and residual slices. If
the facts are extractable, Wave 5 has real work. If they are not, Wave 5 needs
`crates/` work before any doc slice can start.

**That fork is the whole deliverable, and it is an operator-level answer.**

The frame is `docs/program/wp/DOC-W5-CAPABILITY.md`.
