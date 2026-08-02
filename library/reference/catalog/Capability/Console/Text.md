# `Console` — ordinary text-output helpers

Availability: **partial**

Authority: **derived reference**

Canonical source: [checked package](../../../../../catalog/packages/Capability/Console/Text.ken.md).

| Fact class | Disposition | Result and grounding |
|---|---|---|
| Subject | `generated` | `catalog/packages/Capability/Console/Text.ken.md` — “`Console` — ordinary text-output helpers,” from the [subject index](../../subjects.md). |
| Declaration/type | `authored` | Declares four text-output procedures: `print`, `printLine`, `eprint`, and `eprintLine`; they encode UTF-8, select stdout or stderr, and make newline policy explicit. |
| Law | `none-declared` | The canonical checked fence declares no `law`, `proof`, or `theorem` for this package. |
| Effect/capability | `authored` | All four checked procedures return `IO (Result IOError Unit)` and declare `visits [Console]`; failures such as broken pipes remain named result values. |
| Assurance | `authored` | The helpers are ordinary kernel-checked definitions over the byte-exact Console ABI and add zero `trusted_base()` entries. |
| Platform | `held` | No checked per-package facet is available; owner: catalog campaign. See the [held-class disclosure](../../card-format.md#held-class-disclosure). |
| Maturity | `held` | No checked per-package facet is available; owner: catalog campaign. See the [held-class disclosure](../../card-format.md#held-class-disclosure). |
| Dependency | `held` | No complete package-level checked projection exists; owner: `crates/`. See the [held-class disclosure](../../card-format.md#held-class-disclosure). |
| Reverse dependency | `held` | No complete projection and maintained inversion exist; owner: `crates/`. See the [held-class disclosure](../../card-format.md#held-class-disclosure). |
