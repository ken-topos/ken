---
scope: enclave
audience: (see scope README)
source: private memory `systems-os-kernel-interface-first-party`
---

# Systems / OS-kernel interface is first-party standard-package content

Operator ("Pat") strategic direction, 2026-07-01. Extends package ecosystem
comprehensive standard small contrib: the OS/systems layer is the biggest, most
concrete category of "comprehensive standard packages." Goal: Ken is **the most
convenient** option for systems work, self-sufficient against the kernel.

**What's first-party (standard-package content):** per-OS syscall numbers,
`errno`, signal/ioctl/socket/file-flag constants, ABI struct layouts
(`stat`/`sockaddr`/`timespec`/…), kernel-adjacent protocols — for Linux, macOS,
Windows. So a systems program builds in Ken with **no FFI to libc, no
third-party bindings crate**.

**Design shape (settle when the *systems track* is framed):**
- **Three tiers** ("protocols" spans the stack) — (1) raw per-platform bindings
  (`linux`/`macos`/`windows`, arch-specific; kernel-adjacent netlink/socket);
  (2) portable systems std (files/sockets/processes) on (1); (3) **application
  comm protocols** on (2)'s sockets — **REST/HTTP, WebSockets, GraphQL** +
  growing set (operator-confirmed 2026-07-01 "protocols reaches at least this
  far"). Tier (3) is where 3rd-party crates proliferate hardest → strongest
  supply-chain case.
- **Trust class differs BY tier** — tier (1) raw bindings = FFI/syscall
  boundary, `declare_primitive`/FFI-tagged **trusted audited primitives** (ES1
  item-2, largest item-2 pop). BUT tiers (2)-(3) — portable std + app protocols
  (HTTP framing, WS state machines, GraphQL parse/execute) = **derived logic on
  the sockets primitive**, ordinary (largely provable) standard packages, NOT
  item-2. So "protocols" is not one trust class: kernel ABI trusted-audited, app
  protocols derived.
- **Clean-room ABI sourcing** — constants/struct-layouts are interface *facts*
  but kernel headers are GPL `uapi`; generate from the authoritative ABI /
  permissive source, DO NOT copy headers (CLEAN-ROOM.md).
- **Conformance against the REAL ABI** — a wrong offset/syscall# is memory
  corruption, not a type error → differential conformance vs the actual OS ABI,
  a distinct discipline from derived-package law-proofs.
- **Couples to the native backend** (X3a/runtime, spec `38-ffi-io`) — sitting
  against the kernel bottoms out in the runtime exposing syscalls; likely a
  DISTINCT track (not a mere ES4 category) given size + backend coupling +
  clean-room care. ES4's portable std sits atop it.

**Captured in** `docs/program/everyday-surface-program.md` (*Systems / OS-kernel
interface is first-party*). **"Protocols" scope RESOLVED** (operator
2026-07-01): spans kernel-adjacent (netlink/socket) THROUGH application-level
(REST/HTTP, WebSockets, GraphQL + growing set). **Remaining Q when framing the
systems track:** sequencing vs the native backend (raw tier couples to
X3a/runtime; app protocols are derived and could ship earlier on the sockets
std).
