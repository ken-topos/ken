# Catalog card format

Availability: **partial**

Authority: **derived reference**

Catalog cards are compact lookup pages for facts that the checked package and
its grounded sources can support today. They do not create package metadata or
fill facts that the repository does not yet expose.

## Required rows

Every card has exactly these nine nonblank fact rows. Each row uses exactly one
disposition from the closed set `generated`, `authored`, `none-declared`, or
`held`.

| Fact class | Specimen disposition | Specimen content and grounding |
|---|---|---|
| Subject | `generated` | Copy the exact path and heading from the [generated subject index](subjects.md). |
| Declaration/type | `authored` | Summarize the public declarations in the canonical checked package. |
| Law | `authored` | Summarize declared laws and their checked witnesses. |
| Effect/capability | `none-declared` | Use only after reading the canonical fences and finding no declaration for this class. |
| Assurance | `authored` | Summarize the package's checked proof closure and stated trust boundary. |
| Platform | `held` | Link to the common held-class disclosure; do not supply a package answer. |
| Maturity | `held` | Link to the common held-class disclosure; do not supply a package answer. |
| Dependency | `held` | Link to the common held-class disclosure; do not infer from prose or imports. |
| Reverse dependency | `held` | Link to the common held-class disclosure; do not invert incomplete data. |

## Authoring rule

Generate only the subject row, from the path-preserving index command recorded
in `subjects.md`. Author declaration/type, law, effect/capability, and assurance
from the canonical checked package fences and their cited current sources.

Use `none-declared` only when those canonical fences were inspected and
genuinely declare no fact in that class. Name that evidence in the row; silence
in a summary paragraph is insufficient. The four held rows are mandatory and
remain held on every card. They link to one common disclosure and never acquire
package-specific answers in this slice.

Cards are explanatory, derived references. When a concise description would
turn into a normative restatement, cite the owning checked source instead.

## Held-class disclosure

The following gaps apply uniformly to every catalog card. A card links here
instead of inventing a package-specific answer.

- **Platform — `held`.** The catalog campaign reserves a `platform` metadata
  facet but no checked package instantiates it. The catalog campaign owns the
  missing per-package instantiation and the still-open convention decision.
- **Maturity — `held`.** The campaign likewise reserves `maturity` without a
  checked per-package value. The catalog campaign owns the missing
  instantiation and open convention decision.
- **Dependency — `held`.** Checked core has declaration-level dependency data,
  but no complete package-level projection exists for catalog leaves. The
  `crates/` implementation owns that missing projection.
- **Reverse dependency — `held`.** This requires the complete package-level
  dependency projection plus a maintained inversion over the full population.
  The `crates/` implementation owns both mechanisms.

The reserved facets are recorded under
[Sections, Domains, Subdomains, and Packages](../../../docs/program/06-catalog-campaign.md#sections-domains-subdomains-and-packages),
at lines 119–121. This disclosure reports the measured gaps; it proposes no
metadata convention, field, schema, generator, or package value.

## Authored-fact rot

The four authored classes require manual review when their checked sources
change:

- **Declaration/type:** any public-name or signature change.
- **Law:** adding, removing, renaming, or changing the status of a law.
- **Effect/capability:** an effect row, capability index, boundary, or
  pure-to-effectful change.
- **Assurance:** a change to proof closure, assumptions, or the trusted base.
