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
