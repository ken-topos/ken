---
id: NATIVE-HANDLE-CARRIER
title: "Native build-pipeline completeness — a constructor-private resource-carrying handle fails checked-core body-view lowering (MissingClosureMetadata) when it crosses the higher-order withBuffer normalization boundary"
status: active
owner: runtime
size: M
gate: none
depends_on: [RT-NATIVE-FNSPLIT]
blocks: [PX8-F-CAP-41]
github: null
origin: discovered under [[PX8-F-CAP-41]] Phase 2 impl (foundation-implementer hard-stop evt_563ss8821n7f); Architect means/representation ruling evt_2zkjr68y1sdgf (thr_570t9qzcthjv9, 2026-07-23). Steward-filed (agents cannot create tracked work per COORDINATION §2).
---

> ## ⭐ §5a RESEARCH-CONSULT TRIGGER — THE COUNT OF RECORD LIVES HERE NOW
>
> Carried forward from [[RT-NATIVE-FNSPLIT]] when it closed 2026-07-29.
> ⛔ That node's own §5a section still reads a stale **15**; its closeout says so.
> ⛔ Do not read the count from any node other than this one.
>
> | | |
> |---|---|
> | **COUNT OF RECORD** | **20** |
> | ENTRIES | 11 |
> | NEXT PREDICATE CHECK | 12th entry |
> | NEXT RESEARCH PULL | **#21 — one hard stop away** |
>
> **Hard stop #20 (2026-07-29):** Foundation's [[PX8-ERRID-ALLOC]] rebase still
> failed the native size gate at `checked_process_object`. Architect ruling
> `evt_3t7t27e3rv8cx` — the object routes to the monolithic `RecursiveDescent`
> root, so `FunctionizedUnits` never applied. Produced
> [[RT-DECL-CLOSURE-PORT]].
>
> ⚠ **#21 is the next hard stop, and it fires the research pull.** The Architect
> self-triggers it; the Steward holds the count of record as backstop.


> ## ⛔ `draft` → `ready` 2026-07-28 — the banner promised what the status withheld
>
> This node said **"✅ FRAMED — shovel-ready"** while its frontmatter said
> `status: draft`. ⛔ **`gen-progress.sh` computes the frontier as `status:
> ready` AND every `depends_on` merged/closed** — so at `draft` this node would
> **not** have entered the frontier when `RT-NATIVE-FNSPLIT` merged, no matter
> what the banner claimed. ⇒ A Steward pass would have had to stand between the
> umbrella's merge and this kickoff, which is exactly what `§2a-bis` exists to
> remove.
>
> ⭐ **`ready` is correct despite the unmerged dependency.** `RT-SCALE-B` is the
> in-repo precedent: `ready` with an `active` dep. **Blocking is expressed by
> `depends_on`, not by `draft`** — the frontier ANDs the two. `draft` is a claim
> about *framing*, and this node's framing is done.

> ## ✅ FRAMED 2026-07-27 — shovel-ready; blocked ONLY on `RT-NATIVE-FNSPLIT`
>
> **Frame:** `docs/program/wp/NATIVE-HANDLE-CARRIER.md`, measured at
> `origin/main = 5404108a`. Owner **Runtime**, size **M**.
>
> ⭐ **This WP closes [[PX8-F-CAP-41]] Phase 2 in the same merge** — one
> deliverable, two nodes. Flip both together.
>
> ### ⛔ Premise correction the frame carries: there is ONE input ref
>
> The text below says to "fold `c07e63c2` with `f0eb65ce`". **Measured: there is
> nothing to fold** — `f0eb65ce` is `c07e63c2`'s parent. Take `c07e63c2` alone
> (`origin/preserved/native-handle-carrier-c07e63c2`); it already carries the
> handle/admission impl *and* the elaborator slice.
>
> ### ⚠ And the rebase is real work, not a preliminary
>
> `c07e63c2` is based at `8ebe370a`; **`origin/main` is 215 commits ahead**, and
> `prelude.rs`, `erasure.rs`, and `compiler_driver.rs` — all three production
> files of the elaborator slice — were **also edited on `main`** (+224 lines
> there against the branch's +188). A side-preference conflict resolution
> silently reverts landed work. That is `AC-1`.
>
> ⭐ **Status stays `draft` because `depends_on` is unmet**, not because it is
> unframed. Flip to `ready` when [[RT-NATIVE-FNSPLIT]] merges, then kick Runtime
> with a full handoff gate.

## ⚙ RE-HOMED to Runtime 2026-07-23 (elaborator slice DONE; continuation is native)

The **elaborator half is complete** — Foundation de-erased the driver error and fixed
the true root cause: `MissingClosureMetadata` was **masking**
`CheckedCoreBodyViewError::UnsupportedTermShape` / `int_lit_outside_native_i64` —
checked-core `BigInt` literals were narrowed to `i64`, and the CAP-41 fixture reaches
u64-max via the checked `intToUInt64` bound. Foundation widened checked-body literals
to `BigInt` (lossless map to `RuntimeIntV1`), preserving the underlying error through
the driver. Body-view + erasure GREEN. **Preserved on origin: `wp/NATIVE-HANDLE-CARRIER
@ c07e63c2`** (parent carrier fixture `f0eb65ce`; the two-commit branch is one
`ken-elaborator` production slice + test call-site migrations; **no `ken-runtime`
touched**). Sized **S** by the implementer.

The fixture now advances through body-view/census/erasure and fails only at **object
emission**: `int_to_uint64_raw is not in the supported native set`
(`crates/ken-runtime/src/cranelift_backend/lowering/core.rs`). **The remaining work is
`ken-runtime`** — add the primitive, carry the CAP-41 fixture to full native GREEN
(lifting any *further* stacked native gaps), then **fold with `c07e63c2`** and run the
Architect's six-axis matrix + controls = the full two-engine oracle. That merge closes
**this WP and [[PX8-F-CAP-41]] Phase 2** together.

**⛔ Serialized against [[RT-NATIVE-FNSPLIT]] — FAST-FOLLOW (Steward ruling
`evt_1v37rgez26kmf`, runtime-leader read `evt_7dedryvh3fd48`).** RT-NATIVE-FNSPLIT
lands first (it owns the indivisible `lowering/core.rs` continuation-partitioning
change); combining it with CAP-41's primitive-support + two-engine oracle would make
one high-risk unreviewable `core.rs` assembly. **No concurrent `core.rs` edits.**
`depends_on: [RT-NATIVE-FNSPLIT]`; owner flipped foundation→runtime; Foundation stood
down (its elaborator slice `c07e63c2` is the preserved input). Steward kicks the
fast-follow (full handoff gate) **when RT-NATIVE-FNSPLIT merges**.

**Re-homed closure = M** (runtime-leader): take `c07e63c2`, add the `int_to_uint64_raw`
native lowering (identity-precedent arm on the signed-magnitude `RuntimeIntV1` carrier —
`lower_primitive_call` already treats `uint8_to_int`/`int_to_uint8_raw` as identity on
`Lowered::Int`), **run the exact native end-to-end until GREEN**, then the six-axis
matrix + controls + attestation/digest rider for touched native code.

**⚠ Diagnostic-staircase contingency (runtime-leader):** `int_to_uint64_raw` is NOT
asserted the final gap — the CAP fixture has revealed a new wall at each layer
(`MissingClosureMetadata` → `int_lit_outside_native_i64` → `int_to_uint64_raw`). The
acceptance is "full two-engine oracle GREEN," and any further native lowering gap the
exact fixture hits is surfaced/triaged, never worked around.

## Architect means confirmation (`evt_7xrcjp0apb4f1`) — shovel-ready for the fold

**`int_to_uint64_raw` is the sound closure of the exposed axis-(d) gap**, with a
**load-bearing constraint: it is NOT a machine `i64 -> u64` conversion.** Ken's
fixed-width carriers share the exact `Int` runtime representation; the interpreter
implements this raw narrowing as **value identity**. The native arm must:
- require exactly one `Lowered::Int` argument;
- return that same `Lowered::Int` **unchanged** — including the native-Int **tag
  sidecar** and payload/arena slot;
- preserve `18446744073709551615` as the existing **Big signed-magnitude** value;
- leave range admission to the derived checked `intToUInt64` wrapper (which proves
  `0 <= n <= u64::MAX` before calling the raw cast).

Extend the existing `uint8_to_int | int_to_uint8_raw` native arm (the representation-
level identity pattern). **Do NOT** use a Cranelift integer cast or an `i64` fast-path
that loses the Big arm — that would truncate/wrap/retag and is the failure mode.

**No further primitive gap expected on this route (Architect enumerated it):** the
checked closure's primitives are `leq_int, and_bool, int_to_uint64_raw, sub_int,
eq_int, add_int`; native already handles all but `int_to_uint64_raw`. `Some`/`None`,
handle construction/projection, and result branching are constructor/control lowering,
**not** primitives. ⇒ native code slice = **S**; the stop condition is retained only
for a **non-primitive constructor/effect** gap, not another primitive (do not
pre-inflate scope).

**Required focused discriminators (before the full oracle):**
1. `intToUInt64 u64::MAX` reaches `Some` natively, preserving the exact Big value/tag.
2. `intToUInt64 (u64::MAX + 1)` and `intToUInt64 (-1)` reach `None` — proving the
   checked **wrapper**, not the raw identity, owns admission.
3. The raw native arm and the runtime-IR/interpreter evaluator agree on representation
   identity; no wrap/truncation mutation survives.
4. Existing UInt8 conversion behavior is unchanged.

**Scope discipline:** this WP must **not** claim complete fixed-width-conversion
support if it adds only UInt64. A family generalization (the full representation-
sharing `IntN <-> Int` identity set) is **optional**, not required for CAP-41 GREEN,
and if taken must remain exact identity + be tested as a family (Small **and** Big
carriers), never an unreviewed wildcard over primitive names. The means confirm does
not waive the normal exact-SHA QA/CV/Architect gate on `c07e63c2` + the Runtime fold.

Discovered while implementing [[PX8-F-CAP-41]] Phase 2 (the sealed capacity-carrying
`BufferHandle`). The locked representation `data BufferHandle = PrivateBufferHandle
(Resource Buffer) Int` **does not lower on the native path** — the failure is raised
**before erasure and before Cranelift**, so it is a *distinct* native-completeness
gap from [[RT-NATIVE-FNSPLIT]] (which addresses the later single-Cranelift-function
`VReg::MAX` size wall). A ≤2-bracket program excludes the size wall.

## The defect (Architect-grounded, `evt_2zkjr68y1sdgf`)

On the exact preserved fixture `wp/PX8-F-CAP-41-p2-buffer-handle @ f0eb65ce`
(pushed to origin), native compilation fails inside the front half of the pipeline:
`compile_native_program_sources` builds the normalized checked-core package, then
`checked_computational_ih_templates` asks `checked_core_declaration_body_view` for
`main`; that body-view error is **collapsed by the driver to**
`Driver(MissingClosureMetadata { section: "checked computational IH authoritative
runtime body", symbol: [..., "main"] })`.

**The differentiating shape is not resource nesting.** The landed control
`BufferSpan = PrivateBufferSpan (Resource Buffer) Int Nat` already nests the same
`Resource Buffer` in a constructor-private data value and has native-reaching
coverage. The new handle differs by **crossing the higher-order `withBuffer`
body/normalization boundary**. So the fix is in the **compiler layer**, not the
checked representation — the representation is normatively locked by §38 and stands
(a token-only handle cannot supply the raw resource every public consumer needs and
would reopen the authority/ABI boundary Phase 1 deliberately closed).

**Decisive regression evidence:** a *pre-existing two-bracket* native read row
(`fs_read_at_malformed_offset_narrows_to_invalid_offset`), changed **only** by the
required `Resource Buffer -> BufferHandle` API migration, was GREEN before and now
fails before execution with the identical `MissingClosureMetadata`. This candidate
therefore *regresses an already-reachable native buffer program* — which is why
PX8-F-CAP-41 Phase 2 cannot land as a SPAN-PROV-style honest partial.

## First deliverable is diagnostic (do not pick a fix site from the wrapper)

The driver **erases** the underlying `CheckedCoreBodyViewError`, so the observed
`MissingClosureMetadata` does **not** identify the exact missing body-view lane. The
WP must first:
1. **Preserve/report the underlying `CheckedCoreBodyViewError` lane** (de-erase the
   driver collapse) so the exact failing body-view is visible.
2. **Isolation-flip the exact saved fixture** (`f0eb65ce`) against that de-erased
   lane. Only then choose the fix site.

## Acceptance matrix (Architect `evt_2zkjr68y1sdgf` — close every native axis)

- **(a)** normalized checked declaration body view for the higher-order handle path,
  with the underlying error lane visible;
- **(b)** computational-IH census / metadata consistency;
- **(c)** erasure of handle construction, match, and projections;
- **(d)** runtime constructor / value lowering **if the carrier survives
  deforestation** (contingency — see contention flag);
- **(e)** unchanged raw `Resource Buffer` host request and wire ABI;
- **(f)** constructor and both projections remain **absent from the public name map**.

**Controls (all must hold):** the migrated pre-existing two-bracket read row returns
GREEN again; all four CAP-41 rows are **absolute GREEN on both engines** with no
forbidden read/event/host; the landed `BufferSpan` product remains GREEN;
malformed / stale / closed authority behavior is unchanged.

## Fence & contention

- **Primary surface = `ken-elaborator`** (compiler-driver / checked-core / erasure)
  — disjoint from [[RT-NATIVE-FNSPLIT]]'s `ken-runtime`/Cranelift, so the two native
  WPs run **contention-free** as filed.
- **⚠ Axis (d) contingency:** *if* the fix requires `ken-runtime` constructor/value
  lowering, that would contend the concurrent RT track — **STOP and route to the
  Steward** to sequence it. Do not pre-assign the fix to `ken-runtime`/Cranelift on
  current evidence (Architect).
- **Local builds targeted only** (`scripts/ken-cargo -p <crate>`); never
  `--workspace` (COORDINATION §12). Full `-p ken-interp` if the reifier/value shape
  changes (attested `eval.rs` ⇒ OID-bump rider).

## What "done" unblocks

Once the carrier lowers, **fold the fix with the preserved
`wp/PX8-F-CAP-41-p2-buffer-handle @ f0eb65ce`** and run the full two-engine oracle;
[[PX8-F-CAP-41]] Phase 2 then lands **complete** (interp **and** native GREEN) — no
honest-partial, no operator scope exception. Sibling native-completeness WP:
[[RT-NATIVE-FNSPLIT]] (independent). Root gate: [[PX8]] (this is on the critical
path to PX8 clause-(a) closure via PX8-F-CAP-41).

## Sequencing (Steward)

**`active` 2026-07-23** — filed as the named prerequisite that unblocks
[[PX8-F-CAP-41]] Phase 2, Foundation-owned, replacing PX8-F-CAP-41 Phase 2 as the
Foundation track (the preserved `f0eb65ce` carries the handle/admission impl forward
for the eventual fold). Independent of [[RT-NATIVE-FNSPLIT]] (Runtime, Track 1),
which continues. Size **TBD** until the diagnostic step scopes the fix surface.
