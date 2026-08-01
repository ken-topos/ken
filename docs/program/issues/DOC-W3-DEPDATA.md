---
id: DOC-W3-DEPDATA
title: "Wave 3 slice 3 — the dependent-data guide page, the one guide subject of seven with no explanatory coverage anywhere in library/"
status: ready
owner: doc
size: S
gate: none
depends_on: [DOC-W3-GUIDE]
blocks: []
github: null
origin: "Steward 2026-08-01, from the Wave 3 guide-subject residual measurement. Wave 3 names seven guide subjects; six already carry explanatory pages and only dependent data has none. Measured at origin/main = c777d2d4."
---

# Wave 3, slice 3 — dependent data

Wave 3 produces `library/guide/` "filled in demand order — contracts,
dependent data, proofs, effects, security, packages, execution." Slices 1 and 2
delivered the `catalog/guide/` migration and `library/how-to/`. This slice is
the residual of that seven-subject list, and the residual is **one subject**.

## The measurement, and why this node is small

Before framing anything here, the seven subjects were set against what
`library/` already holds. The result is that the guide is substantially
delivered already, by pages that were not filed under `guide/`:

| Wave 3 subject | existing explanatory coverage |
|---|---|
| contracts | `learn/reading-ken/02-types-contracts-and-proofs.md`; `guide/surface-reference.ken.md` §1 purity keywords, §4 refinement types |
| **dependent data** | **none** |
| proofs | `learn/reading-ken/02`; `guide/proof-techniques.ken.md` (six sections) |
| effects | `learn/reading-ken/04-effects-capabilities-and-authority.md`; `guide/surface-reference.ken.md` §6 |
| security | `learn/reading-ken/04` (Capabilities); `learn/reading-ken/03-assurance-and-trust.md` (Certificates and Trust) |
| packages | `learn/reading-ken/05-packages-and-provenance.md` |
| execution | `learn/reading-ken/06-execution.md` |

**The load-bearing fact is that those pages are already the right shape.** All
six spine chapters carry `kind = "explanatory"` and `authority = "explanatory"`
in `library/manifest.toml`, as do the three migrated guide pages. They are not
tutorials that a conceptual page would sit beside — they are the conceptual
pages, filed under `learn/` rather than `guide/`. Wave 3's exit property
(*tutorials teach, how-tos direct work, and conceptual pages explain*) is
already met for six of the seven subjects, twice over for several.

⇒ **Framing seven chapters here would re-author material that exists and is
correctly classified.** That is the failure this program has already made
twice, on L5 and V3, and it is the reason this node covers one subject.

## Why dependent data is a real residual and not a manufactured one

`Vec` occurs **zero times** anywhere under `library/`. So does `Fin`. The only
occurrences in the repo outside `spec/` are one contrastive mention in
`catalog/packages/Data/Collections/Derived.ken.md` (noting that the `List` zip
is *not* the length-indexed one) and the conformance challenge
`conformance/challenge/C4-indexed-vec-head/vec-head.ken`.

The capability is not aspirational. `spec/50-stdlib/60-length-indexed-vectors.md`
is normative, and its §4 table records the family constructors, `head`, the
`Fin` declaration, and `tail` as **landed**, with `zip` and `lookup` gated on
`DS-5c`. So there is a real, checked, present-tense subject to explain, with an
honest boundary to label rather than a lane to describe hopefully.

The frame is `docs/program/wp/DOC-W3-DEPDATA.md`.
