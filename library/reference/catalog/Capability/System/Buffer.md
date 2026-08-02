# System.Buffer

Availability: **partial**

Authority: **derived reference**

Canonical source: [checked package](../../../../../catalog/packages/Capability/System/Buffer.ken.md).

| Fact class | Disposition | Result and grounding |
|---|---|---|
| Subject | `generated` | `catalog/packages/Capability/System/Buffer.ken.md` — “System.Buffer,” from the [subject index](../../subjects.md). |
| Declaration/type | `authored` | Declares a `BufferWindow` constructor, scalar and structural projections from `BufferSpan` and `TransferCount`, and proofs for positive, request-bounded transfer counts. |
| Law | `authored` | Checked theorems expose the positivity proposition and the structural equation that splits a request budget into transferred and remaining counts. |
| Effect/capability | `authored` | The canonical checked fence consumes constructor-private `BufferSpan` and `TransferCount` boundary values while exposing no pointer, mutable reference, or producer for either carrier. |
| Assurance | `authored` | Count and budget witnesses are kernel-checked Ken data; fixed capacity, current-window discipline, and settlement invalidation remain explicitly runtime-enforced rather than restated as proofs. |
| Platform | `held` | No checked per-package facet is available; owner: catalog campaign. See the [held-class disclosure](../../card-format.md#held-class-disclosure). |
| Maturity | `held` | No checked per-package facet is available; owner: catalog campaign. See the [held-class disclosure](../../card-format.md#held-class-disclosure). |
| Dependency | `held` | No complete package-level checked projection exists; owner: `crates/`. See the [held-class disclosure](../../card-format.md#held-class-disclosure). |
| Reverse dependency | `held` | No complete projection and maintained inversion exist; owner: `crates/`. See the [held-class disclosure](../../card-format.md#held-class-disclosure). |
