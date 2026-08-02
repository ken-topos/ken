# Native target and ABI facts

> **Availability:** partial. **Authority:** derived reference.

Ken generates a target ABI manifest while building `ken-host`. This page is a
lookup for the fields that exist today and for the boundary at which generation
fails closed. It is not a supported-platform promise.

## Generated fields

The generated `TargetAbi` value contains:

| Field | Meaning |
|---|---|
| `schema_version` | Version of the generated manifest representation. |
| `target` | Cargo's target triple for this build. |
| `target_os` | Cargo's target operating-system value. |
| `backend` | The selected host backend; the available lane is `linux_raw`. |
| `dependencies` | Exact audited dependency versions, checksums, and enabled features. |
| `fact_count`, `facts` | Probed widths, Linux flags, syscall numbers, modes, and errno values. |
| `manifest_hash` | SHA-256 identity of the canonical manifest text. |

The runtime carries the same manifest hash into native artifacts and rejects a
mismatch before the artifact enters the host boundary.

## Availability by lane

| Build lane | Availability | Behaviour |
|---|---|---|
| Linux, target equals host | **Current.** | The build probes and verifies the Linux ABI facts, writes `target_abi.rs`, and selects `linux_raw`. |
| Linux cross-target | **Unavailable.** | The build fails closed because system headers may attest only their own target. It does not emit a cross-target inventory. |
| Non-Linux target | **Unavailable.** | Host effect ABI generation fails closed before a target manifest is produced. |
| Miri or the `rustix_use_libc` backend | **Unavailable.** | The audited host boundary requires the `linux_raw` backend and rejects these alternatives. |

## Boundary of the claim

The generated value records facts about one build. It does **not** define a
ratified target-support contract, enumerate future supported targets, or turn an
unavailable lane into planned support. For the current native-build command,
see [`ken native-build`](../toolchain/native-build.md).
