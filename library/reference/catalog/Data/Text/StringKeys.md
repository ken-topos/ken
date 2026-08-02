# `Data.Text.StringKeys` — lawful String equality and order

Availability: **partial**

Authority: **derived reference**

Canonical source: [checked package](../../../../../catalog/packages/Data/Text/StringKeys.ken.md).

| Fact class | Disposition | Result and grounding |
|---|---|---|
| Subject | `generated` | `catalog/packages/Data/Text/StringKeys.ken.md` — “`Data.Text.StringKeys` — lawful String equality and order,” from the [subject index](../../subjects.md). |
| Declaration/type | `authored` | Declares transported equality and ordering operations plus lawful `DecEq String` and `Ord String` dictionaries over the checked `List Char` views. |
| Law | `authored` | Checked soundness, completeness, reflexivity, antisymmetry, transitivity, and totality proofs inhabit the dictionary fields. |
| Effect/capability | `none-declared` | The canonical checked fences declare no effect row, `proc`, `visits`, or capability value for this package. |
| Assurance | `authored` | The package contains no `Axiom`; equality-producing fields cite the separately homed injectivity certificate, and all other fields use dictionary projections and congruence. |
| Platform | `held` | No checked per-package facet is available; owner: catalog campaign. See the [held-class disclosure](../../card-format.md#held-class-disclosure). |
| Maturity | `held` | No checked per-package facet is available; owner: catalog campaign. See the [held-class disclosure](../../card-format.md#held-class-disclosure). |
| Dependency | `held` | No complete package-level checked projection exists; owner: `crates/`. See the [held-class disclosure](../../card-format.md#held-class-disclosure). |
| Reverse dependency | `held` | No complete projection and maintained inversion exist; owner: `crates/`. See the [held-class disclosure](../../card-format.md#held-class-disclosure). |
