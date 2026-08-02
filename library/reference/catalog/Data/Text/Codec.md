# `Data.Text.Codec` — safe UTF-8 and ASCII views

Availability: **partial**

Authority: **derived reference**

Canonical source: [checked package](../../../../../catalog/packages/Data/Text/Codec.ken.md).

| Fact class | Disposition | Result and grounding |
|---|---|---|
| Subject | `generated` | `catalog/packages/Data/Text/Codec.ken.md` — “`Data.Text.Codec` — safe UTF-8 and ASCII views,” from the [subject index](../../subjects.md). |
| Declaration/type | `authored` | Declares safe UTF-8 decoding, byte-level ASCII classification, and an optional indexed ASCII view that preserves absent-byte results. |
| Law | `authored` | Checked proofs expose the decode definition, preserve absent and present `ascii_view` cases, and carry the existing one-way `BytesRoundTripLaw` without strengthening it. |
| Effect/capability | `none-declared` | The canonical checked fences declare no effect row, `proc`, `visits`, or capability value for this package. |
| Assurance | `authored` | Trust delta is zero: every operation is ordinary Ken over landed total byte operations, and the checked fences add no `Axiom`, primitive, postulate, or opaque declaration. |
| Platform | `held` | No checked per-package facet is available; owner: catalog campaign. See the [held-class disclosure](../../card-format.md#held-class-disclosure). |
| Maturity | `held` | No checked per-package facet is available; owner: catalog campaign. See the [held-class disclosure](../../card-format.md#held-class-disclosure). |
| Dependency | `held` | No complete package-level checked projection exists; owner: `crates/`. See the [held-class disclosure](../../card-format.md#held-class-disclosure). |
| Reverse dependency | `held` | No complete projection and maintained inversion exist; owner: `crates/`. See the [held-class disclosure](../../card-format.md#held-class-disclosure). |
