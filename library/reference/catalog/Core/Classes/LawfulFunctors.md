# `lawful-functors` — `Semigroup`, `Monoid`, `Functor`, `Foldable`

Availability: **partial**

Authority: **derived reference**

Canonical source: [checked package](../../../../../catalog/packages/Core/Classes/LawfulFunctors.ken.md).

| Fact class | Disposition | Result and grounding |
|---|---|---|
| Subject | `generated` | `catalog/packages/Core/Classes/LawfulFunctors.ken.md` — “`lawful-functors` — `Semigroup`, `Monoid`, `Functor`, `Foldable`,” from the [subject index](../../subjects.md). |
| Declaration/type | `authored` | Declares the four named classes, their operations and coherence fields, and checked `List`, `Bool`, and `Option` instances with their supporting functions. |
| Law | `authored` | Associativity, unit, functor identity/fusion, and fold coherence are class fields with checked witnesses: structural induction for `List`, finite cases for `Bool`, and case splitting or reduction for `Option`. |
| Effect/capability | `none-declared` | The canonical checked fences declare no effect row, `proc`, `visits`, or capability value for this package. |
| Assurance | `authored` | Every instance has zero trusted-base delta. The law fields are kernel-checked with induction, case splitting, `Proved`, `Refl`, and `cong`; none is postulated. |
| Platform | `held` | No checked per-package facet is available; owner: catalog campaign. See the [held-class disclosure](../../card-format.md#held-class-disclosure). |
| Maturity | `held` | No checked per-package facet is available; owner: catalog campaign. See the [held-class disclosure](../../card-format.md#held-class-disclosure). |
| Dependency | `held` | No complete package-level checked projection exists; owner: `crates/`. See the [held-class disclosure](../../card-format.md#held-class-disclosure). |
| Reverse dependency | `held` | No complete projection and maintained inversion exist; owner: `crates/`. See the [held-class disclosure](../../card-format.md#held-class-disclosure). |
