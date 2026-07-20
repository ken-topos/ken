# PX8-TR — native-lowering post-effect terminal-continuation route

> **▶ RESUME NOW (2026-07-20) — base `origin/main = 765c73ac`.** This is a
> **Runtime native-lowering completeness gap** that BLOCKS PX8-F, minted from the
> Architect's mechanism classification (evt_6c7w9ffc87k4c, thr_2gj12skfxqn5e) of
> the PX8-F post-PX8-DS hard-stop. **PX8-DS did its job:** the first real nested
> `withResource` `writeAll` program now emits, links, runs, and writes the exact
> bytes `abcdef`. The newly-exposed failure is **after that effect**: native
> lowering's post-effect terminal value is already `Lowered::Trap`, process-object
> compilation maps **any** final trap to sentinel `-4`, and the starter renders
> `-4` generically as `ken native trap: explicit entry trap` (exit 1) instead of
> `finish → host_exit Success` (exit 0). Repair the **post-effect
> semantic-result / terminal-continuation route** so a *successful* nested-bracket
> result exits Success. **Do NOT weaken any check or edit the PX8-F fixture.**

- **ID:** PX8-TR · **Owner:** **Team Runtime** (leader `agt_37reqrd72cg00` /
  implementer `agt_37reqg3nync00` / qa `agt_37reqvb6ce400`) · **Size:** S–M ·
  **Risk:** Medium (terminal-route native lowering; the fix must route *only* the
  genuinely-successful terminal continuation to Success and must not blunt the
  `-4` fail-closed backstop for real traps).
- **Branch:** `wp/px8-tr-terminal-continuation-route` off `origin/main @ 765c73ac`.
- **Gate:** **Runtime QA → Architect §14. NO CV lane** (compiler-private Runtime
  consumer realizing already-checked terminal semantics; no spec / conformance /
  observable-contract / public-IR / ABI / wire / catalog delta — same posture as
  PX8-DS / PX8-TA).
- **Blocks:** PX8-F (held immutable at `376773b6` — evidence, not the repair
  surface).

## Fixed inputs (Architect classification — settled, do not reopen)

1. **This is (a) a distinct Runtime native-lowering completeness gap** — NOT a
   PX8-TA checked-root/terminal-authority defect (those fail *before* object
   emission via `Unsupported` through `?`; this artifact emitted and ran), and NOT
   a PX8-F fixture/approach issue (fixture + both riders unchanged, backend delta
   is only the accepted `ResourceWriteCreate` seam, the effect runs with correct
   bytes). Do not re-litigate the classification.
2. **The `-4` sentinel is generic and erases provenance.** Process-object
   compilation maps *any* final `Lowered::Trap` to `-4`; the starter renders `-4`
   as "explicit entry trap" (`object_linker_packaging.rs:1925-1926`). `-4` does
   **not** identify an authority failure — it erases which particular
   `RuntimeTrap` / source-continuation / default route produced the final
   `Lowered::Trap`.
3. **The Architect deliberately did NOT name the faulty lowering arm** — the
   generic sentinel makes that claim ungrounded. **Grounding the exact arm via
   diagnostic provenance is the first deliverable** (AC1), not a guess.
4. **Preserve, do not weaken:** the PX8-DS affine dynamic-splice mechanism; the
   checked-root and terminal (`emit_result`-minted) authority checks; the `-4`
   fail-closed boundary as a backstop for *real* traps; and the immutable PX8-F
   candidate `376773b6`. **Do not edit the PX8-F fixture to manufacture exit 0.**

## Anchors (search surface — verified on `765c73ac`; re-grep the rebased tree)

- **Trap-sentinel rendering / final-trap → `-4`:**
  `object_linker_packaging.rs:1925-1926` (starter's `-4` message);
  process-object compilation entry `checked_process_object`
  (`object_linker_packaging.rs:730`).
- **Terminal emission fork** (final value → trap-sentinel vs `emit_result`
  success token): `cranelift_backend.rs:2576` / `:2581` and `:2789` / `:2797`
  (`Lowered::Trap(trap) => { … }` vs `value => compiler.emit_result(&mut builder,
  value)?`). `fn emit_result` at `:14078`.
- **`Lowered::Trap` production surface** (dozens of arms — the diagnostic must
  identify *which* fires on the successful nested-bracket path): the
  `return Ok(Lowered::Trap(...))` / `Err(trap) => Ok(Lowered::Trap(trap))` /
  eliminator-`default` arms across `cranelift_backend.rs` (e.g. the
  source-continuation / producer-default / eliminator-default routes near
  `:6237`, `:6267`, `:6303-6307`, `:6496`, `:6571`). Do **not** assume any of
  these is the culprit — AC1 grounds it.

## Mandated deliverable (ordered — diagnostic FIRST)

1. **AC1 — Trap-provenance diagnostic (grounds the faulty arm).** Add
   compiler-private or `#[cfg(test)]`-only instrumentation that captures, for the
   final `Lowered::Trap` reaching process-object compilation, the **exact
   provenance** the `-4` sentinel currently erases: which `RuntimeTrap` variant /
   source-continuation route / eliminator-or-producer `default` produced it, and
   at which terminal-route site. Drive the real nested-bracket path (the PX8-F
   trace shape, or a Runtime-owned equivalent) and **record the concrete
   provenance** of the trap that fires on the *successful* case. This is the
   evidence that names the arm; do not proceed to the repair until it is grounded.
2. **AC2 — Repair the real route.** Fix the grounded arm so a genuinely-successful
   post-effect nested-bracket terminal continuation reaches
   `finish → host_exit Success` (native program exits 0) instead of collapsing to
   `Lowered::Trap`/`-4`. The repair must be to the *specific* route AC1 grounds —
   not a blanket "map final trap to Success" (that would blunt the fail-closed
   backstop). Real traps must still reach `-4`.
3. **AC3 — Preserve the backstop and all intact mechanisms.** Keep the `-4`
   fail-closed boundary reachable for real traps; keep PX8-DS splice, checked-root
   + terminal authority, and the immutable PX8-F candidate unchanged.

## Acceptance criteria

- **Load-bearing reaching test:** a Runtime-owned nested-bracket native program
  (the successful `withResource`/`writeAll` shape) **compiles, links, runs, and
  exits 0 (`host_exit Success`)** on this branch — where before the fix it exits 1
  with the `-4` "explicit entry trap". The test must drive the **real** terminal
  route (emit → link → run the artifact), not a unit assertion on the mapping.
- **Negative control retained:** a genuine terminal trap still routes to `-4`
  (the fail-closed backstop is not blunted). Assert the specific trap path, not
  just non-zero exit.
- **AC1 provenance diagnostic** is present (compiler-private/test-only) and
  demonstrably distinguishes the repaired route from the generic sentinel.
- `ken-elaborator/src/` and all non-Runtime surfaces **byte-unchanged**; the
  PX8-F fixture **untouched**.
- Targeted suites green via `scripts/ken-cargo -p ken-runtime` (+ the reaching
  test's crate, e.g. `-p ken-cli --test <name>`); **no `--workspace`** —
  workspace-green / `--locked` / conformance run in CI.

## Do-not-reopen guard

Repair exactly the terminal-continuation route AC1 grounds. Do **not** weaken the
`-4` backstop, the checked-root/terminal authority, the PX8-DS endpoint checks, or
any fail-closed gate; do **not** edit the PX8-F fixture or manufacture exit 0 by
special-casing the test. Do **not** touch the elaborator or any producer surface.
If the grounded route cannot be repaired without a broader change (e.g. it
implicates the checked terminal contract, reclassifying toward (b)), **stop and
route to the Steward + Architect** rather than expanding scope.

## Coordination note

`cranelift_backend.rs` + `object_linker_packaging.rs` are Runtime-owned. PX8-F is
held immutable at `376773b6` and re-kicks as a semantic rebase onto the new `main`
once PX8-TR lands (its `px8f_buffer_native` nested trace is the ultimate consumer
proof, but PX8-TR is validated on its own Runtime-owned reaching test).
PX8-J-ERR (#30, `install_recursor_invocation` guard) stays parked behind this on
the Runtime ring. Release the candidate SHA on
`wp/px8-tr-terminal-continuation-route` when green locally; the Steward routes
Runtime QA → Architect §14 → publisher.
