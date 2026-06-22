# Reality Check: the Analysis vs. the Source

**Status:** grounded against the `1.1-dev` source tree on 2026-06-21 by five
parallel source-audit passes (value representation, Spaces/wire/spawn,
type-system/kernel, surface/stdlib, heap/Leech/limits). Every verdict below
carries `file:line` evidence from those passes.

> **Role under the clean-room direction (see `02-strategy.md`).** The effort is
> now a *new, MIT-licensed language* — a clean-room reimplementation, not a fork.
> This document is therefore a **knowledge artifact about the reference
> prototype**, not implementation material. Its purpose is to record what the
> prototype actually does so a spec can be written from behavior. The `file:line`
> citations are analysis/commentary about AGPLv3 code; they must **not** be
> copied or closely paraphrased into the new MIT codebase. Carry the
> *understanding* forward; write the code from specs and tests. The AGPL source
> stays out of the implementation context.
>
> This file contains **no copied prototype source** — only `file:line` references
> and original analysis, which is why it is safe to keep in this public repo.

The companion analysis (`../yon/Yon Programming Language Analysis.md`) is a strong
*idea map* but was written almost entirely from `yon-lang.org`, the LLVM
Discourse post, and Hacker News — **not from this source tree**. It is therefore
factually unreliable about the current system. Two systematic errors run through
it:

1. It generalized narrow **ABI/wire conventions** (handles cross function and
   Space boundaries as `f64`) into a whole-language **ontology** ("every value
   is an f64").
2. It took the **public 1.0 docs** as ground truth, missing everything added
   since — the dependent kernel, the module/package system, the effect system,
   the 33-module stdlib, and the existing Z3/Coq backends.

Treat the analysis as a source of *design directions*, and this document as the
*facts*.

---

## 1. The headline corrections

### 1.1 There is no uniform-f64 representation (the analysis's central claim is REFUTED)

The analysis devotes two full exchanges to arguing that "f64-uniform
representation is the most consequential premature optimization in the
language." **It is not how the language works.**

The OCaml frontend emits **heterogeneously typed** MLIR from the start, and the
type converter maps each surface type to a conventional ABI type:

| Surface type | MLIR/LLVM type | Evidence |
|---|---|---|
| `number` / `money` | `f64` | `frontend/emit_mlir.ml:22-24` |
| `boolean` | `i1` | `emit_mlir.ml:11,23`; `LowerToposToStandard.cpp:105-106` |
| `proposition` (Ω, tri-valued) | `i8` | `emit_mlir.ml`; converter in `LowerToposToStandard.cpp:188-203` |
| `section` (place instance / handle) | `i64` = `(heap_id:u32<<32)\|slot` | `runtime/yon_rt.h:34-68`; `LowerToposToStandard.cpp:198-203` |
| `probe` (closure) | `struct{i32,i32}` | `LowerToposToStandard.cpp` |
| `heyt_int` | `struct{i64,i64}` | `LowerToposToStandard.cpp` |

Scalars are **unboxed SSA values** (`arith.addf`, `arith.cmpf`, `arith.andi`),
never heap traffic (`emit_mlir.ml:2357-2373`). There is no "uniform f64 stratum"
that a pass decodes away — the premise is backwards. Booleans use GPRs (i1),
handles use GPRs (i64); only floats use XMM. Raw pointers exist (`!llvm.ptr` in
builtin signatures, `void*` arena payloads — `emit_mlir.ml:287`,
`xleech2_heap.h:189-199`).

**The one grain of truth:** when a `section` *handle* crosses a function or Space
boundary it is shuttled as an `f64` via `sitofp` of its i64 slot index
(`emit_mlir.ml:1408-1435`), and library structures expose slot indices as
f64-typed handles. That is "handles travel as f64," a wire convention — *not*
"every value is an f64," and it is plain integer-in-double, **not** NaN-boxing.

**What is actually true and worth fixing:** there is no distinct fixed-width
`Int` or decimal type. The single surface numeric type is `number`, lowered to
`f64`, so integer-valued computation silently loses precision above 2^53
(`emit_mlir.ml:1449-1458,2357-2367`). This is a **missing-type** problem, narrow
and contained — not a representation-ontology problem. It is the *only* part of
the analysis's f64 argument that survives.

> Consequence for the program of work: the analysis's largest proposed
> workstream — "abandon f64, adopt typed handles, redesign the runtime, wire
> protocol, and heap" — **should not be undertaken.** It targets a problem that
> does not exist. Replace it with a small, contained work package: add `Int`
> (and optionally `Decimal`) as first-class types.

### 1.2 The cross-Space wire is not 4×f64; strings already cross (REFUTED)

The analysis treats "4 × f64 = 32-byte wire, strings can't cross Spaces" as a
fundamental ceiling blocking serialization, networking, and structured `promote`.
The implementation has **two** transports, and neither matches that description:

- **RPC mailbox** (`yon_rpc2_req_t`): up to **8 f64 args** plus nonce, seq,
  128-byte reply name, selector, argc — a ~190-byte struct, single-f64 reply
  (`runtime/yon_rt.h:728-735`).
- **DTO / wire stream**: serializes whole place instances — scalars, **strings**
  (`[u32 len][bytes]`), and **nested sections** — recursively *by value* into a
  **64 KB frame ring**, rebuilt in the consumer's own heap
  (`runtime/yon_rt.c:7360-7475`). `regression/cross_space/weather.yon` sends a
  `Reading` struct with a string `label`; `sub_weather.yon` reads
  `String.length(r.label)` on the far side.

The transport is POSIX **shared memory** (`shm_open` + `mmap(MAP_SHARED)`,
`PTHREAD_PROCESS_SHARED` mutex/cond — `yon_rt.c:1289,1818`), so "actor model with
no shared memory" is also false: isolation is logical (copy-on-receive), not
absence of shared pages.

**Real gaps here:** `LIST`/`MAP` wire tags are reserved but unimplemented
(`yon_rt.h:175`), so *collections* of structured data don't cross yet; and
`spawn`/`promote` plus the RPC reply are **scalar-only** (each `promote` moves
exactly one f64 — `yon_rt.c:2320-2332`). `spawn` is plain `fork()` (COW), not an
"isolated arena reclaimed via munmap." Crash-respawn on a virgin channel with
advanced epoch is real and correct (`yon_rt.c:~1985,~2640`).

### 1.3 The kernel is partway to L2, not pure L1 (PARTIAL → the analysis is materially out of date)

The analysis's L1-vs-L2 framing is correct **for the surface programmer** — you
cannot today attach a postcondition, refinement, or correctness proposition to
your own function; `proposition` is a *runtime* three-valued (Heyting/Ω)
computation decided by place visibility (`prop_eval.ml:1-26`), evaluated, never
proven. There is no tactic language and no refinement types on arrows
(`requires` is capability tokens only — `surface_ast.ml:444`).

But the **kernel** has moved a long way the analysis never saw:

- Genuine dependent **Pi** (`tycheck.ml:1492-1504`; `dispatcher.ml:368-372`).
- Endpoint-aware identity type **Id** with real compile-time path equality
  (`tycheck.ml:1162-1225`; `dispatcher.ml:360-367`).
- **J / path induction** eliminator (`test_j_tarski` green).
- A **computing cubical core**: transport (`cubical.ml:508-558`), comp
  (`:336-451`), hcomp (`:454-494`), Glue / univalence-as-computation
  (`:537-557`), HITs with reducing eliminators (`:651-684`), `isEquiv` with
  coherence checked on real terms (`test_isequiv` 11/11).
- **SCT-certified δ-conversion** (size-change termination gating definitional
  unfolding — `sct.ml`, `dispatcher.ml:229-300`); the ROADMAP §1.6 "δ-debt" is
  already closed on this branch.
- **Z3 *and* Coq backends** already wired (`naturality_smtcheck.ml`,
  `naturality_coqcheck.ml`, gated by `YON_F2D`/`YON_F3`) — but **only** for
  naturality of natural transformations with single-variable Real-arithmetic
  bodies.
- **Yoneda** mechanically verified by kernel oracles (`test_yoneda_typed`,
  `test_yoneda_lemma`) — ROADMAP §2's "no mechanical verification" describes the
  pre-1.1 state.

Oracle suite observed green this session: `test_glue 19/19`, `test_isequiv
11/11`, `test_path_core 15/15`, `test_hit_compute 5/5`, `test_yoneda_typed 5/5`,
`test_yoneda_lemma 11/11`, and others (all rc=0).

> Consequence: the L1→L2 task is **plumbing an already-dependent kernel to the
> surface plus generalizing existing prover backends**, not building a proof
> system from nothing. This is the single biggest reframing in this document and
> the foundation of the strategy.

### 1.4 Most "missing" features already exist (OUTDATED claims)

The analysis's "existential gaps" and "15 hard problems" are largely already
addressed in this fork:

**Already exist** (analysis wrong): module/import system (file import, qualified
import with alias, cross-package `from Space` RPC import — `module_prefix.ml`,
`regression/cross_space/`); package manager + build (`pkg/yon-pkg`, git-based,
`yon.toml`/`yon.lock` with pinned commits); **effect system** (`visits`,
statically checked, transitively inferred — `tycheck.ml:1682-1844,3607-3622`);
serialization (`yon_rt_serialize/deserialize` + Merkle); Hindley-Milner
polymorphism + dependent types; full tooling (formatter `yonfmt`, linter
`yon_lint`, doc-gen `yon_doc`, LSP `yon_lsp` 544 lines + nvim/vscode clients);
TCP sockets (`Wire.*_net`); structured error model (`error E subcontains Base`
subobject places + tri-valued logic + `is`/`is not` guards); law/property
verification (`law`/`verify` + MLIR `AlgebraVerifier`). The stdlib has **33
modules**, not the 20 in the public docs.

**Genuinely missing** (analysis correct): FFI (no C/BLAS — external symbols are a
fixed prefix-gated allowlist, `emit_mlir.ml:5676-5681`); `Bytes`/binary I/O
(File is text-only); `Result`/`Option`/`Either`; linear/affine types; delimited
continuations / coroutines; `Stream.flatMap`/`bind`; crypto beyond FNV-1a (no
HMAC/SHA/base64); HTTP/TLS; `match`/`case` + exhaustiveness; open user-defined
typeclasses; interactive debugger; runtime reflection.

**Partial / stubbed**: sum types (`A | B` *parses*, `variant` AST node exists,
but lowers to an opaque base type with no constructor/eliminator —
`desugar.ml:169-172`); `pushout` expression (lowers to `0.0` —
`desugar.ml:633-634`); `with multishot` (parses, reaches an MLIR attribute, but
the reducer ignores it — no real multi-resume); capabilities (`requires`)
(runtime FNV-1a gate only, static check not wired); effect handlers
(tail-resumptive in-place substitution, not reified continuations).

### 1.5 The heap and Leech facts (mostly CONFIRMED, with key clarifications)

- 196,560 slots/heap (= Λ₂₄ kissing number / type-2 vector count), 256
  heaps/chain (8-bit id, 24-bit slot), ~50.3M slot ceiling, 64 MB arena/heap,
  ~17–18 GB content/process — all confirmed (`xleech2_heap.h:43,45,227-242`;
  `BASELINE-1.0.md:14-15`).
- **Addressing is FNV-1a + memcmp, NOT lattice geometry** — the Leech quantizer
  is never on the allocation path (`xleech2_heap.c:34-47,325`). Slot ids are a
  monotonic counter. The lattice has **three distinct, code-separate roles**:
  Golay(24,12,8) error-correction (VoyagerList), the kissing number as a
  fixed-size bitmap/MPHF domain (XSet — 196,560 bits ≈ 3073 words), and Co₀/M24
  orbit canonicalization (`leech_orbits`, `yon_curtis_canon`).
- Auto-chaining + **global** dedup via a process-wide `(root,hash64)→HeapRef`
  index (`xleech2_heap.c:656-780`); **loud refusal** at limits (design
  principle, `BASELINE-1.0.md`).
- No automatic GC/compaction, but manual reclamation exists (`clear`/`reset`/
  `strip_trim` via `madvise(MADV_DONTNEED)`). Dedup means realistic burn is "one
  slot per *distinct* value," not per occurrence.
- **C-level introspection exists** (`yon_rt_heap_occupancy`, checkpoint
  snapshots) but **no surface primitive** exposes it to programs.
- `yon_xcoord_to_int24` is **fully implemented** (`xleech2_coord.c:50-93`) — the
  ROADMAP "stub returning -1" text is stale. The Co₀ orbit cardinalities
  (98280/8386560/8292375) are **not** present in source as constants — they are
  an unverified doc claim the ROADMAP itself flags for confirmation.

---

## 2. Corrected capability map

| Capability | Analysis said | Actually (1.1-dev) |
|---|---|---|
| Value representation | uniform f64, premature optimization | heterogeneous typed lowering (f64/i1/i8/i64/struct); fine |
| Integer precision | f64, 53-bit (representation flaw) | true, but a *missing `Int` type*, not a representation flaw |
| Cross-Space wire | 4×f64, no strings | 8-f64 RPC + 64 KB structured stream; strings/structs cross; collections + structured `promote` are the real gap |
| Verification level | L1, no dependent types | L1 surface over an L2-capable dependent + cubical kernel |
| Prover | none; must build from scratch | Z3 + Coq exist for a narrow naturality domain; generalize them |
| Sum types | absent | parsed but lowers opaque (finish it) |
| Modules / packages | absent | full import system + git package manager |
| Effect system | absent | `visits`, statically checked + inferred |
| Continuations | absent | one-shot handlers; `multishot` parsed-only |
| FFI | absent | absent (correct) |
| Bytes / binary I/O | absent | absent (correct) |
| Stdlib breadth | ~20 modules | 33 modules |
| Tooling | none | fmt + lint + doc + LSP + editor clients |
| Heap addressing | Leech geometry | FNV-1a + memcmp; Leech in 3 separate roles |
| Runtime introspection | none | C-level yes, surface no |

---

## 3. Which analysis threads survive — and where they belong

| Analysis thread | Verdict | Disposition |
|---|---|---|
| "Abandon f64 / typed handles" | premise false | **Drop.** Replace with "add `Int`/`Decimal` types." |
| "4×f64 wire is the ceiling" | premise false | **Drop.** Replace with "add LIST/MAP wire tags + structured `promote`." |
| L1→L2 via formal spec + prover interface | sound, and half-built | **Adopt as the core.** Plumb kernel to surface; generalize Z3/Coq. |
| Kripke-embedding over recheck (intuitionistic→classical SMT) | strongest technical idea; now directly applicable | **Adopt** in the prover-backend WP. |
| Proof-failure diagnostics (countermodels, holes, 3-region Heyting) | sound; high value for agents | **Adopt** as the agentic differentiator. |
| Sum types / Result / Option | real gap | **Adopt** (finish the parsed-but-opaque path). |
| Bytes, FFI, crypto, networking | real gaps | **Adopt** selectively (commercial necessity). |
| Coalgebraic layer (Store comonad cells, process coalgebras, profunctor wires, co-Heyting) | intellectually rich; not on the commercial critical path; partly subsumed by `visits` + Space | **Research track**, not core. |
| Delimited continuations, linear types | real gaps; research-grade | **Research track.** |
| REPL / "Little Prover" pedagogy | sound; onboarding value | **Adopt** in ergonomics phase. |

---

## 4. Documentation honesty fixes surfaced during the audit

Flag to maintainers (cheap, high-trust-value — feeds Phase 0):

- `website/docs/book/06-worlds-and-places.md:73` calls `Magma.reachable`; the
  real op is `Land.reach`.
- `book/18` and `extends_*.yon` example *filenames* say `extends`; the
  implemented keyword is `subcontains` (example *bodies* are correct).
- The published syntax-reference module index documents 20 of 33 stdlib modules.
- `ROADMAP-1.1.md` lists `yon_xcoord_to_int24` as a stub and the δ-conversion
  debt as open; both are already done on `1.1-dev`.
- The Co₀ orbit cardinalities cited in docs are not backed by source constants.
- Magma op list in the analysis (subsetsum/knapsack/solve, "P=NP era") is
  retracted material per `MANIFEST.md`; real Magma ops are
  `empty/gen/is_commutative/is_associative/identity/closure_size/word_push/
  normal_form/from_catalog`.
