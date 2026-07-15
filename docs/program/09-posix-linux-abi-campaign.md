# Linux ABI (direct-to-kernel) — campaign charter and work program

> **★ REFRAMED (operator, 2026-07-15): POSIX abstraction DROPPED; Ken targets
> the Linux syscall ABI directly.** The title/thesis was "POSIX / Linux ABI";
> the operator ruled that the Linux ABI alone provides the systems-adjacent
> surface Ken needs and sits directly against the kernel, so we **do not** build
> a POSIX portability layer — we bind Linux directly and take each other *nix
> (BSD, macOS) as its own per-target backend later. See §2 lead fixed input.
> Legacy "POSIX" naming below is being swept; the direction is Linux-direct.

**Owner:** Steward · **Status:** **PX1/PX2 RELEASING** (operator pulled ahead of
CC9, 2026-07-15) on the Runtime `crates` lane; PX3+ still sequenced behind CC9 ·
**Source:** `local/ken-posix-linux-interface-gap-report.md` (research report,
2026-07-12), **regrounded against `origin/main @ 26d5255e` on 2026-07-14**;
**reframed Linux-ABI-direct (POSIX dropped) 2026-07-15.**

This is the campaign charter. It is **not** a release order. Each `PX` work
package gets a shovel-ready frame (`docs/program/wp/px<N>-<slug>.md`) authored
at release time, per `agent/playbooks/federation/steward.md §2c`.

---

## 0. Read this first — the report is STALE, and its headline RISK came true

The report was written against `3a5cd323`. **Seventy-five commits have landed
since**, including the entire `I-*` (interface) and `CC*` (catalog) arc. Two of
its three load-bearing "current state" claims are **now false**:

| Report claim | Verdict | Proof |
|---|---|---|
| "Ken has **one** real host operation: capability-gated whole-file read" | **STALE** | **16 driven host ops**: 10 FS (`eval.rs:2179-2189`), 4 Console (`eval.rs:1847-1857`), 2 Clock (`eval.rs:2045-2048`) |
| "`Cap` = authority level + effect name; no rights bitset, no resource identity, no scope" | **STALE** | `Cap { authority, effect, scope: FsScope }` — `capabilities.rs:199-203`; `RightSet` `:58-86`; `FsHandle(OwnedFd)` `:97-100`; `FsIdentity{device,inode}` `:129-132`; `lineage` `:136-142` (ADR-0017) |
| "No realized packages in `catalog/packages/`" | **STALE** | `Capability/Filesystem/Path/Posix.ken.md` (1758 lines), `Capability/Process/Exit`, `Process/{Arguments,Environment,WorkingDirectory}`, `Capability/Time/WallClock`, `Capability/{FS,Console}` |
| "`write_bytes`/`append`/`send`/`recv` are undriven placeholders" | **STALE** | Retired at `6088e0b8`; FS write/append are **driven**. Net + Rand are not merely undriven — they are **not declared at all** |
| FFI marshalling is `BytesPtr` + debug-formatted scalar; no foreign call executed | **VERIFIED** | `foreign.rs:36-40`, `:54-58`; zero `dlopen`/`dlsym`/`libloading` hits repo-wide |
| Native: `ClosedNullary` only; `HostEffectExecution` unavailable; Cranelift rejects `RuntimeExpr::Effect` | **VERIFIED** | `executable_entrypoint_packaging.rs:85-93`, `platform_runtime_support.rs:327-331`, `cranelift_backend.rs:1438-1441` |
| No `USize`/`ISize`/`CInt`/`Ptr` | **VERIFIED** | zero hits across `crates/`, `spec/`, `catalog/` |

**The report's own anchors have moved. Do not pin them.** Every anchor in *this*
document was re-verified against `26d5255e`; treat even these as **perishable**
and re-ground at frame time.

### ★★ The finding that reorders the report's own phasing

The report's §16 lists *"ABI facts drift silently"* as a **risk**. It is not a
risk. **It has already happened, in the security-enforcement path, and it is on
`main` right now.**

**The inventory, corrected** — the Architect enumerated it after I under-counted
it by a factor of 3.6 (see §0a):

| What | Where | Count |
|---|---|---|
| `*_KEN` flag constants (`O_RDONLY` … `AT_REMOVEDIR`), **all hand-asserted** | `eval.rs:2355-2375` | **11** |
| `unsafe extern "C"` — `openat`, `mkdirat`, `unlinkat`, `renameat`, `readlinkat` | `eval.rs:2378-2394` | **5 fns** |
| `unsafe extern "C" { fn signal }` + `SIGPIPE = 13` + `SIG_IGN = 1`, **nested inside `mask_sigpipe()`** | `eval.rs:3714-3730` | **1 fn, 2 facts** |

**⇒ 6 FFI declarations. 13 hand-asserted ABI facts. All `#[cfg(unix)]`-gated.**

Read what that actually says:

1. **Six raw syscall boundaries are hand-declared `unsafe extern "C"` INLINE in
   the 4,600-line pure evaluator** — and one of them hides *inside a function
   body*. There is no boundary crate. **Only `ken-kernel` forbids `unsafe`**
   (`ken-kernel/src/lib.rs:42` — the sole hit in the repository). `ken-interp`
   does not.
2. **Thirteen target-ABI facts are asserted from memory.** No probe, no manifest,
   no target binding. Nothing in the tree can check them.
3. **The `cfg` gate is `unix`. The values are `linux`.** `#[cfg(unix)]` compiles
   on macOS and every BSD. These numbers are not the same there. The code
   compiles, links, and passes those bits to real syscalls.
4. **`O_NOFOLLOW` is the enforcement mechanism for `SymlinkPolicy::NoFollow`**
   (`capabilities.rs:89-92`) — an **ADR-0017 security property**. If that bit is
   wrong on a target, the symlink-escape defense **silently does not apply**, and
   every gate stays green, because the gates test *behavior on this box*.

> **⇒ A capability-confinement guarantee rests on a magic number that nobody
> probed, gated by a `cfg` broader than the fact it encodes.** Whether the values
> happen to be correct on today's target is **not the point** — the point is that
> **the artifact cannot tell you, and has nowhere to state the obligation.**

This is the **contract-expressibility failure** (`PRINCIPLES #14`) in its purest
form, and we have seen its exact shape twice this month: CC6b's segment
invariant living in a `--` comment, and LET-1's *readability* having no gate.
**Same disease: an obligation the artifact has no way to carry, invisible in a
green diff.** The difference is that those two were caught because someone tried
to *write the guarantee down* and found the pen had nowhere to land.

**Nobody has tried here yet. PX0, PX1 and PX2 are that attempt.**

### §0a — and the first draft of this charter made the same mistake

**I wrote "three constants, five FFI declarations." It is eleven and six.** In a
document whose entire thesis is *"the artifact cannot state its own contract,"*
**I under-counted the very inventory I was indicting.** The Architect found it by
**enumerating**; I had **sampled**. The mechanism, exactly:

- I read the file from **`:2370`**, because a prior audit cited `AT_REMOVEDIR` at
  **`:2375`**. **The inventory begins at `:2355`.** I picked my window from *a
  citation of one instance* rather than from *the extent of the kind*. **A line
  number tells you where something IS; it does not tell you where its KIND begins
  and ends.**
- I grepped `unsafe extern`, got **two** hits, and **read one**. The second is
  **indented, inside a function body**, and does not look like a boundary.

> **⇒ A grep SELECTS candidates. It never DECIDES, and it never COUNTS.**
> **Enumerate, count, and state the count.** Every PX frame's handoff must say
> ***"there are N; here are all N"*** — with N a number the author counted, never
> one inherited from a citation.

---

## 1. What this campaign IS and IS NOT

**IS:** hosted user-space Linux programs — CLI tools, file utilities, services,
protocol implementations — built on a **small, audited, manifest-bound host
boundary**, with the OS surface above it as **ordinary kernel-checked Ken**.

**IS NOT:** bare-metal, drivers, MMIO, interrupt context, or a general
imperative core. **ADR-0012 already ruled this out and it is not reopened.** The
report agrees (§1). Pursuing it would replace Ken's model with a second,
Rust-like one.

**IS NOT:** an untyped `syscall6` escape hatch. If a raw tier ever exists it is
explicitly audited and **never** the standard interface (report §16).

**The kernel does not grow.** OS operations, ABI manifests, handles, and
capabilities are **data, ordinary package types, and audited runtime
primitives**. They justify **no new trusted typing rules**. Any WP that reaches
for one is a **scope fork → escalate to the Architect**, not a judgment call.

---

## 2. Fixed inputs — SETTLED, do not reopen

- **★ LINUX ABI IS THE TARGET — POSIX abstraction DROPPED (operator,
  2026-07-15).** The Linux syscall ABI is the one Unix interface with a *hard*
  userspace-stability guarantee, and it is sufficient for the whole
  systems-adjacent surface this campaign needs — file I/O, directories, sockets,
  `clone`/`execve`/wait, `mmap`/`mprotect`, `epoll`, pipes, signals, time.
  `rustix`'s `linux_raw` backend binds it **directly against the kernel — no
  libc, no POSIX layer.** So **Ken targets the Linux ABI directly; there is NO
  POSIX portability abstraction in Ken.** Other Unixes (BSD, macOS) are each a
  **separate per-target backend** that binds *that* OS's own stable interface
  (which for the BSDs/macOS is their libc, POSIX-shaped) behind the **same
  `ken-host` seam — added when reached, deferred, Linux-first.** This DROPS all
  POSIX-lowest-common-denominator scope and directly validates §0's finding: the
  `cfg` gate must be `linux` and the ABI facts are Linux facts, probed (PX2).
  *Consequence to sweep:* the existing `Capability/Filesystem/Path/Posix.ken.md` name is now a
  misnomer — rescope it (catalog change; sequence separately, Steward-flagged).
  *Backend-seam decomposition is Architect-lane* (operator: "confirm the backend
  seam when I frame the work") — confirmed at PX1 framing (`evt_1t429wz5ehf42`).
- **ADR-0012** — verified total leaf components are a Ken target; general
  mutation-heavy driver code is not.
- **ADR-0011** — programs depend on lawful platform interfaces; the platform
  handler installs at the edge (now the **Linux-ABI** handler, per the lead
  fixed input — not a POSIX portability handler); no preprocessor as the platform
  abstraction.
- **ADR-0017** — the scoped capability model: `openat`-relative,
  **handle-not-path**, inode-keyed. **The resolve/operate split is correct and
  stays.** `HostHandler` has *no byte-path bypass that can re-resolve after
  authorization* (`eval.rs:2245-2248`). **Do not reintroduce string prechecking.**
- **PRINCIPLES #14** — never pin a shape that cannot state its own contract.
- **PRINCIPLES #15** — prefer a **fixed, audited trusted-base extension** over
  unbounded consumer `Axiom`s.
- **Successful OS execution is never promoted to kernel proof.** The status of
  every host guarantee is `tested` / `validated` / `delegated` — **never
  `proved`** — and that disclosure lives **in the source**, not only in the frame.
- **`rustix` ACCEPTED in the runtime trust boundary (operator, 2026-07-15) —
  SETTLED, NEVER RE-ASK.** FORK 2's dependency-risk acceptance is **granted**:
  exact-pinned, checksum-locked `rustix`, private behind the first-party
  `ken-host` shell; `ken-kernel` stays `forbid(unsafe_code)`. **This is a closed
  fixed input — it is never re-surfaced to the operator as a pending decision,
  not even bundled inside an adjacent question** (the operator granted it, then
  had to grant it three times because each session re-derived it as if live —
  2026-07-15). State it as a *fact* when framing; do not re-confirm it. **PX1/PX2
  are pulled ahead of CC9 (operator, 2026-07-15) — released now on the Runtime
  lane.**
- **NO linear/affine types in Ken — R2 CLOSED, not deferred (operator,
  2026-07-15).** Linear/affine types and further OTT extensions are **open
  research and stay OUT of the Ken language.** Resource-lifetime safety
  ("exactly-once release") is delegated OUTSIDE the language: **enforced
  operationally in the runtime** (generation-checked handle table; close lives at
  the runtime level, not in Ken types) and its **assurance obligation routed to
  Ward** through the assumption-boundary export (ADR-0006, G-Ward-seam). Ken
  reports the invariant runtime-enforced / `tested`, **never `proved`**, and
  never chases affinity to close the gap. This is the PRINCIPLES #9 Ken/Ward
  division applied to resources: *if a concern can't live in the static,
  propositional, total fragment, push it to the runtime and Ward, not into
  research-grade type theory.*

---

## 3. The forks — ✅ ALL THREE RULED

> ### ★★★ THE OPERATOR'S INTENT — the sentence every PX frame is
> measured against
>
> > **"The intention is to make Ken a software engineering language. Not a toy,
> > not a research project, but a tool for doing real practical work in the
> > safest way possible."** *(Pat, 2026-07-14)*
>
> **Read all three clauses; they constrain in different directions.** *"Real
> practical work"* forbids stopping at a demo. *"Not a research project"* forbids
> chasing affine types to get there. *"In the safest way possible"* forbids
> buying reach with a **silent** guarantee.
>
> **They pull against each other, and they do it at PX7.** The tie-break:
> **SHIP THE CAPABILITY · ENFORCE IT IN THE RUNTIME · SAY PLAINLY WHAT IS NOT
> PROVED — in the SOURCE.** Never a comment. Never a frame-only disclosure.
> **Never an unearned `proved`.** *"Safest possible" is not "safe"; it is the
> best available guarantee plus an honest statement of its edge. A language that
> overstates its safety is more dangerous than one that understates it.*

### FORK 1 — scope · ✅ **RULED: COMMIT THROUGH PX-E** *(operator, 2026-07-14)*

- **COMMITTED: PX-A → PX-E.** Audited boundary → **native effect execution** →
  descriptor streaming and resources → **processes and sockets** → **nonblocking
  and the event loop.**
- **STILL OUT OF SCOPE** *(named so nobody builds them)*: Linux control APIs
  (netlink, seccomp/Landlock, `io_uring`, cgroups, typed `ioctl`), the public C
  ABI and generated headers, a thread-safe runtime, affine/unique types, raw
  pointers/atomics/MMIO. **ADR-0012 stands. The kernel does not grow.**

**Campaign exit criterion:**

> **Ken builds and ships a real CLI tool AND a real network service as NATIVE
> BINARIES** — under scoped capabilities, over files larger than memory,
> observationally identical to their interpreter runs, with the exact target-ABI
> manifest hash bound into each artifact — **and the dangerous logic (binary
> parsers, path policy, protocol state machines, capability attenuation) is
> PROVED, not merely tested.**

**That last clause is not decoration.** **A verified `cat` is a toy.** A *proved*
path-confinement layer and a *proved* protocol state machine, sitting on a small
audited syscall boundary, **is the entire pitch** — and it is the only reading of
this campaign that serves *"the safest way possible"* rather than merely
*"reaches the OS."* **CC6b is the existence proof** that we can already do this
(a proved `path_parse ∘ path_render` round-trip with its reaching lemma, at zero
trusted-base delta). **PX-D/PX-E exist to give that capability something
dangerous to be pointed at.**

### FORK 3 — native early or late? · ✅ **RULED: NATIVE EARLY** *(operator)*

**PX-B runs BEFORE PX-C.** The weak argument for this was *porting 16 ops is
cheaper than porting 60*. **The real one:**

> **If the interpreter is the only implementation, it will teach us the wrong
> lessons — and we will not find out until the design is load-bearing.** The
> interpreter's `EvalStore` is `Rc`-based, single-threaded, and mutable.
> **PX7 designs resource handles.** Design them against the interpreter alone and
> we may design something that **cannot lower to native at all** — discovered
> *after* the semantics are public and the catalog depends on them.

**⇒ PX6's differential harness must exist BEFORE the things it is meant to
guard.** Build the guard, then build what it guards.

### FORK 2 — whose `unsafe`? · ✅ **RULED (Architect, `evt_7qqf827rr1jxk`)**

**RULING: (b) — an exact-pinned, checksum-locked `rustix`, private behind a
first-party `ken-host` policy shell. Ken-authored raw declarations are
retired.** Component design is the Architect's lane; this stands.

**I had leaned (a) — keep the declarations, probe the constants — and the
Architect's counter is better than my argument:**

> **A probe checks NUMBERS. It does not check SIGNATURES.** It cannot validate a
> handwritten function signature, the calling convention, pointer/length
> coupling, the errno convention, per-target `cfg`, or ownership transfer. **So
> (a)'s "our manifest is the source of truth" buys the CHEAP half of the ABI
> surface while retaining ALL of the `unsafe`** — and the 13-fact/6-declaration
> defect in §0 is the existence proof that handwritten ABI facts widen silently.
> **Keeping the declarations maximizes precisely the audit surface Phase A exists
> to remove.**

**And ADR-0009 already settles the shape of the question — *curate a mature
component before constructing one*. I should have cited it myself.** (c) is
strictly dominated: it takes on a dependency **and** keeps our raw-call
`unsafe`, buying neither the small boundary of (b) nor the single-source
ownership claimed by (a).

**The required shape (binding on PX1/PX2):**

- **`ken-host` is the ONLY public callable boundary; `rustix` is a private
  implementation detail.** Expose owned/borrowed handle types and validated path
  components — **no `RawFd`, no raw pointer, no integer flag, no unrooted path
  operation** escapes it. This is exactly ADR-0017's handle-not-path boundary.
- **Exact-pin the crate and every enabled feature; checksum-lock it.** Enable
  only the APIs PX1 actually uses. **Record** the selected backend, the
  transitive closure, licenses, and the **exercised upstream `unsafe` surface**
  in `docs/program/dependency-deltas.md`. *The third-party code is in the runtime
  trust boundary — but as a named, pinned, re-audited dependency, not invisible
  trust.*
- **PX2 SURVIVES.** The manifest records `TargetAbi`, the selected binding
  backend, exact dependency/version/checksum/features, the **complete** ABI-fact
  inventory, a schema version, and an output hash. **A system-header probe is an
  INDEPENDENT CROSS-CHECK, not a replacement set of handwritten constants** — and
  **any disagreement fails the build closed.** Bind the manifest hash into both
  interpreter and native artifacts.
- **★ V1 support is `linux`, NOT `unix`.** Every other Unix gets an **explicit
  unavailable backend** until it has its own manifested implementation. **Do not
  let a broad `cfg(unix)` imply a contract we never established.** *That widened
  gate was the bug.*

**✅ DEPENDENCY-RISK ACCEPTED (operator, 2026-07-14).** `rustix` goes inside the
**runtime** trust boundary. **`ken-kernel` is untouched and keeps
`forbid(unsafe_code)`** — the logical TCB does not move. **PX1 is unblocked.**

> **What we declined was never "no `unsafe`."** It was **6 hand-written FFI
> declarations and 13 unprobed ABI facts** — the status quo, and the thing that
> just failed (PX0). *There was no zero-risk option on this fork; there was only
> a choice of whose competence to trust.*

**⛔ PX1 PRECONDITION — enumerate the closure BEFORE you add the line to
`Cargo.toml`.** The **complete transitive dependency closure**, every license,
the exact feature set, and the **exercised upstream `unsafe` surface** go into
`docs/program/dependency-deltas.md` **as a deliverable, not a footnote.** *The
Steward has not counted it and will not assert it — that is precisely the error
PX0 exists to correct.* **If the closure is larger than the boundary it replaces,
that is a finding: say so and escalate.**

### FORK 3 — does native go early or late? *(operator)*

**The interpreter/native gap is WIDENING and gets more expensive every WP.**
Since `3a5cd323` the interpreter gained **16 host ops**; native gained **zero**.
Every OS capability we add interpreter-first increases the size of the eventual
native port.

- **Native early (PX-B before PX-C)** — *my recommendation.* "A real tool" means
  a **binary**. Porting 16 ops is cheap; porting 60 is not.
- **Native late** — acceptable only if the interpreter is the accepted delivery
  vehicle for the foreseeable future. **If so, say it out loud**, because it
  changes what "a real CLI tool" means.

---

## 4. The work packages

`PX` = POSIX/Linux. **Kernel team: nothing in this campaign.** By design (§1).

### Phase PX-A — make the boundary that ALREADY EXISTS honest · **P0**

*Pure debt. No new surface. This is the enabler for everything downstream, and
it is the cheapest thing in the campaign.*

| ID | Objective | Owner | Size |
|---|---|---|---|
| **PX0** | ⚡ **ERRATUM, RELEASED AHEAD OF THE CAMPAIGN** — *the host ABI is Linux, not `unix`.* Re-gate all **11** constants and both `unsafe extern "C"` blocks from `#[cfg(unix)]` to `#[cfg(target_os = "linux")]`; every non-Linux target returns a **named unavailable lane BEFORE any host call.** No value changes, no crate moves. Frame: `wp/px0-target-classification-erratum.md`. | **Runtime** | **S** |
| **PX1** | **`ken-host`: the first-party policy shell over an exact-pinned `rustix`.** Retire all 6 hand-declared FFI boundaries + call sites (`eval.rs:2378-2394`, `:2414/:2426/:2435/:2924/:2942/:2972/:2997`, `:3714-3730`). **`ken-host` is the only public callable boundary; `rustix` is private.** No `RawFd`, raw pointer, integer flag, or unrooted path escapes it. **Then `#![forbid(unsafe_code)]` on `ken-interp`.** **Gated on Pat's dependency-risk acceptance (§3, FORK 2).** | Runtime | **M** |
| **PX2** | **Target ABI identity + a generated manifest, with the probe as an INDEPENDENT CROSS-CHECK.** `TargetAbi` + selected backend + exact dep/version/checksum/features + **the complete 13-fact ABI inventory** + schema version + output hash. A system-header probe **cross-checks** `rustix`'s facts; **any disagreement fails the build CLOSED.** Manifest hash binds into interpreter **and** native artifacts. **All 11 `*_KEN` constants and both `SIG*` facts are DELETED.** | Runtime | **M** |
| **PX3** | **Machine/ABI scalar types in Ken** — `USize`/`ISize` and the `CInt` family, **bound to the manifest**, with **explicit, partial** conversions to/from arbitrary-precision `Int`. A narrowing conversion is a `Result`, never a silent truncation. | Language | **S** |

**★ PX2 carries a clean-room gate.** A build probe that `#include`s the system
headers and **prints values** learns a *fact from a build*; it does **not copy
GPL'd source into the tree**. That distinction is load-bearing and it is
**not mine to assert** — **PX2's frame routes through the Spec enclave's leakage
recheck before a line is written.** Report §4.2 agrees; `CLEAN-ROOM.md` decides.

> ### ★ Phase A exit — CORRECTED (Architect, `evt_7qqf827rr1jxk`)
>
> **My first draft said "No `unsafe` outside `ken-host`." That is FALSE on
> current `main` and it is a DANGEROUS acceptance criterion.**
> `cranelift_backend.rs:1059` holds the JIT entry `mem::transmute` — **a
> native-code-execution boundary, not a POSIX host-call boundary.** A
> literal-minded implementer could "satisfy" my AC by **moving the JIT trampoline
> into `ken-host`**, which would be actively wrong: it would merge two unrelated
> trust boundaries to make a checkbox go green.
>
> **Corrected exit:** ***No OS/host-ABI `unsafe` outside `ken-host`, and
> `ken-interp` forbids `unsafe`.*** Every ABI fact is manifest-bound and
> cross-checked; the artifact reports its exact target identity and manifest hash;
> a wrong-target manifest **fails closed before any syscall runs**.
>
> **If we ever want repository-wide `unsafe` confinement, the JIT trampoline gets
> its OWN named boundary and its OWN WP. Do not launder it into `ken-host`.**
>
> *An AC that can be satisfied by doing the wrong thing is worse than no AC — it
> **directs** the wrong thing, with authority.*

### Phase PX-B — native effect execution · **P0**

| ID | Objective | Owner | Size |
|---|---|---|---|
| **PX4** | **Native entrypoint ABI beyond `ClosedNullary`** (`executable_entrypoint_packaging.rs:85-93`): raw argv, environment, process exit status, runtime init/teardown, stdout/stderr/trap reporting. | Runtime | **M** |
| **PX5** | **Lower `RuntimeExpr::Effect` natively** (`cranelift_backend.rs:1438-1441`) to a call into a **versioned `ken-host` shim**: validate op support → check the carried capability → marshal per the manifest → call → map the response → resume exactly once. **Unsupported ops stay stable *unavailable lanes*. NEVER a no-op, never a generic scalar call.** | Runtime | **L** |
| **PX6** | **Interpreter/native differential harness for effects.** Compares **external deltas**, not return values: stdout/stderr, file and directory deltas, error identity, effect trace, exit status. | **Verify** | **M** |

**★ PX6 is Verify's lane, and it is deliberate.** Verify has been idle by design
since Z3/FO/Kripke were deferred. **This needs none of them** — it is
differential-observation discipline, which is exactly their competence. It also
guards the report's §16 risk *"native effects disappear or reorder silently."*

**Phase B exit:** ★ **The Ken CLI tool from Milestone C runs as a native
executable**, under scoped capabilities, observationally identical to its
interpreter run.

### Phase PX-C — descriptor streaming, resources, structured errors · **P1**

**★ Today every FS op is whole-file.** `fs_read_at` is `read_to_end`
(`eval.rs:2765-2767`); `fs_write_at` is `set_len(0)` + `write_all` + `sync_all`
(`eval.rs:2789-2795`). **`cp` on a 4 GB file interns 4 GB into the content
store.** There is no `open`, no `close`, no seek, no partial IO — and therefore,
usefully, **no use-after-close bug is currently possible.** PX7 is the WP that
*introduces* that hazard, and it must pay for it.

| ID | Objective | Owner | Size |
|---|---|---|---|
| **PX7** | **Ken-visible resource handles + `System.Resource` bracket.** Opaque, **generation-checked** handle table; `open`/`close`; **double-close and use-after-close FAIL VISIBLE** (stale generation ⇒ `Closed`, never a recycled fd); scoped `withResource` closes on success, error, **and trap**. | Runtime + Foundation | **L** |
| **PX8** | **Partial/positioned IO + `System.Buffer`.** `read`/`write` return **progress**, not all-or-nothing — a short write is **success with progress**, not an error. `writeAll` is a **derived Ken loop, proved**. Bounded mutable buffer floor. | Runtime + Foundation | **L** |
| **PX9** | **`System.Error` — structured errno.** Every error kind retains **operation + handle/path context**. (Today: 10 `io::ErrorKind` mappings, `eval.rs:3933-3942`.) | Foundation | **M** |

> ### ★★ PX7 is a CONTRACT-EXPRESSIBILITY WP and its frame must say so
>
> **"Exactly-once release" has nowhere in Ken to live.** Ken has no affine or
> linear types. The runtime can *enforce* the invariant with generation checks;
> the **language cannot state it**, and no test will ever show you the gap —
> because tests exercise **values**, and the hole is in the **type surface**.
>
> **The (b‴) audit is MANDATORY on PX7's frame**, and the honesty statement —
> ***"exactly-once release is `tested`, enforced by the runtime handle table; it
> is NOT `proved`, and Ken cannot currently express it"*** — goes **in the
> `System/Resource` SOURCE**, not only in the frame. *(CC6b: the disclosure that
> `path_normalize` is lexical-not-canonical had to live in the source. A frame is
> read once; the source is read forever.)*
>
> Affine/unique resources are the permanent fix. **They are research, they are
> out of scope, and PX7 must not smuggle them in.**

**Phase C exit:** *`cat`, `cp`, and `wc` run natively over a file larger than
memory, under scoped capabilities, matching the interpreter's external deltas.*

### Phase PX-D — processes and sockets · ✅ **COMMITTED**

- **PX10** — spawn/exec/wait, pipes, **deny-by-default fd inheritance**, `pidfd`
  where available. **Prefer `posix_spawn` semantics; raw `fork` is a restricted
  raw tier if it exists at all.**
- **PX11** — sockets + typed address families; **an injected resolver
  capability** (DNS is **not** a syscall — its trust source must be visible).

### Phase PX-E — nonblocking and the event loop · ✅ **COMMITTED**

- **PX12** — nonblocking descriptors, `epoll`/`eventfd`/`timerfd`/`signalfd`,
  cancellation and timeout contracts in the **operation type**, not in prose.

**Phase D/E exit:** *a Ken **network service** and a **process supervisor** run
as native binaries, single-threaded and event-driven, with **no unsafe pointers
in application Ken** — and their **protocol state machines and path policy are
PROVED**, per the campaign exit criterion.*

---

## 4a. ★★ The honesty gap the operator's own ruling creates — read before PX7

**"Through PX-E" + "the safest way possible" collide at exactly one place, and I
want it visible now rather than discovered at PX7.**

Sockets, epoll handles, timers, pipes, child processes, mapped regions — **PX-D
and PX-E are a resource explosion.** And **Ken cannot state "exactly-once
release."** It has no affine or linear types. The runtime can *enforce* the
invariant with a generation-checked handle table; **the type system cannot
express it**, and **no test will ever show you the gap** — because tests exercise
**values**, and the hole is in the **type surface**.

**Today this is harmless: there is no `open`/`close` at all** (every FS op is
whole-file), **so no use-after-close is possible.** **PX7 introduces the hazard.
PX-D and PX-E multiply it.**

| | |
|---|---|
| **The rule (from the operator's tie-break)** | Ship it. Enforce it in the runtime. Report it **`tested`, never `proved`**. Put the disclosure **in the `System/Resource` SOURCE**, not only in the frame. *(A frame is read once. The source is read forever.)* |
| **⛔ The forbidden move** | Building affinity into Ken. **R2 (linear/affine types) and further OTT extensions are open research; the operator ruled them OUT of the language (2026-07-15). Do not chase them to close this gap.** |
| **✅ The resolution (operator, 2026-07-15)** | R2 is **CLOSED, not deferred** — no linear/affine types in Ken, ever. Push the safety machinery **out of the language**: (1) the **runtime** enforces exactly-once release operationally via the generation-checked handle table — close lives at the runtime level, not in Ken types; (2) the *assurance obligation* is **routed to Ward** through the assumption-boundary export (ADR-0006, G-Ward-seam), where resource-lifetime discipline is monitored behaviorally. Ken reports the invariant **runtime-enforced / `tested`, never `proved`**, and the disclosure lives in the `System/Resource` source. |

**PX7's frame does NOT re-open R2.** The Steward's booked obligation is now to
**wire the resource-lifetime assurance into the Ward seam** — not to weigh affine
types. *The one thing that would betray the operator's intent is to ship a
resource API whose safety we cannot state and then not say so — and the Ward
export is exactly where we state it.*

### Explicitly OUT OF SCOPE

netlink · seccomp/Landlock/namespaces · `io_uring` · cgroups · typed `ioctl`
families · the public C ABI and generated headers · a thread-safe runtime ·
affine/unique types · raw pointers/atomics/MMIO.

**Named so that nobody builds them.** Each is a separate campaign, and each needs
a fork resolved before it is one.

---

## 5. The capability gaps that survive the regrounding

ADR-0017 landed most of what the report asked for. **Two gaps are real:**

- **Runtime revocation membrane.** `RevocationHandle { revoked: bool }`
  (`capabilities.rs:468-471`) is a **static contract**; its own doc comment says
  so (`:464-467`). Real OS resources need shared delegation identity, transitive
  child invalidation, close-on-revoke policy, and defined in-flight semantics.
  **Fold into PX7** (it is the same handle-lifetime machinery) **or split out if
  PX7 grows.**
- **IFC at the OS boundary** — authority and information-flow are independent
  axes; holding permission to *write* a socket must not imply permission to
  *send secrets through it*. **This is `Sec1`, an existing workstream. Note the
  dependency; do NOT duplicate it here.**

---

## 6. Sequencing

**★ PX0 is OUT OF BAND and already released.** It is an erratum on landed code,
not campaign work: it does not wait behind CC9, and it does not need the campaign
forks resolved. It quarantines the wrong-target path *now*; PX1/PX2 delete the
code it fixes.

**★ NATIVE EARLY (operator ruling).** PX-B (`PX4`→`PX5`→`PX6`) lands **before**
PX-C (`PX7`→`PX8`→`PX9`). **The differential harness exists before the resources
it has to guard** — so no resource design can become load-bearing on
interpreter-only assumptions.

```mermaid
flowchart LR
    PX0[PX0 erratum - Linux not unix - MERGING]
    LET4[LET-4 multi-binding let] --> LET2b[LET-2b refresh guides]
    LET2b --> LET3[LET-3 catalog let pilot]
    LET3 --> CC9[CC9 Resource/Bracket + Tooling.Testing.Property]
    CC9 --> PX1[PX1 ken-host over pinned rustix]
    PX0 --> PX1
    PX1 --> PX2[PX2 target ABI + manifest + probe cross-check]
    PX2 --> PX3[PX3 USize/ISize/CInt]
    PX2 --> PX4[PX4 native entrypoint ABI]
    PX4 --> PX5[PX5 native effect lowering]
    PX5 --> PX6[PX6 interp/native differential harness]
    PX6 --> PX7[PX7 resource handles + bracket - safety in runtime plus Ward, no affine]
    PX7 --> PX8[PX8 partial IO + buffers]
    PX8 --> PX9[PX9 structured errno]
    PX9 --> PX10[PX10 processes - spawn/wait/pipes]
    PX9 --> PX11[PX11 sockets + resolver capability]
    PX10 --> PX12[PX12 nonblocking + epoll event loop]
    PX11 --> PX12
```

### ★ Milestone-D CLI remainder FOLDS INTO THIS CAMPAIGN (operator, 2026-07-15)

**"Roll Milestone D into the Linux-ABI effort — it was an overreach for the CLI
work." (Pat, 2026-07-15.)** The Milestone-C CLI program **exits at CC9**; its
leftover runtime capabilities are **not** framed as standalone CLI WPs
(the erstwhile "I-9…I-12"). They are delivered *here*, as ordinary campaign
scope, sequenced by the DAG above:

| Milestone-D remainder | Home in this campaign |
|---|---|
| subprocess (spawn/exec/wait, pipes) | **PX10** |
| signals | **PX12** (`signalfd`) / **PX10** (child signalling) |
| temp files | **PX-C** (descriptor/FS: PX7/PX8) |
| terminal / tty | **slot TBD** — small add to **PX9** (Console/`System.Error`) or its own late PX; placed when that phase is reached |

**⇒ CC9 is the CLI-program exit criterion.** No separate CLI-milestone WPs for
these four; the campaign's native-first PX-D/PX-E track subsumes them.

### ⚠ What this campaign does NOT deliver — and *"not a toy"* implies it

**The POSIX campaign buys REACH — can Ken talk to the operating system.** A
*software engineering language* also needs a **TOOLCHAIN**, and that is a
**separate axis this campaign does not touch.** Verified against `main`:

| | Status |
|---|---|
| Diagnostics | ✅ landed (CC4 `Capability.Diagnostics.Core`) |
| Modules / namespaces | ⚠ **partial** — ADR-0014/15/16 landed, but **`export` is SPECIFIED AND NOT PARSED** (zero hits in `parser.rs`) |
| Test / property framework | ⛔ **nothing** — no `catalog/packages/Tooling/Testing`; CC9 is unbuilt. **Temporary scaffolding only (operator, 2026-07-15) — see below.** |
| Package manager (L4) | ⛔ not started |

**⇒ Flagged for the operator, not assumed.** *"Not a toy" is not purchased by
syscalls alone — a language you cannot write a test in is not a tool for real
practical work.* **The Steward will bring a toolchain-axis proposal separately;
it is not smuggled into PX.**

**★★ The in-Ken test framework is TEMPORARY SCAFFOLDING until Ward takes over
(operator, 2026-07-15).** Testing is behavioral/empirical assurance —
PRINCIPLES #9 puts that in **Ward's** domain, not Ken's core. So CC9's
`Tooling.Testing.Property` is a **stopgap**: it exists so real work can be written and
checked *now* (a language you can't test in isn't a tool), but it is **explicitly
provisional and destined for Ward takeover.** Design consequence, binding on the
CC9 frame: **minimize language-surface entrenchment** — no test-only primitives
baked into the kernel or surface grammar that would make the Ward handoff a
breaking migration; keep it a catalog package over existing capabilities, so
Ward can assume the role behind the same seam (ADR-0006, one-way Ken→Ward export)
without a language change. Same principle as resource-lifetime safety: **the
obligation is Ward's; Ken carries only the minimal, non-entrenching stopgap.**

**★ CC9 is a real dependency, not a courtesy** — but a *scaffolding* one. CC9 is
framed as `Resource`/`Bracket` + `Tooling.Testing.Property`. **PX7 is the `Bracket`
consumer** — if CC9 lands a `Bracket` shape that PX7 cannot use for OS handles,
we will build it twice; **CC9's frame must be re-read against PX7 before CC9 is
released** (on me). The `Tooling.Testing.Property` half is the temporary-scaffolding piece
above; the `Resource`/`Bracket` half is durable runtime-safety machinery.

**PX1 and PX2 are the only ones that could start early.** They are pure debt,
they touch no surface, and the defect they close is **on `main` in the security
path today**. If Pat wants them pulled ahead of CC9, they are ready to frame.

---

## 7. Do-not-reopen guardrails (binding on every PX frame)

- **⛔ The kernel does not grow.** No new trusted typing rule. A WP that needs
  one is a **scope fork → Architect**, not an implementer's judgment call.
- **⛔ No untyped `syscall6` escape hatch** as the standard interface.
- **⛔ Do not reintroduce string-precheck path authorization.** ADR-0017's
  handle-pinned, `openat`-relative resolve/operate split is **settled**.
- **⛔ No ABI fact without a probe.** After PX2, a hand-written constant in the
  boundary is a **defect**, not a shortcut. *This rule exists because we already
  did it three times.*
- **⛔ Never promote successful OS execution to kernel proof.** `tested` /
  `validated` / `delegated` — and **the disclosure lives in the source.**
- **⛔ A conditional law ships with its REACHING LEMMA, proved.** (CC6b. A
  vacuous law has **zero trust delta** — the gate's trigger never fires on the
  hollow WP.)
- **⛔ `catalog/` · `examples/` · `conformance/` ⇒ FULL CI**, whatever the file
  extension. Never `--doc-only`.
- **⛔ Build/test TARGETED ONLY** — `scripts/ken-cargo -p <crate>`. **Never
  `--workspace`** (`COORDINATION.md §12`, operator hard rule). CI owns
  workspace-green.
