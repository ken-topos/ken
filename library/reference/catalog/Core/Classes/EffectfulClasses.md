# `Applicative`, `Monad`, and `Traversable` — effectful constructor classes

Availability: **partial**

Authority: **derived reference**

Canonical source: [checked package](../../../../../catalog/packages/Core/Classes/EffectfulClasses.ken.md).

| Fact class | Disposition | Result and grounding |
|---|---|---|
| Subject | `generated` | `catalog/packages/Core/Classes/EffectfulClasses.ken.md` — “`Applicative`, `Monad`, and `Traversable` — effectful constructor classes,” from the [subject index](../../subjects.md). |
| Declaration/type | `authored` | Declares the `Applicative`, `Monad`, and `Traversable` classes; concrete `Option` and `List` instances; an `Identity` support instance; and the checked helper functions used by their dictionaries. |
| Law | `authored` | Class fields state the applicative and monad laws; checked theorems prove the `Option` and `List` instances. The traversal section proves identity, naturality, and composition, including the composed-applicative support laws. |
| Effect/capability | `authored` | `Traversable.traverse` is declared `proc`: its abstract result constructor is fail-closed as potentially effectful. The checked `List` and `Option` implementations are ordinary `fn` values using an explicit `Applicative` dictionary. No capability value is declared. |
| Assurance | `authored` | The package states zero trusted-base delta for its law fields: proofs are kernel-checked, with finite `Option` cases and structural `List` induction. Its validation evidence checks the trust posture, discriminating failures, and checked fences. |
| Platform | `held` | No checked per-package facet is available; owner: catalog campaign. See the [held-class disclosure](../../card-format.md#held-class-disclosure). |
| Maturity | `held` | No checked per-package facet is available; owner: catalog campaign. See the [held-class disclosure](../../card-format.md#held-class-disclosure). |
| Dependency | `held` | No complete package-level checked projection exists; owner: `crates/`. See the [held-class disclosure](../../card-format.md#held-class-disclosure). |
| Reverse dependency | `held` | No complete projection and maintained inversion exist; owner: `crates/`. See the [held-class disclosure](../../card-format.md#held-class-disclosure). |
