---
id: DOC-W5A-CARD-FORMAT
title: "Wave 5 slice 1 — the reference card format, the generated subject index for all 39 packages, and six proving cards across Core and Tooling"
status: merged
owner: doc
size: M
gate: none
depends_on: [DOC-W5-CAPABILITY]
blocks: [DOC-W5B-CARDS-APP-DATA]
github: null
origin: "Steward 2026-08-02 under section 00 — the doc ring held zero ready nodes after DOC-W5-CAPABILITY merged. Implements the mixed fork that the capability report's D2 recommended: five populated fact classes, four held. Measured at origin/main = a8df4b7b."
---

# Wave 5 slice 1 — establish the card, prove it on six packages

Wave 5 produces one reference card per live catalog package. This slice builds
the format and proves it on the two smallest areas, Core (5) and Tooling (1).
Slices 2 and 3 apply it to the remaining 33.

## What the frame settles that this node must not be read without

**"None" has two meanings and the card must distinguish them.** A row reading
"Effects: none" is either *the package declares none* — a fact read off
canonical fences — or *we could not determine whether it has any*, which is an
absence of measurement that looks identical on the page. The capability report
already hit this: `Capability/Filesystem/Authority.ken.md` has no law
declaration, and the report was careful that "none declared" there is human-read
rather than emitted.

So the disposition vocabulary is closed and has four values, not two:
`generated | authored | none-declared | held`. **No row may be blank**, and the
four held classes appear on every card rather than being omitted — silence
would read as an answer.

**The four held classes stay held.** Platform and maturity are reserved by
`docs/program/06-catalog-campaign.md:119-121` but uninstantiated; dependency and
reverse-dependency have no package-level projection. A card **reports** that gap
and never creates a convention to fill it — the same boundary that blocked
[[DOC-W5-CAPABILITY]]'s first candidate.

**Only the subject class may be presented as generated.** It is the one class
the report measured as mechanically extractable.

The frame is `docs/program/wp/DOC-W5A-CARD-FORMAT.md`.
