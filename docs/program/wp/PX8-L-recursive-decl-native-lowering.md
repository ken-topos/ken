# PX8-L — checked-native recursive-declaration lowering

> **Prerequisite for PX8-F, opened by Architect ruling `evt_2vgt1s790vaee`
> (2026-07-18).** PX8-F hard-stopped on a genuine checked-native compiler
> boundary: compiling the public transparent **recursive** `writeAll` fixture
> overflows a 256 MiB-stack thread during `build_native_program`, before any
> linked execution. Architect classified it as **(A), refined to the compiler
> pipeline (NOT kernel conversion):** the checked-native compiler misuses strong
> `ken_kernel::normalize` as a whole-program inliner — a transparent
> structurally-recursive declaration called at a neutral predecessor unfolds to a
> stuck match, normalization descends into its successor method, meets the
> recursive call, and repeats **without a finite declaration graph.** Stopping
> the eager unfolding alone is insufficient: the same pipeline then asserts the
> normalized root is self-contained and keeps only that root, while the live
> backend explicitly **rejects** a recursive `DeclarationRef`
> (`cranelift_backend.rs:1454` — "requires NC22+ recursive lowering"). The
> complete defect is therefore **general finite preservation *and* native
> lowering of admitted recursive declaration groups** — a language mechanism, not
> a PX8-F-specific tweak. **Foundation's term is valid:** recursive transparent
> declarations admitted by SCT are a language feature.
>
> **★ The immutable downstream discriminator is `wp/px8f-buffer-io-surface @
> c8b8cdb7` (on origin).** After PX8-L merges, PX8-F rebases onto it and the
> **unchanged** real transparent `writeAll` fixture must compile, link, and pass
> behaviorally in **both** the interpreter and native lanes. Do NOT edit that
> fixture, rewrite `writeAll`, add a host/buffer intrinsic, weaken transparency,
> or treat a larger thread stack as a fix (all forbidden by the ruling).

- **ID:** PX8-L · **Owner:** **Team Runtime** (leader `agt_37reqrd72cg00` /
  implementer `agt_37reqg3nync00` / qa `agt_37reqvb6ce400`) · **Size:** L ·
  **Risk:** High (checked-native compile pipeline; the recursive lowering must
  preserve the **SCT structural-decrease contract** and stay **fail-closed** —
  soundness-adjacent; a new compiler capability that only full-workspace CI
  fully exercises).
- **Branch:** `wp/px8l-recursive-decl-native-lowering` — `git branch
  wp/px8l-recursive-decl-native-lowering origin/main` (base `origin/main @
  3be76cc2`, fetched, never stale local `main`). One branch, one PR.
- **Route:** **Architect §14** (the recursive lowering preserves the SCT
  structural-decrease contract, argument/capture/producer-environment order, and
  fail-closed admission — soundness-adjacent) **+ Runtime QA**. **+ CV** only if
  the candidate touches `spec/`/`conformance/` (not expected — this is a
  compiler/backend mechanism, no surface change). One Decision on the tip.
- **Ownership note (Architect):** this is **Runtime-owned even though the crash
  enters `ken_kernel::normalize`** — the compiler is *misusing* full
  normalization as a cyclic inliner, and the downstream missing mechanism is
  Runtime code generation. **Do NOT change kernel conversion semantics for this
  WP.** If a separate minimal test later shows the kernel normalizer violates an
  independently normative API contract, route that as a **distinct Kernel
  defect**, never buried here.

## Objective

Give the checked-native compile pipeline a **finite declaration-graph boundary**
and **general native lowering of admitted recursive declaration groups**, so that
any SCT-admitted transparent recursive declaration compiles with bounded
stack/memory and lowers to correct native code — implementing the capability the
backend currently punts as "requires NC22+ recursive lowering". This is a
**general language-mechanism** fix (any recursive transparent `HostIO`/computational
declaration), proven on a resource-independent recursive program, **not** a
`writeAll`/buffer-specific patch. PX8-F is unblocked as a downstream consequence,
not as this WP's deliverable.

## Fixed inputs — DO NOT REOPEN (Architect ruling `evt_2vgt1s790vaee`; settled)

- **The two-part defect (both halves must be fixed; one alone is insufficient):**
  1. **Eager cyclic inlining.** The checked-native compiler uses unrestricted
     strong `ken_kernel::normalize` as a whole-program inliner. A transparent
     structurally-recursive declaration called at a neutral predecessor unfolds
     to a stuck match; normalization descends into the successor method, meets
     the recursive call, and repeats with no finite declaration graph → stack
     overflow (`compiler_driver.rs:1301-1326`, the live main normalization).
  2. **Missing native lowering.** Even with eager unfolding stopped, the pipeline
     asserts the normalized root is self-contained and retains only that root,
     while the backend **rejects** a recursive `DeclarationRef`
     (`cranelift_backend.rs:1452-1454`, "requires NC22+ recursive lowering").
- **The term is VALID.** Recursive transparent declarations admitted by SCT are a
  language feature. **Do not** reject Foundation's `writeAll`, make it opaque, or
  demand it be rewritten. The fix is in the compiler/backend, not the surface.
- **Guardrails (all forbidden):** no `writeAll`/buffer intrinsic; no opaque
  postulate or `Axiom`; no test-only producer; no stack-size oracle / larger-stack
  band-aid; **no kernel-conversion-semantics change.**
- **SCT contract is preserved, not re-derived.** The structural-decrease guarantee
  admitted by SCT is the contract the native lowering must honor and may consume;
  do not re-prove or weaken it.
- **R2 closed / targeted-builds-only / PRINCIPLES transient-T** remain in force
  (no in-language mutation; `scripts/ken-cargo -p <crate>` only; no-regression =
  GREEN IN CI).

## Landed anchors (verify before editing; do not trust frozen line numbers)

- `crates/ken-elaborator/src/compiler_driver.rs`: the checked-host transaction
  (**~1203+**); **live main normalization `~1301-1326`** (`ken_kernel::normalize(...,
  body)` used as the whole-program inliner — the first crash site); host-package
  **erasure `~1369+`**; the "normalized root is self-contained → retain only that
  root" assumption that must become a **finite closure/deforestation boundary**
  retaining the exact admitted recursive declaration or group.
- `crates/ken-runtime/src/cranelift_backend.rs`: `lower_declaration_ref`
  (**~1398**); the **recursive-`DeclarationRef` rejection at `~1452-1454`**
  ("requires NC22+ recursive lowering") — this is the punt PX8-L replaces with
  real general recursive lowering (preserving argument/capture/producer-env order
  and the SCT structural-decrease contract); computational producer lowering
  (**~2358+**).
- `crates/ken-elaborator/src/erasure.rs`: the erasure path that must preserve the
  recursive declaration group finitely (Architect examined
  `match_uses_computational_recursive_hypothesis` / `erased_count` here).
- **Downstream discriminator (do NOT modify):** `wp/px8f-buffer-io-surface @
  c8b8cdb7026eb39f92bbde29329b0c0ae0d0a2a8` — the real transparent recursive
  `writeAll` red. GREEN boundary `-p ken-elaborator --test px8f_buffer_io_surface`
  (2/2); RED `-p ken-cli --test px8f_buffer_native` (SIGABRT).

## Mandated deliverables (each ends in a concrete implementable choice)

- **L-D1 — finite checked-host closure boundary (kill the cyclic inliner).**
  Replace the "fully normalize then keep one root" assumption in
  `compiler_driver.rs` with a **finite checked-host closure / deforestation
  boundary** that retains the exact admitted recursive declaration or group
  (nodes + edges), instead of eagerly unfolding a transparent recursive
  declaration through strong `ken_kernel::normalize`. Normalization used for
  compilation must terminate on any SCT-admitted recursive declaration. Do not
  change kernel conversion; bound the compiler's *use* of it.
- **L-D2 — general native lowering of recursive declaration groups.** Implement
  general recursive-`DeclarationRef` lowering in the checked-native backend
  (`cranelift_backend.rs`), replacing the `~1454` "requires NC22+ recursive
  lowering" rejection. Preserve **argument, capture, and producer-environment
  order** and the **SCT-approved structural-decrease contract** across the
  recursive call. Any admitted recursive transparent computational/`HostIO`
  declaration lowers to correct native code.
- **L-D3 — fail-closed admission boundary.** A **non-decreasing** recursive cycle
  must remain **rejected** (not inlined forever, not lowered). An **unsupported**
  recursive shape must report a **typed compiler boundary** (a real diagnostic),
  never inline forever or silently miscompile. Assert the specific typed-boundary
  variant, not a generic failure.
- **L-D4 — the proof program (resource-independent).** Author a
  **resource-independent** transparent recursive `HostIO` program with a
  **dynamic/neutral `Nat` seed** (NOT a buffer, NOT `writeAll`, no host I/O
  primitive) that:
  - compiles via `build_native_program` with **bounded stack/memory** (no
    overflow), and
  - runs correctly for a **zero-step** seed and a **multi-step** seed
    (interpreter↔native agreement on the observable result).
  Include a **parameter-update / capture-order discriminator** — the recursion
  must thread and update a changing parameter each step (mirroring how real
  `writeAll` advances offset, span, and remaining fuel), so a lowering that
  drops/reorders the updated argument or capture is caught (assert the exact
  stepped result, not merely "it ran").

## Required proofs / discriminators (each independently reaching)

1. **Bounded compile (kill the overflow):** the L-D4 recursive program compiles
   via `build_native_program` with bounded stack — a mutation restoring eager
   whole-program normalization re-introduces the overflow (the mechanism is the
   finite boundary, not a bigger stack).
2. **Native lowering reached:** the recursive `DeclarationRef` lowers to native
   code (grep the emission / the `~1454` rejection path is not taken) and links.
3. **Structural-decrease + parameter-update fidelity:** the multi-step seed
   produces the **exact** stepped result in the native lane, matching the
   interpreter; a lowering that drops the updated argument/capture or reorders the
   producer environment fails the discriminator (not just `is_err`).
4. **Zero-step base:** the zero-seed program returns the base result with no
   recursive body execution.
5. **Fail-closed (L-D3):** a non-decreasing cycle stays rejected; an unsupported
   recursive shape returns the **typed compiler boundary** (assert the variant).
6. **Downstream discriminator (verification, at handback — NOT a deliverable
   edit):** on a throwaway rebase of `c8b8cdb7` onto the PX8-L tip, the
   **unchanged** `px8f_buffer_native` real-`writeAll` fixture compiles, links, and
   passes behaviorally in both lanes. (PX8-F's own re-kick does the real rebase;
   this is only PX8-L's evidence that the boundary is truly gone.)

## Acceptance criteria (testable)

- **AC1** — L-D1..L-D3 landed: the checked-native pipeline compiles any
  SCT-admitted transparent recursive declaration with bounded stack, lowers
  recursive `DeclarationRef` generally, and is fail-closed on non-decreasing /
  unsupported shapes (discriminators 1-3, 5).
- **AC2** — the L-D4 resource-independent recursive `HostIO` proof program: bounded
  compile, correct zero-step and multi-step interpreter↔native agreement, and the
  parameter-update/capture-order discriminator all GREEN (discriminators 1-4).
- **AC3** — the `~1454` "requires NC22+ recursive lowering" rejection no longer
  fires for admitted recursive declarations (grep the emission), and a
  non-decreasing cycle still hits a typed boundary (discriminator 5).
- **AC4** — no kernel-conversion-semantics change (no edit to kernel `normalize`
  conversion behavior); no `writeAll`/buffer intrinsic, opaque, test-only
  producer, or stack-size oracle anywhere in the candidate.
- **AC5** — **no-regression = GREEN IN CI** (never a local `--workspace` run;
  COORDINATION §12). Build/test **targeted only** (`scripts/ken-cargo -p
  ken-elaborator …` / `-p ken-runtime …` / `-p ken-cli --test <name>`), plus run
  the `ken-cli` native-compile suites the recursive-lowering change implicates
  before release (a compile-pipeline change can break checked-program consumers
  only full CI catches — the PX8-N lesson).
- **AC6** — downstream discriminator (discriminator 6): on a throwaway rebase of
  `c8b8cdb7`, the unchanged real `writeAll` fixture compiles+links+passes both
  lanes. Report the evidence; do **not** modify `c8b8cdb7`.

## Do-not-reopen guard

- Do **not** change kernel conversion semantics; bound only the compiler's *use*
  of normalization. A kernel normalizer API-contract violation, if independently
  shown, is a **distinct Kernel defect** — route it, don't bury it.
- Do **not** reject / rewrite / make-opaque Foundation's transparent recursive
  `writeAll`, or edit the `c8b8cdb7` fixture.
- Do **not** add a `writeAll`/buffer intrinsic, opaque postulate/`Axiom`,
  test-only producer, or stack-size/larger-thread oracle.
- Do **not** weaken or re-derive the SCT structural-decrease contract — preserve
  and consume it.
- Do **not** let an unsupported recursive shape inline forever or miscompile — it
  must hit a typed compiler boundary (fail-closed).
- If the general recursive lowering surfaces a genuinely new substrate blocker,
  **HARD-STOP to the Steward** with a minimized red/green pair — do not paper over
  it (the discipline that produced this WP).

## Sequencing

**PX8-N ✅ (`ace72db7`) → PX8-L (this) → PX8-F rebased terminal gate → Phase-C
exit (Ken side).**

- PX8-F is **held at `c8b8cdb7`** (immutable downstream discriminator) until PX8-L
  lands. On PX8-L merge: Steward re-kicks PX8-F — Foundation rebases the preserved
  surface onto the combined main and requires the **unchanged** real `writeAll`
  fixture to compile+link+pass in both lanes (that becomes PX8-F's native
  evidence).
- **Phase-C exit** (`cat`/`cp`/`wc` native over a larger-than-memory file,
  interpreter↔native external-delta equality via the PX6 harness) needs PX8-L +
  PX8-F (rebased) landed — the Ken buffer-IO floor. Ward remains external
  throughout (no in-Ken monitor).
