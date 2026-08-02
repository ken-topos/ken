# `Ord Nat` — a lawful total order on `Nat`, and its operations

Availability: **partial**

Authority: **derived reference**

Canonical source: [checked package](../../../../../catalog/packages/Data/Numeric/Nat/Order.ken.md).

| Fact class | Disposition | Result and grounding |
|---|---|---|
| Subject | `generated` | `catalog/packages/Data/Numeric/Nat/Order.ken.md` — “`Ord Nat` — a lawful total order on `Nat`, and its operations,” from the [subject index](../../subjects.md). |
| Declaration/type | `authored` | Declares structural `Nat` ordering, an `Ord Nat` dictionary, three-way `OrdResult`, and `min`, `max`, `sub`, and `compare`. |
| Law | `authored` | Checked structural proofs establish reflexivity, antisymmetry, transitivity, and totality; the dictionary fields are inhabited by those proofs. |
| Effect/capability | `none-declared` | The canonical checked fences declare no effect row, `proc`, `visits`, or capability value for this package. |
| Assurance | `authored` | The package reports zero `trusted_base()` delta: every order law is kernel checked and the entry introduces no `Axiom`, primitive, or postulate. |
| Platform | `held` | No checked per-package facet is available; owner: catalog campaign. See the [held-class disclosure](../../card-format.md#held-class-disclosure). |
| Maturity | `held` | No checked per-package facet is available; owner: catalog campaign. See the [held-class disclosure](../../card-format.md#held-class-disclosure). |
| Dependency | `held` | No complete package-level checked projection exists; owner: `crates/`. See the [held-class disclosure](../../card-format.md#held-class-disclosure). |
| Reverse dependency | `held` | No complete projection and maintained inversion exist; owner: `crates/`. See the [held-class disclosure](../../card-format.md#held-class-disclosure). |
