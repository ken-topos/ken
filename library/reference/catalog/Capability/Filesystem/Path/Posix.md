# `Capability.Filesystem.Path.Posix` — byte-preserving lexical paths

Availability: **partial**

Authority: **derived reference**

Canonical source: [checked package](../../../../../../catalog/packages/Capability/Filesystem/Path/Posix.ken.md).

| Fact class | Disposition | Result and grounding |
|---|---|---|
| Subject | `generated` | `catalog/packages/Capability/Filesystem/Path/Posix.ken.md` — “`Capability.Filesystem.Path.Posix` — byte-preserving lexical paths,” from the [subject index](../../../subjects.md). |
| Declaration/type | `authored` | Declares structured raw-byte paths, parsing and rendering, joining and parents, validity checks, and lexical normalization without decoding through `String`. |
| Law | `authored` | The canonical checked fences prove parse/render closure for valid paths, validity preservation, normalization, idempotence, and removal of dot segments and absolute dot-dot segments. |
| Effect/capability | `none-declared` | The canonical checked fences declare no effect row, `proc`, `visits`, or capability value for this package. |
| Assurance | `authored` | Every operation is transparent over `List UInt8` and existing lawful equality; the package declares no primitive, postulate, opaque constant, or `Axiom`, so its `trusted_base()` delta is zero. |
| Platform | `held` | No checked per-package facet is available; owner: catalog campaign. See the [held-class disclosure](../../../card-format.md#held-class-disclosure). |
| Maturity | `held` | No checked per-package facet is available; owner: catalog campaign. See the [held-class disclosure](../../../card-format.md#held-class-disclosure). |
| Dependency | `held` | No complete package-level checked projection exists; owner: `crates/`. See the [held-class disclosure](../../../card-format.md#held-class-disclosure). |
| Reverse dependency | `held` | No complete projection and maintained inversion exist; owner: `crates/`. See the [held-class disclosure](../../../card-format.md#held-class-disclosure). |
