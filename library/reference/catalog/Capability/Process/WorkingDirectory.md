# Capability.Process.WorkingDirectory

Availability: **partial**

Authority: **derived reference**

Canonical source: [checked package](../../../../../catalog/packages/Capability/Process/WorkingDirectory.ken.md).

| Fact class | Disposition | Result and grounding |
|---|---|---|
| Subject | `generated` | `catalog/packages/Capability/Process/WorkingDirectory.ken.md` — “Capability.Process.WorkingDirectory,” from the [subject index](../../subjects.md). |
| Declaration/type | `authored` | Declares raw-byte working-directory projection and replacement over `ProcessInput`, preserving arguments and environment unchanged. |
| Law | `authored` | The checked `round_trip` proof shows that projecting the working directory after replacement returns the replacement bytes. |
| Effect/capability | `none-declared` | The canonical checked fence declares no effect row, `proc`, `visits`, or capability value for this package. |
| Assurance | `authored` | The declarations are transparent checked terms over landed `ProcessInput` and `Bytes`; the package adds no primitive, postulate, opaque constant, `Axiom`, or trusted-base entry. |
| Platform | `held` | No checked per-package facet is available; owner: catalog campaign. See the [held-class disclosure](../../card-format.md#held-class-disclosure). |
| Maturity | `held` | No checked per-package facet is available; owner: catalog campaign. See the [held-class disclosure](../../card-format.md#held-class-disclosure). |
| Dependency | `held` | No complete package-level checked projection exists; owner: `crates/`. See the [held-class disclosure](../../card-format.md#held-class-disclosure). |
| Reverse dependency | `held` | No complete projection and maintained inversion exist; owner: `crates/`. See the [held-class disclosure](../../card-format.md#held-class-disclosure). |
