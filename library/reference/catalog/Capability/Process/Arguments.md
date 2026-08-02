# Capability.Process.Arguments

Availability: **partial**

Authority: **derived reference**

Canonical source: [checked package](../../../../../catalog/packages/Capability/Process/Arguments.ken.md).

| Fact class | Disposition | Result and grounding |
|---|---|---|
| Subject | `generated` | `catalog/packages/Capability/Process/Arguments.ken.md` — “Capability.Process.Arguments,” from the [subject index](../../subjects.md). |
| Declaration/type | `authored` | Declares raw-byte argv projection and replacement, positional lookup, byte lookup, structural bounds comparison, and checked `ArgLocation` construction. |
| Law | `authored` | The checked `round_trip` proof shows that projecting arguments after replacement returns the replacement list. |
| Effect/capability | `none-declared` | The canonical checked fences declare no effect row, `proc`, `visits`, or capability value for this package. |
| Assurance | `authored` | All declarations are transparent checked terms over landed `ProcessInput`, `List`, `Bytes`, and `ArgLocation`; there is no primitive, postulate, opaque constant, or `Axiom`, and the `trusted_base()` delta is zero. |
| Platform | `held` | No checked per-package facet is available; owner: catalog campaign. See the [held-class disclosure](../../card-format.md#held-class-disclosure). |
| Maturity | `held` | No checked per-package facet is available; owner: catalog campaign. See the [held-class disclosure](../../card-format.md#held-class-disclosure). |
| Dependency | `held` | No complete package-level checked projection exists; owner: `crates/`. See the [held-class disclosure](../../card-format.md#held-class-disclosure). |
| Reverse dependency | `held` | No complete projection and maintained inversion exist; owner: `crates/`. See the [held-class disclosure](../../card-format.md#held-class-disclosure). |
