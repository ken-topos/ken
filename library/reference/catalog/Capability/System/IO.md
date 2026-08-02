# System.IO

Availability: **partial**

Authority: **derived reference**

Canonical source: [checked package](../../../../../catalog/packages/Capability/System/IO.ken.md).

| Fact class | Disposition | Result and grounding |
|---|---|---|
| Subject | `generated` | `catalog/packages/Capability/System/IO.ken.md` — “System.IO,” from the [subject index](../../subjects.md). |
| Declaration/type | `authored` | Declares five theorems about the transparent `writeAll` loop: its call bound, exact-prefix step, complete success, first-error preservation, and all-success result. |
| Law | `authored` | All five named theorems have checked proof terms in the canonical fence, covering termination, exact-prefix preservation, success completeness, and error behavior. |
| Effect/capability | `authored` | The exact-prefix theorem consumes constructor-private `BufferSpan` and `TransferCount` values; the checked fence exposes no producer for those boundary carriers. |
| Assurance | `authored` | The five proofs are ordinary kernel-checked terms and not axioms or runtime claims; exactly-once settlement and liveness remain explicitly delegated to the runtime boundary. |
| Platform | `held` | No checked per-package facet is available; owner: catalog campaign. See the [held-class disclosure](../../card-format.md#held-class-disclosure). |
| Maturity | `held` | No checked per-package facet is available; owner: catalog campaign. See the [held-class disclosure](../../card-format.md#held-class-disclosure). |
| Dependency | `held` | No complete package-level checked projection exists; owner: `crates/`. See the [held-class disclosure](../../card-format.md#held-class-disclosure). |
| Reverse dependency | `held` | No complete projection and maintained inversion exist; owner: `crates/`. See the [held-class disclosure](../../card-format.md#held-class-disclosure). |
