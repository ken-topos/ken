# `Empty` and `Dec` — computational falsity and decidability

Availability: **partial**

Authority: **derived reference**

Canonical source: [checked package](../../../../../catalog/packages/Core/Logic/EmptyDec.ken.md).

| Fact class | Disposition | Result and grounding |
|---|---|---|
| Subject | `generated` | `catalog/packages/Core/Logic/EmptyDec.ken.md` — “`Empty` and `Dec` — computational falsity and decidability,” from the [subject index](../../subjects.md). |
| Declaration/type | `authored` | The public API is `Empty`, `absurd_empty`, `Dec`, `Yes`, `No`, `decide`, `yes`, `no`, and `dec_eq_decides`. The standard `Empty`/`Dec` display is illustrative; the package-authored wrappers and bridge are checked. |
| Law | `authored` | `yes_is_true` and `no_is_false` check `decide`'s two computation facts; the local `DecEq` record carries its soundness and completeness contract. |
| Effect/capability | `none-declared` | The canonical checked fences declare no effect row, `proc`, `visits`, or capability value for this package. |
| Assurance | `authored` | Standard inductives and package functions add no postulate or primitive. `dec_eq_decides` preserves the trust posture of the supplied `DecEq` instance; the checked `Bool` example has zero added trust. |
| Platform | `held` | No checked per-package facet is available; owner: catalog campaign. See the [held-class disclosure](../../card-format.md#held-class-disclosure). |
| Maturity | `held` | No checked per-package facet is available; owner: catalog campaign. See the [held-class disclosure](../../card-format.md#held-class-disclosure). |
| Dependency | `held` | No complete package-level checked projection exists; owner: `crates/`. See the [held-class disclosure](../../card-format.md#held-class-disclosure). |
| Reverse dependency | `held` | No complete projection and maintained inversion exist; owner: `crates/`. See the [held-class disclosure](../../card-format.md#held-class-disclosure). |
