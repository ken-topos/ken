# ADR 0012 — Ken for systems programming: no intrinsic barrier; the leaf-components target and arena-as-capability

- **Status:** Accepted (orientation — defensive design, no near-term work
  scheduled). Ratified by the Architect 2026-07-09, ground-checked against landed
  code. The ratification covers the **Ken-side machinery + arena claims**; the
  survey and external comparisons (HACL\*/EverParse/Rust-for-Linux/klint) are
  illustrative, **out of sign-off** (as with ADR 0011). The ratification confirms
  the ADR *honestly characterizes* the crown-jewel effect-row context-law
  enforcement and the D1 productivity checker as **undemonstrated/aspirational
  research** — it does *not* vouch those proofs are achievable; it affirms only
  that **nothing in the design forecloses the leaf target** ("no intrinsic
  barrier" is sound). The value of this ADR is **defensive confirmation plus a
  set of design orientations that inform choices elsewhere** — it is explicitly
  *not* a systems-programming roadmap.
- **Date:** 2026-07-09
- **Deciders:** the operator (a defensive-design question dispatched via the
  Steward); Architect ratification of the characterization of Ken's machinery and
  the runtime arena.
- **Related:** ADR 0011 (platform-dependent code) — this ADR extends 0011's
  F\*/HACL\* thread to the *extreme* target (freestanding kernel code); ADR 0009
  (capability-supply strategy) — the `trusted_base()` discipline invoked here.

## Context

This ADR answers a **defensive-design** question, not a scheduled work item:
*what would it take to make Ken the best — or at least a very good — choice for
(Linux) kernel programming?* The concern is foundational validation: is there an
**intrinsic** reason Ken's design could not, with adaptation, serve some aspects
of systems programming? The finding, recorded so the reasoning survives: **there
is no intrinsic barrier.** Ken is systems-*adjacent*, not a systems language, and
the honest target is a specific niche — but nothing in the fundamental design
forecloses it.

The value is forward-looking. Even with no kernel work planned, the analysis —
the memory-model-as-capability discipline, fixed-width/manifest ABI, the
uniqueness/linearity question, coinduction/productivity — bears directly on
**non-systems** decisions Ken will face (performance, serialization, long-running
services, resource bounds). Capturing it here informs those choices.

## The two ambitions (they have very different price tags)

"Kernel programming" splits into two targets that must not be conflated:

1. **A verified *leaf component* inside a kernel module** — a pure, total,
   provable piece (a parser of attacker-controlled bytes, a crypto primitive, a
   checksum, a codec, a table-driven policy/verifier engine) compiled and linked
   as a module. This is the HACL\*/EverParse niche: HACL\*'s formally-verified
   Curve25519 already ships *in* the Linux kernel (via WireGuard); EverParse's
   verified parsers run in the Windows kernel. Kernel bugs cluster exactly here —
   in memory-unsafe parsing of untrusted input. **This is the achievable "very
   good choice" answer**, and it plays to dependent types' strengths.
2. **A general driver / systems language** (a Rust competitor for
   mutation-heavy, hardware-poking code). This requires Ken to grow into a
   systems language it fundamentally is not; done straight it risks being "a
   slower Rust." **This is the "best choice" answer and it is enormous.**

Ken will never be the best choice for target 2's imperative core — the
honest-boundary principle (`PRINCIPLES §7`) says to state that plainly. Target 1
is real, valuable, and defensible.

## What kernel programming actually demands

Nine constraints, roughly ordered by how hard they cut against Ken-as-it-is:
(1) freestanding execution, ~zero runtime; (2) no GC, deterministic
deallocation, in-place mutation; (3) raw pointers, volatile MMIO, atomics,
memory ordering, precise layout; (4) concurrency, interrupts, and context rules
("hold a spinlock ⇒ must not sleep", RCU read-side sections); (5) *deliberate*
non-termination (idle loop, scheduler); (6) zero-cost abstraction; (7) interop
with a giant C kernel ABI; (8) tiny stacks (8–16 KB), no unwinding; (9)
auditable unsafe.

## Where Ken is already differentially strong

The reasons to bother — places Ken can do what C and Rust structurally cannot:

- **Effect rows = provable context/locking rules.** "Runs in atomic context /
  must not sleep / is in an RCU read-side section" is an effect-row constraint; a
  handler boundary is the context transition. Rust-for-Linux gropes at this with
  `klint` — it can *discipline* but not *prove* it. Ken could carry it as a law.
  This is the **crown-jewel research bet** — the one thing that would make Ken
  *categorically* better than Rust-for-Linux at what Rust-for-Linux strains
  hardest to do. (Today: aspirational. The mechanism — effect rows + the
  interaction-tree model, ADR 0011 claim 2 — exists; proving a temporal context
  law over a handler is undemonstrated, the same open work ADR 0011 flags for
  effectful laws.)
- **Dependent types = provable worst-case bounds** — no overflow, no OOB,
  provable stack/time bounds. Constraint 8 (tiny stacks) turns from a hazard into
  a theorem.
- **`trusted_base()` = auditable unsafe, enumerated and spec-carrying** —
  strictly stronger than Rust `unsafe` (a keyword, not an audited manifest);
  constraint 9, already solved by design (ADR 0009).
- **Lawful classes = a driver proving it honors a device contract** (proved
  laws, not signatures — ADR 0011 claim 1).
- **Manifest ABI + fixed-width types** (ADR 0011) — constraint 3's layout half,
  already the chosen discipline.

## The four adaptation axes (A–D), with the operator's corrections folded

The gap analysis, corrected against what Ken actually has today:

- **A — Freestanding codegen. Retarget, not build.** Ken **has a compiler that
  produces executables** (Architect-confirmed first-hand against the NC19–NC27
  runtime line — executable-artifact-contract, entrypoint-packaging,
  native-execution-differential, object-linker, Cranelift lowering) — so this is
  not "interpreted → invent a native backend" (tectonic) but "hosted executable →
  freestanding module" (engineering). Named delta: freestanding flags (no red
  zone, `-mcmodel=kernel`,
  FPU-guarded SSE); no libc calls in emitted code (the pure allocation-into-arena
  leaf subset already mostly satisfies this); the arena re-backed by kernel
  memory (see B); C-linkage exported symbols; kernel stack discipline. That the
  compiler already emits working executables also retires most of constraint 6
  (erasure/monomorphization/codegen are already solved to ship a binary). The
  pragmatic road is ADR 0011's own state-of-the-art: extract a low-level subset
  to C (F\*/Low\*→KaRaMeL), reusing the kernel's C toolchain — literally how
  HACL\* entered Linux. A verified native backend (CakeML/CompCert-style) is the
  aspirational road.
- **B — Memory model: an arena, not GC — and it is nearly kernel-shaped
  already.** See the grounded analysis of `crates/ken-runtime/src/store.rs`
  below. The short form: bump-allocate + whole-arena reset (no GC/refcount),
  per-space regions already implemented, bounded loud exhaustion — and the *only*
  thing between it and a kernel backing is a single localized allocation seam.
- **C — Split it; the leaf target needs only the cheap half.** (C1) *in-place
  mutation for speed within pure semantics* → uniqueness/linearity (Clean, Idris
  2, Koka's Perceus/FBIP), **not** raw pointers; a linear/unique scratch-buffer
  capability, provably unique, composes with the arena. A *core-language*
  exploration that doubles as a general performance lever. (C2) *genuine
  low-level access* (volatile MMIO, atomics + memory ordering, physical pointers)
  → **thin, effect-typed, spec-carrying trusted primitives** enumerated in
  `trusted_base()` (the ADR 0011 posture); relaxed-atomics semantics are a
  research minefield one cannot cheaply prove *through*, so model the interface,
  tag the effect, put the semantics in the trusted spec, prove above the barrier.
  A pure parser touches **no** C2.
- **D — Productive non-termination: the seed is already owned.** Ken's
  interaction trees *are* the coinductive object (potentially-infinite by
  construction; effect handling is productive corecursion). Missing: (D1) a
  surface for guarded corecursion with a **productivity checker** — the dual of
  the landed SCT (SCT proves recursion *terminates*; productivity proves
  corecursion *produces*); (D2) compiling a guarded step-loop to an actual
  `while(1){ step }` rather than an ever-growing structure — with a compiler
  extant (A), a concrete codegen question. Leaf components *terminate by
  definition*, so D is a tier-2 need.

## The tier boundary is a clean fault line

The cheap/expensive split of *every* axis falls on the same seam — the leaf
target needs only cheap halves; the expensive halves are all tier-2:

| Axis | Cheap half (leaf target needs) | Expensive half (tier-2 systems) |
|---|---|---|
| A | retarget codegen freestanding, re-back arena | verified native backend |
| B | today's arena + per-space regions (both landed) | static region *types* (compile-time escape checking) |
| C | C1 uniqueness for in-place scratch | C2 raw/volatile/atomic primitives + memory model |
| D | *nothing* (leaves terminate) | productivity checker + loop codegen |

So "systems-adjacent, not systems" is a **structural** claim, not a judgment
call — and the operator's two corrections (compiler exists; arena, not GC) retire
the expensive half of A and B for the leaf target outright.

## Grounded: the runtime arena (`crates/ken-runtime/src/store.rs`)

The single most informative check for the leaf-component thesis was "measure the
minimal runtime and whether the arena backing is a swappable capability." Reading
the content-addressed value store (spec `40-runtime/44-capacity.md §1–3`)
answers it:

- **Pure bump + whole-arena reset, no GC.** The arena is a chain of fixed 4 MiB
  pages (`store.rs:43-45`, `PAGE_SIZE:28`); `append` extends the current page
  (`:77-96`); reclamation is `reset() → pages.clear()` (`:104-107`), O(pages) not
  O(values); slot ids monotonic, never reused across reset (`:33-37`, spec
  `41 §3b`). No refcount, no tracing, no drop-based value reclamation — and
  "no GC" is not merely a present fact but **conformance-pinned**
  (`auto_gc_not_present`, `store.rs:737`), so the property is guarded against
  regression. This *is* the bump-then-reset discipline that fits the leaf pattern
  (bounded input → bounded output → reset), and it never GC-pauses — which the
  kernel cannot tolerate.
- **Per-space regions already exist.** Each `Space` owns a *separate* arena +
  index partition and is reclaimed independently (`:191-199`, spec `44 §3`).
  Nested/scoped regions were placed in B's *expensive* half; the structural half
  is already built. Only static region *types* (compile-time "no live pointer
  escapes a short-lived arena") remain, and only for tier-2.
- **Bounded, loud exhaustion — no OOM-panic/unwind.** A soft `capacity_limit`
  yields a typed `CapacityExhausted { limit }` (`:201-202`, `:251-253`, spec
  `44 §2`/OQ-5). The failure mode is a value, not a trap — directly serving
  constraint 8.
- **The one seam.** Page memory is `Vec::<u8>::with_capacity(PAGE_SIZE)` at a
  single site (`Page::new`, `:48-52`) — hardcoded to Rust's global allocator; the
  open-addressing index is a second `Vec`-backed site (`buckets: Vec<Bucket>`,
  `:142`/`:149`, resizing at 0.70 load `:24-25`). Re-backing on a kernel
  allocator (kmalloc/vmalloc) means abstracting those two points behind a
  **page-allocator capability** — **precisely the ADR 0011 "Platform capability"
  move, applied to memory.** The arena's *structure* is already kernel-shaped;
  only the *backing* is a parameter to lift. (Ledger item: the slot-id counter is
  a process-wide relaxed `AtomicU64`, `:33` — fine in-kernel, a global to note.)

**Honest caveats.** (1) This is specifically the *interned/content-addressed*
value store — it dedups by hash and carries a growing, resizing index, more
machinery (and allocation behavior) than a bare byte-arena a minimal leaf
component might want; interning is a feature (structural sharing) but a
per-component characteristic to weigh. (2) Only the store was read, not the
compiler's full emitted-runtime surface — the *complete* "minimal required
runtime" question (does emitted code pull anything beyond this store + a trap
path?) is the remaining half of the probe, a measurement on compiler output.

## Decision — the design orientation

1. **The target, if ever pursued, is the verified *leaf component*** — a
   first-order, allocation-bounded Ken subset extracted to C and linked behind a
   `trusted_base()` boundary (HACL\*/EverParse-shaped). This needs only road A(i)
   and the *cheap* half of each axis. It is the "very good choice" that Ken's
   dependent types actually earn.
2. **No intrinsic barrier exists.** The fundamental design accommodates the need
   with machinery Ken already has or that lifts cleanly: a retargetable compiler
   (A), a bump/reset arena with per-space regions one seam from a kernel backing
   (B), effect-rows + `trusted_base()` for low-level access (C2), interaction
   trees for productive loops (D), and lawful capability interfaces for
   implementation selection (ADR 0011).
3. **The differentiator, and the crown-jewel research bet, is effect-row
   context-law enforcement** — provable "atomic context / no-sleep / RCU"
   discipline. Without it, tier 2 is "a slower Rust"; with it, Ken offers *proof*
   where Rust offers *discipline*. It is undemonstrated and is the honest research
   frontier (the effectful-law target ADR 0011 also names).
4. **TCB honesty (as ADR 0011).** Platform/hardware primitives are irreducibly
   trusted; make the interface small and spec-carrying, prove everything above
   it, and grow `trusted_base()` — never the kernel — visibly and audited. This
   is F\*/HACL\*'s posture and fits `PRINCIPLES §5` better than any mainstream
   language, because the interface above the boundary is *lawful*, not merely
   typed.

## Consequences

- **No near-term action.** This is defensive confirmation. Nothing is scheduled.
- **The analysis informs non-systems choices** (the operator's stated value):
  the **arena-as-capability** discipline (parameterize the page backing) is good
  design for *any* embedding, not just kernels; fixed-width/manifest ABI (ADR
  0011) governs all serialization; the **C1 uniqueness/linearity** question is a
  general performance lever; **D productivity/coinduction** governs any
  long-running service or stream. These decisions arise well outside kernel work.
- **A concrete, low-cost lift on record:** should a freestanding target ever be
  wanted, the arena is a single page-allocator-capability abstraction away — not
  a redesign. Worth remembering when the runtime is next touched.
- **A guardrail:** resist letting "systems ambitions" pull Ken toward growing an
  imperative mutation core as a *primary* mechanism (the target-2 trap). The leaf
  niche is the disciplined shape; C2's low-level primitives stay thin and
  trusted-base-bounded, never a general escape hatch into the pure core.

## Alternatives considered

- **Declaring Ken a general systems language (target 2)** — rejected as a design
  goal: it demands B's region types + C1 + C2 + D + full C-ABI interop, a
  multi-year build that risks "a slower Rust" and contradicts the honest
  boundary. Available as a *finding*, not a program.
- **Declaring kernel/systems work simply out of scope** — rejected: the value is
  the defensive confirmation that the design already accommodates it, precisely
  because it is *not* near-term, and the orientations feed non-systems choices.
- **A native verified backend now (road A(ii))** — deferred: the pragmatic
  extract-to-C road (A(i)) reaches the leaf niche without it; the verified
  backend is a research program to activate only if a broader compilation
  commitment is made (itself an open decision, cf. ADR 0011's per-backend
  specifier note).

## Specifics to pin before any build

- The **remaining half of the runtime probe**: measure the *full* minimal
  runtime the compiler's output depends on (beyond the value store + a trap
  path).
- Whether the **page backing** is lifted to a capability generically or only at a
  freestanding target (the seam is identified; the abstraction is not yet built).
- Whether Ken ever commits to **multiple compilation backends** — the open
  decision (shared with ADR 0011) that would activate both the per-backend
  specifier model and road A(ii).
- The **effect-row context-law** proof obligation — the same undemonstrated
  effectful-law target ADR 0011 flags, here in its hardest form.
