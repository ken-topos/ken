# `NonEmpty` — lists with a structural head

Availability: **partial**

Authority: **derived reference**

Canonical source: [checked package](../../../../../catalog/packages/Data/Collections/NonEmpty.ken.md).

| Fact class | Disposition | Result and grounding |
|---|---|---|
| Subject | `generated` | `catalog/packages/Data/Collections/NonEmpty.ken.md` — “`NonEmpty` — lists with a structural head,” from the [subject index](../../subjects.md). |
| Declaration/type | `authored` | Declares a head-plus-tail `NonEmpty` carrier, total head and tail projections, list conversion, mapping, append, and its `Semigroup` dictionary. |
| Law | `authored` | A checked three-value structural proof lifts list-append associativity to `nonempty_append` and inhabits the semigroup law. |
| Effect/capability | `none-declared` | The canonical checked fences declare no effect row, `proc`, `visits`, or capability value for this package. |
| Assurance | `authored` | The package reports zero `trusted_base()` delta: the carrier is strictly positive, operations are structural, and the sole class law is a checked proof term. |
| Platform | `held` | No checked per-package facet is available; owner: catalog campaign. See the [held-class disclosure](../../card-format.md#held-class-disclosure). |
| Maturity | `held` | No checked per-package facet is available; owner: catalog campaign. See the [held-class disclosure](../../card-format.md#held-class-disclosure). |
| Dependency | `held` | No complete package-level checked projection exists; owner: `crates/`. See the [held-class disclosure](../../card-format.md#held-class-disclosure). |
| Reverse dependency | `held` | No complete projection and maintained inversion exist; owner: `crates/`. See the [held-class disclosure](../../card-format.md#held-class-disclosure). |
