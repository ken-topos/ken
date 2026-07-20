# PX8-DS — oriented dynamic-splice edge (invocation-local segments + affine child→parent splice)

> ## ▶ KICK-READY (2026-07-20)
>
> **Owning team:** Runtime. **Reviewers:** Runtime QA → **Architect §14 only.**
> **NO CV lane** — the change is compiler-private native lowering; it touches
> **no** `spec/` or `conformance/` path and changes **no** public/observable
> semantics, public IR, ABI, or wire format. It converts a compile-time **false
> rejection** of a valid checked program into a correct compile — a completeness
> fix, not a behavior change. **Size:** S–M. **Risk:** medium-high (touches the
> trusted native oriented-subcontinuation lowering the PX8-TA redesign just
> landed; fix-forward, no rollback; must not weaken any fail-closed check).
>
> **Base / branch:** `wp/px8-ds-oriented-dynamic-splice-edge`, cut from current
> `origin/main @ e45ca05e8796734fd1b6656169e69c6f26fae08b` (PX8-TA merge-base;
> never stale local `main`). One WP, one branch, one §14 Decision.
>
> **This is a BLOCKING prerequisite for PX8-F.** PX8-F is HELD immutable at
> `origin/wp/px8f-buffer-io-surface @ 2da1e590f7a36ab3f5576252d0362550cf4ed40d`
> and re-kicks (as another semantic rebase onto post-PX8-DS main) only when this
> lands. **Do not touch PX8-F's branch or fixtures from this WP.**
>
> **Authority (fixed inputs; execute, do not re-derive or re-open):**
> - Architect **mechanism ruling** `evt_7rdeq8hmetpsv` (thread
>   `thr_2gj12skfxqn5e`, 2026-07-20 06:09Z) — the defect's exact landed causal
>   path + the corrected mechanism boundary, reproduced verbatim in §2/§3.
> - Architect **superseding endpoint correction** `evt_4apv0j2enhvzy`
>   (2026-07-20) — a frame's terminal output is a checked answer-transformer
>   boundary, while a computational-IH call's result/caller interface is an
>   independently inferred application-result descriptor. Their byte equality
>   is a false cross-domain invariant, including at a real dynamic edge. The
>   lawful checks are internal frame-segment adjacency and the call's own
>   result-to-caller equality.
> - Steward **scope disposition** `evt_3c4amv9sf1peg` (2026-07-20 06:10Z) —
>   this WP's framing, ownership, gate, and PX8-F re-freeze.
> - The accepted **PX8-TA** checked interfaces (merged `e45ca05e`, PR #784) are
>   the substrate: checked descriptors, endpoint interfaces, semantic/control
>   separation, occurrence census, and root-terminal authority are **CORRECT and
>   UNCHANGED**. This WP does **not** re-open them.

## 1. Objective (one line)

Make native lowering of nested/sibling computational recursive invocations
compose by their **exact dynamic parent occurrence**, not by a global static
sort: keep each dynamic invocation's semantic frames as one invocation-local
oriented segment and join segments with one exact affine child→parent dynamic
splice edge minted at checked IH-marker consumption — deleting the flat
cross-instance `(depth, semantic_position)` ordering rule — **without
weakening any lawful same-domain endpoint or fail-closed check.**

## 2. The defect (Architect `evt_7rdeq8hmetpsv`, fixed input — do not re-open)

A valid checked nested program (PX8-F's outer `withResource ResourceRead` → body
opens nested `withResource (ResourceWriteCreate CreateOrTruncate)` → `withBuffer`
/ read / real recursive `writeAll`) is **falsely rejected** before CFG at object
emission:

```
Packaging(ObjectLinkerPackagingError { stage: ObjectEmission,
  field: "checked_process_object",
  reason: "unsupported runtime-IR lowering: OrientedSubcontinuationPlanV1:
    oriented splice answer endpoints do not compose:
    left=(instance=330, frame=7, depth=1, out=12711271229755895252)
    right=(instance=331, frame=5, depth=1, in=14516916414270856217)
    order=[(330,7,1),(331,5,1),(332,4,1),(333,3,1),(334,2,1),(335,1,1),(336,0,1)]" })
```

This is a **compile-time false rejection of a valid checked nested program — NOT
a miscompile, NOT grounds to relax fail-closed validation.** The exact landed
causal path (Architect-verified):

1. `mint_checked_computational_ih_instance` gives every fresh IH instance
   `semantic_depth = active_recursive_invocations.len() + 1`. IH instances are
   **not** pushed on that declaration-recursion stack, so all seven reaching
   instances lawfully receive **depth 1**.
2. `CheckedRecursiveInvocationInstance` retains only static call source, the
   fresh instance ID, and that coarse depth.
3. `compose_oriented_subcontinuation` then **flattens every pending semantic
   layer into one map and globally sorts distinct dynamic instances by
   `(Reverse(depth), static semantic_position)`.** It consequently **invents
   adjacency** between `(330, frame 7)` and `(331, frame 5)`; their checked
   frame-transformer endpoints correctly reject.
4. The checked IH-call template's `parent_frame_template_id` is only a
   **reusable static edge**. Current validation proves the selected static
   template but **cannot identify which dynamic parent occurrence owns this
   child.**
5. **The missing dynamic fact has NOT been erased at mint time:** the consumed
   recursor closure's selected layer already carries its exact
   `checked_invocation_id` and `checked_frame_id`; in the source consumer,
   `control.selected.selected_scope` independently exposes the same open
   occurrence and is already checked against the static call parent. Runtime
   currently **validates that occurrence and then drops its dynamic parent
   identity.**

## 3. The corrected mechanism boundary

Architect `evt_7rdeq8hmetpsv`, as superseded for endpoint meaning by
`evt_4apv0j2enhvzy`, is fixed input.

Implement exactly this; do not substitute a different cut.

1. **Mint a move-only dynamic splice edge at exact checked IH-marker
   consumption**, from the fresh child invocation to the exact open parent
   occurrence, **qualified at least by**: child instance, parent invocation
   instance, checked call template, parent frame template, and segment site.
   **Cross-check** the closure-selected occurrence and, where present, the
   source open occurrence; **disagreement rejects before CFG.**
2. **Invocation-local oriented segments.** Keep each dynamic invocation's
   semantic frames in the **checked static order** as one invocation-local
   oriented segment. **Validate that segment internally against its checked
   frame-transformer endpoints** (the existing within-segment adjacency check
   — unchanged, not weakened).
3. **Splice on completion, once.** Complete a child segment, then consume its
   exact dynamic edge **once** to splice it into the **named parent** at the
   checked IH call occurrence. Retain the call's own
   `result_interface == caller_interface` check. The edge names the parent
   occurrence; it does **not** assert that the child's terminal frame output
   equals the independently inferred IH-call result descriptor. **Nested and
   sibling instances remain separate until their own completion; no child may
   steal another sibling's edge or token.**
4. **`ExitsScope` stays affine control-only.** Retain inherited `ExitsScope`
   rows as affine control obligations. They **do not** become executable
   semantic frames and **do not** select semantic order.
5. **Delete the global ordering rule.** Remove the global
   `depth + semantic_position` cross-instance ordering entirely. **Invocation
   allocation ID, Rust call depth, control-occurrence order, provenance,
   constructor shape, and trial endpoint matching are NOT lawful substitutes.**
6. **Fail-closed in every consumer + the source machine.** Deletion,
   duplication, stale-parent, cross-sibling, and wrong-static-parent mutations
   must **reject before CFG in all three direct consumers AND the source
   machine.**

## 4. Surface (the fence)

Compiler-private **Runtime consumer** surface only — the PX8-TA
elaborator-side producer (`ken-elaborator/src/{erasure,compiler_driver,checked_core}.rs`)
is **UNCHANGED** (the dynamic fact is already produced/carried; the defect is
purely in Runtime consumption). Work within the landed PX8-TA Runtime fence:

- `crates/ken-runtime/src/oriented_subcontinuation_plan.rs` — the plan
  representation / `compose_oriented_subcontinuation`; the dynamic splice edge +
  invocation-local segment structure live here.
- `crates/ken-runtime/src/cranelift_backend.rs` — object-emission consumer;
  splice at the exact edge-qualified IH-call occurrence; retain the call's
  result-to-caller equality without comparing that descriptor to a frame
  terminal; mutation rejection pre-CFG.
- `crates/ken-runtime/src/ir.rs` — marker/edge carrier if the edge needs a
  representation slot (add only what §3.1's qualification requires; no public IR
  change).
- The **source machine** and the **three direct consumers** named by the
  Architect — `runtime_ir_evaluator.rs`, `native_execution_differential.rs`,
  and the backend — must each mint/cross-check/consume the edge and reject the
  §3.6 mutations. Pin exact call sites against the landed code (do not assume
  line numbers; the landed symbols are `mint_checked_computational_ih_instance`,
  `CheckedRecursiveInvocationInstance`, `compose_oriented_subcontinuation`,
  `parent_frame_template_id`, `control.selected.selected_scope`).
- The `ken-cli` reaching tests (add the §5 discriminator alongside the existing
  `px8ta_oriented_subcontinuation` / `px7n_nested_computational_eliminator`
  suites).

No kernel, spec, conformance, public Runtime IR/semantics, source syntax,
ABI/wire, or catalog change. If any of those appears necessary, **hard-stop to
the Architect (mechanism) and the Steward (scope)** — do not widen silently.

## 5. Mandated deliverable outline (each section ends in a concrete choice)

1. **Dynamic splice edge type** — the move-only (non-`Clone`) edge with the §3.1
   qualification tuple; how it is minted at IH-marker consumption and how it is
   affine-consumed exactly once. *End: the concrete struct + mint/consume sites.*
2. **Invocation-local segment** — how each invocation's frames are grouped in
   checked static order and validated internally against checked
   frame-transformer endpoints (reusing, not rewriting, the within-segment
   adjacency check). *End: the concrete grouping + validation call.*
3. **Compose rewrite** — `compose_oriented_subcontinuation` no longer flattens +
   globally sorts; it splices completed child segments into named parents along
   the dynamic edge. *End: the deleted sort + the new splice.*
4. **Cross-checks & rejection** — closure-selected vs source open occurrence
   agreement; the five mutation classes (delete / duplicate / stale-parent /
   cross-sibling / wrong-static-parent) rejecting pre-CFG in all three direct
   consumers + the source machine. *End: each rejection lane named.*
5. **Discriminator test** — §7's reaching same-depth nested/sibling test.
   *End: the test file + the two arms.*

## 6. Build/test discipline (hard rules)

- **⛔ TARGETED BUILDS ONLY — NEVER `--workspace`** (`COORDINATION.md §12`,
  operator hard rule). Use `scripts/ken-cargo -p ken-runtime` / `-p ken-cli
  --test <name>`. The full-workspace/`--locked`/conformance gate runs in **CI**,
  not on this box. "Workspace-green" in any AC = **green in CI**.
- Run the executed nested-eliminator suites (`px7n_nested_computational_eliminator`,
  `px8ta_oriented_subcontinuation`) as part of the QA set — **RUN, not
  `--no-run`** ([[declref-lowering-change-needs-elaborator-integration-tests]],
  [[ken-cargo-workspace-does-not-match-ci-locked-for-rejection-changes]]).
- Keep the turn active through adaptation → focused gates → rebase → commit →
  release → exact-SHA handoff.

## 7. Acceptance criteria — merged evidence bar (all mandated; each a real reaching test)

1. **The reaching same-depth nested/sibling discriminator** (Architect-mandated,
   the load-bearing AC): a test in which multiple same-`depth`-1 nested/sibling
   invocations exist, proving **the old flat `(depth, semantic_position)` sort
   reproduces the present `(330,f7)↔(331,f5)`-style rejection while the exact
   dynamic edges compose.** Both arms must reach the real producer/consumer path
   (not a hand-built view). The negative arm — the flat sort — must observably
   reproduce the false rejection; the positive arm — exact edges — compiles,
   links, and runs.
2. **Nested composition compiles & runs.** The minimized PX8-F-shaped nested
   `ResourceRead` → nested `ResourceWriteCreate` → `withBuffer`/`writeAll`
   program lowers, links, and executes to the correct result (no
   `oriented splice answer endpoints do not compose`).
3. **Lawful endpoint checks NOT weakened.** A genuinely non-composable checked
   program (mismatched frame-transformer endpoints within an invocation-local
   segment) still rejects before CFG, and a checked IH call whose own result and
   caller descriptors differ still rejects. No test may reintroduce or require
   the withdrawn cross-domain terminal-frame-output = IH-call-result oracle.
4. **Five mutation rejections, all four+ consumers.** Deletion, duplication,
   stale-parent, cross-sibling, and wrong-static-parent edge mutations each
   reject **before CFG** in the object-emission backend, the source machine, and
   the other two direct consumers. Each rejection through a **named lane** (not a
   generic panic).
5. **Sibling isolation.** A test proving no child consumes another sibling's edge
   or token (affine, exactly-once, per-parent).
6. **`ExitsScope` affinity preserved.** Inherited `ExitsScope` rows remain affine
   control obligations and do not select semantic order (retain the PX8-TA
   projection-separation discriminator, still green).
7. **No-regression.** PX8-TA suite (`px8ta_oriented_subcontinuation` public
   depths 1/2/3 exact LIFO), `px7n` nested two-executor, PX8-L, ken-runtime lib,
   ken-elaborator lib, `ken fmt`, capstone — all green; **full-workspace green in
   CI.** Elaborator-side files byte-unchanged (`git diff` empty over
   `ken-elaborator/src/`).

## 8. Scope fence & do-not-reopen guards

- **DO NOT weaken lawful same-domain endpoint checks or any fail-closed
  validation.** Preserve invocation-local frame-transformer adjacency and each
  checked call's result-to-caller equality. Do **not** compare a terminal frame
  output to a computational-IH result descriptor or reintroduce the withdrawn
  cross-domain mutation oracle. The defect is a false rejection *because
  dynamic identity was dropped* — the fix restores identity; it does not relax
  either lawful check. (Architect, explicit.)
- **DO NOT re-open the accepted PX8-TA mechanism** — checked descriptors,
  endpoint interfaces, semantic/control separation, occurrence census, and
  root-terminal authority are fixed inputs.
- **DO NOT touch the elaborator producer** — the dynamic fact is already carried;
  this is Runtime consumption only. An elaborator edit = a scope hard-stop to the
  Steward.
- **DO NOT substitute a static proxy** for the dynamic edge (alloc ID, Rust call
  depth, control-occurrence order, provenance, constructor shape, trial matching
  are explicitly non-lawful).
- **DO NOT touch PX8-F's branch/fixtures.** PX8-F re-verifies its own unchanged
  public trace post-landing.
- **PX8-J-ERR** (`install_recursor_invocation` guard, queued) may follow this on
  Runtime; flag the Steward if the splice-edge site overlaps
  `install_recursor_invocation`.
- Any genuinely new mechanism boundary → **hard-stop** (mechanism → Architect,
  scope → Steward), do not work around.

## 9. On landing

Runtime QA APPROVE (exact SHA) → Architect §14 (one Decision, resolve-on-cast) →
Steward publishes via the scripted publisher (non-doc-only, full CI) → verify by
content on `origin/main` → PIN → **notify adversary** (code merge) → post §10
retros (leader + implementer + QA) → close. Then **Steward re-kicks PX8-F** as a
semantic rebase of `2da1e590` onto the new post-PX8-DS `origin/main`
(re-derive anchors again; PX8-F gate stays Foundation QA → Architect §14 → CV).
