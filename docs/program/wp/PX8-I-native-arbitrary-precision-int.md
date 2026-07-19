# PX8-I — native arbitrary-precision Int lowering

> **Runtime-owned prerequisite for PX8-F, opened by Architect ruling
> `evt_1mhavqy6yy128` (2026-07-18 20:52 — the 17th PX8-H ruling).** PX8-H banked
> the heterogeneous-continuation machine and, per that ruling, the Int-arithmetic
> gap was **split out** of PX8-H into this new WP. PX8-H's H-P6a overlay reaches
> *exactly* the dynamic `add_int` boundary — the native lowering refuses dynamic
> Int arithmetic with `"{symbol} requires statically known non-overflowing Int
> operands in native lowering"` (`crates/ken-runtime/src/cranelift_backend.rs`,
> the `lower_int_binop` static-known guard). PX8-I completes native `Int` at the
> value-representation + primitive-lowering layer so that boundary computes exact
> mathematical **Z**, not wrapping `i64`. It is the last Runtime prerequisite
> before PX8-F.
>
> Chain position:
> **PX8-L ✅ → PX8-H ✅ → PX8-I (this) → PX8-J → PX8-F.**
>
> **⚠ Overlay-AC deferral (Architect §14, `evt_2gs0zfg66x9qk`; reconciled
> `evt_2v6x1rs70c06g`).** PX8-I does **not** directly unblock PX8-F. Running the
> unchanged PX8-F `c8b8cdb7` overlay clears PX8-I's dynamic-`add_int` boundary
> but then stops at a **distinct** continuation defect (`ComputationalMatch:
> scrutinee is not a constructor value` — process-object scalarization mints
> `ProcessExitStatus` while a computational eliminator is still live), which the
> Architect split into the new prerequisite **PX8-J** (terminal-answer
> boundary). The overlay's
> **real-`FsWriteAt` + interpreter/native differential** is therefore
> **PX8-J's** acceptance, **not** PX8-I's. PX8-I's acceptance is its
> **exact-Int mechanism + direct green gates** (below).
> PX8-I → PX8-J → PX8-F.

## Objective

Native lowering computes `Int` add/sub/mul/eq/leq over **dynamic** operands in
exact mathematical **Z**, by **bridging the already-landed canonical bignum
store model** (not a second numeric semantics), so that the unchanged PX8-F
`writeAll` fixture's dynamic `add_int file_offset count` reaches a real
`FsWriteAt` and the native run **agrees with the interpreter run**.

Owner: **Runtime** (L/I/Q `agt_37reqrd72cg00` / `agt_37reqg3nync00` /
`agt_37reqvb6ce400`). Size **M**, Risk **Medium** (representation change crosses
the native scalar surface; the discipline is bridge-don't-reinvent + a complete
exhaustive-consumer ledger). Route **Architect §14 + Runtime QA**; **no CV lane**
(no spec/conformance movement).

## Fixed inputs — DO NOT REOPEN (Architect ruling `evt_1mhavqy6yy128`; settled)

These are the **contract**, ruled by the Architect. They are not open for the
implementer to relitigate; a mechanism question *beyond* them is a hard-stop to
the Architect, never an implementer choice.

1. **Representation.** `Native Int = Small exact i64 | Big canonical
   content-addressed integer`. `Small` is a **fast-path only**; the *semantics*
   are exact Z at both arms. A `Big` value must never be observable as a machine
   scalar.
2. **Arithmetic.** `add`/`sub`/`mul` operate **exactly**. The `Small` fast-path
   is taken **only when the exact result fits `i64`**; otherwise **promote before
   overflow** and compute in `Big`. **Canonicalize `Big → Small` iff the result
   fits `i64`.** No `i128` ceiling — real arbitrary precision.
3. **Comparison.** `eq`/`leq` are **exact across all four `Small`/`Big`
   combinations**.
4. **Host narrowing.** Every `Int → host-width` conversion (file offset, byte
   count, bounds) is a **checked range conversion** to the host's target unsigned
   width. Out-of-range (negative, or greater than the target width max) yields
   the **consuming op's** `InvalidOffset` / `InvalidBounds` — **never** truncation
   or wrap.
5. **Reuse the landed model.** PX8-I **must reuse/bridge the landed canonical
   bignum/store model** (`ken-foundation` `Value::BigInt { sign, limbs }`). **No
   second numeric semantics.** `known: Option<i64>` stays an **optimization hint
   only** — never the source of truth for a value.
6. **Differential oracle.** The native result must **agree with the
   interpreter's** arbitrary-precision semantics (`ken-interp` `EvalVal::Int |
   BigInt`, `bigint_to_int_val` narrowing). The interpreter is the reference.
7. **Authorization boundary.** **No** new kernel rule, **no** public surface op,
   **no** trusted primitive, **no** `writeAll` intrinsic, **no** range-postulate.
   Runtime-owned **native lowering + value-representation** layer only. (The
   checked layer already *types* `Int` as arbitrary-precision; PX8-I only lowers
   it natively.)

### Mechanism correction (Architect ruling `evt_4dq6xdw4m87nn`)

The pre-edit grounding proved that the former one-word native carrier could not
injectively represent both every `i64` and a `Big` identity. PX8-I therefore
owns the necessary Runtime-private representation change:

- every compiler-private Int edge carries `NativeIntV1 { tag, payload }` as two
  native words; `Small` stores the exact signed `i64` bits and `Big` stores a
  nonzero typed slot in the current invocation's `NativeIntArenaV1`;
- `RuntimeValue::Int` and `RuntimeGroundValue::Int` use one nested
  `RuntimeIntV1 = Small(i64) | Big { sign, minimal little-endian limbs }`, byte
  compatible with the production `ken-runtime::Value::BigInt` image and the
  `ken-foundation` design-validation oracle;
- exactly one Runtime-owned arena is shared by all generated calls in an
  invocation and lives through final decoding. Slots never enter Ken values,
  host requests, traces, wire artifacts, or terminal observations;
- every Int CFG join, declaration/invocation return, recursive backedge,
  closure capture, SourceControl edge, and decoder transports both words; and
- exact arithmetic/comparison and checked narrowing are local functions emitted
  into the same Cranelift module/object. They receive the hidden arena pointer;
  they are not `HostOpV1`, `ken_host_dispatch_v1`, a public ABI, or a dynamic Ken
  bignum service.

Every tag and slot lookup validates fail-closed. The external process
`ExitCode` ABI remains scalar only after the compiler-private Int has been
resolved and narrowed while its arena is still alive.

### Local-helper boundary correction (Architect ruling `evt_7f86e5mfjdcph`)

The exact integer semantics are one Cranelift-local helper graph emitted by one
module-construction routine into every JIT and object module. The entry function
calls only local `FuncId`s for tag/slot resolution, canonical interning,
add/sub/mul, comparison, checked narrowing, and terminal export. JIT callbacks,
undefined `ken_runtime_native_int_*` symbols, and header-compiled C arithmetic
bodies are forbidden.

Only ordinary allocator imports (`malloc` / `free`) may remain. JIT setup and
both C starters own a zeroed, C-compatible invocation arena header and free its
raw allocations after terminal decoding; they implement no integer semantics.
Construction evidence compares the normalized helper CLIF emitted for fresh
JIT and object modules. Both generic and process linked artifacts must execute
the same Big path, and a shared-helper mutation must turn JIT and object
discriminators red.

## Landed anchors (verify before editing; do not trust frozen line numbers)

Re-grep each; the line numbers below are orientation, not contract.

- **Canonical bignum store image (bridge target):**
  `crates/ken-foundation/src/values.rs` — `Value::BigInt { sign: Sign, limbs:
  Vec<u64> }`. Canonical byte encoding + minimal form:
  `crates/ken-foundation/src/canonical.rs` (`minimal_limbs` strips trailing zero
  limbs → the content-addressed minimal representation). Spec:
  `docs/design/content-addressing.md §1.1`, arbitrary-precision Int semantics
  `spec` `18a §5.2.1` (narrowing never wraps: `18a §5.2.1(1)`).
- **Interpreter reference semantics (the oracle to match):**
  `crates/ken-interp/src/eval.rs` — `EvalVal::Int(i64) |
  EvalVal::BigInt(num_bigint::BigInt)`; `bigint_to_int_val` (≈`:865`) is exactly
  the **narrow-`Big`→`Small`-iff-fits, never-wrap** rule; the `EvalVal::BigInt →
  RtValue` bridge is `bigint_to_rt` (≈`:266`, and `EvalVal::Int → RtValue::SmallInt`
  ≈`:265`). Mirror these; do not diverge.
- **Native `i64`-only surface to widen:** `crates/ken-runtime/src/ir.rs` —
  `RuntimeValue::Int(i64)` (≈`:394`), `RuntimeGroundValue::Int(i64)` (≈`:421`),
  and every `Lowered` scalar carrier that currently assumes `i64`.
- **Arithmetic lowering:** `crates/ken-runtime/src/cranelift_backend.rs` —
  `lower_int_binop` dispatch for `add_int`/`sub_int`/`mul_int` (≈`:1609`); the
  **H-P6a static-known refusal** `"… requires statically known non-overflowing
  Int operands in native lowering"` (≈`:1709`) — the boundary this WP replaces
  for dynamic operands.
- **NC-certificate / static-fact validation path for `add_int`:**
  `crates/ken-runtime/src/artifact_validation.rs` (≈`:779`, arity + literal-Int
  operand-shape facts) and `cranelift_backend.rs` (≈`:2356`). Keep these
  consistent with the new dynamic path (a dynamic operand must no longer be the
  only supported shape's negation).

## Mechanism as ruled (authoritative; by citation)

Ruling `evt_1mhavqy6yy128` fixes the **contract** in "Fixed inputs" above —
representation, op semantics, narrowing, reuse-canonical-model, and the
authorization boundary. Architect ruling `evt_4dq6xdw4m87nn` additionally fixes
the two-word carrier, invocation arena, two-word joins/returns, local emitted
support functions, and pre-dispatch narrowing above. Instruction details remain
**implementer execution under Architect §14**. Any further mechanism question
the two rulings do not settle is a **hard-stop to the Architect** (`mentions`
architect + leader), not an implementer guess.

## Mandated deliverables (each ends in a concrete implementable choice)

1. **Native `Int` value representation.** Widen the native scalar carrier
   (`RuntimeValue::Int` / `RuntimeGroundValue::Int` and every `Lowered` `Int`
   carrier) to `Small(i64) | Big(<canonical bignum handle>)` **bridging**
   `ken-foundation` `Value::BigInt`. **Choose the exact carrier** — reuse the
   production canonical image; do **not** introduce a parallel bignum type. Use
   the ruled two-word carrier and invocation arena and state where `Big` values
   are interned.
2. **Arithmetic lowering.** For `add_int`/`sub_int`/`mul_int` over **dynamic**
   operands: `Small` fast-path via **overflow-checked** `i64` ops; on overflow,
   **promote before overflow** and compute exactly via the canonical model;
   **canonicalize `Big → Small` iff it fits.** Replace the `:1709` static-known
   refusal for these three ops. Call the ruled Cranelift-local helper graph;
   imported runtime callbacks and separately authored C bodies are forbidden.
3. **Comparison lowering.** `eq_int`/`leq_int` **exact** across all four
   `Small`/`Big` operand combinations.
4. **Host narrowing.** A **single** checked conversion at **every** `Int →
   host-width` consumer (offset / count / bounds): out-of-range → the op's
   `InvalidOffset` / `InvalidBounds`, never wrap. **Enumerate the consumers**
   and route each through the one checked conversion.
5. **Producer/consumer audit (Int scalar surface).** Audit **every** native
   `Lowered::Int` producer, consumer, scalar-join, call-return, host-narrow, and
   final decoder so a `Big` value can never be misread as a machine scalar.
   Deliver the **audited-site ledger** (the PX8-H consumer-edge ledger form).
6. **Cross-crate exhaustive-consumer closure (the PX8-H CI-red lesson).** After
   widening the Int carrier, derive the **whole-test-tree** exhaustive-matcher
   ledger (`git grep -nE 'RuntimeValue::|RuntimeGroundValue::|match .*Int'
   crates/*/tests crates/ken-cli/tests` and the lib crates) and **gate every
   classified consumer** — a targeted `--lib` + one-suite gate will miss a second
   exhaustive integration consumer, and **only full-workspace CI catches it**
   (this cost PX8-H a CI-red respin; see `dec_52wt6cya1w7ye`). No broad `_` arms.

## Required proofs / discriminators (each independently reaching)

From the ruling's discriminator set — each must be a distinct, reaching test:

- `Small ± Small` **crossing the `i64` boundary** promotes to `Big` (e.g.
  `i64::MAX + 1`).
- `Big ± Small` and `Big × Big` **beyond `i128`** compute exactly (no `i128`
  ceiling).
- `eq`/`leq` on **equal-low-64, different-value** operands compare **not equal**
  (a naive low-word compare must fail this).
- **Mutation control:** a wrapping `iadd` (wrap-on-overflow) makes a
  discriminator **fail** — proves the fast-path is overflow-checked.
- **Mutation control:** a `Small`-overflow **trap** (instead of promote) makes a
  discriminator **fail** — proves promotion, not trapping.
- `Small`/`Big` **round-trip content identity**: a `Big` that fits canonicalizes
  to `Small` and back to the same canonical value.
- **Scalar-join / invocation-return preserve full `Int`**: a `Big` flowing
  through a PX8-H join/return is not truncated (reuses PX8-H's join/return
  surface).
- **Checked narrowing rejects** a negative value and a value `> u64::MAX` (or the
  exact host width) with the op's `InvalidOffset`/`InvalidBounds`, not a wrap.
- **Local-helper identity:** normalized CLIF for the complete helper graph is
  identical for JIT and object construction. Generic and process objects have
  no undefined `ken_runtime_native_int_*` symbols; only allocator imports (and
  the already-authorized process host dispatch) remain.
- **Three-lane reachability:** a generic object and a process object each
  execute a genuinely Big intermediate beyond `i128`; the same helper mutation
  makes both the JIT and linked-object discriminators fail.

## Acceptance criteria (testable)

- `add_int`/`sub_int`/`mul_int`/`eq_int`/`leq_int` over dynamic operands lower
  and execute exact-Z; **every discriminator above is green**.
- **Overlay real-write/differential is DEFERRED to PX8-J — NOT a PX8-I AC**
  (Architect `evt_2gs0zfg66x9qk`; reconciled `evt_2v6x1rs70c06g`). The unchanged
  PX8-F `c8b8cdb7` `writeAll` overlay (run as a **throwaway overlay** — do
  **not** edit `c8b8cdb7` or the fixture) must clear PX8-I's dynamic-`add_int`
  boundary and **both** dynamic additions must compute exact-Z; it then stops at
  the distinct process-object continuation defect owned by **PX8-J**. Reaching a
  **real `FsWriteAt`** and the interpreter/native differential are **PX8-J's**
  acceptance. PX8-I's acceptance stops at: the overlay clears `add_int` and
  computes both additions correctly (overlay surface 2/2), plus all direct
  exact-Int gates above.
- **No second numeric semantics:** the native path shares the `ken-foundation`
  canonical bignum model — `git grep` shows **no parallel bignum type** in
  `ken-runtime`.
- **No forbidden movement:** no kernel / host-ABI / wire / public-surface /
  trusted-primitive / `writeAll`-intrinsic / range-postulate / Cargo / spec /
  conformance change. The ruled two-word Runtime-private Int carrier is the one
  authorized Runtime-IR representation change; no other representation or ABI
  widening is implied.
- **Whole-test-tree exhaustive-consumer ledger complete**; every classified
  integration suite gated. **Targeted local builds only** (`scripts/ken-cargo -p
  <crate>`; never `--workspace`); "workspace-green" / no-regression means
  **green in CI**.

## Do-not-reopen guard

- The representation (`Small | Big`), **promote-before-overflow**,
  **canonicalize-iff-fits**, **narrow-never-wraps**, **reuse-canonical-model**,
  and the **authorization boundary** are Architect-ruled fixed inputs. Do **not**
  relitigate to `i128`, saturating arithmetic, or a runtime-`Big`-only model.
- Do **not** add a public surface op or kernel rule for bignum — the checked
  layer already types `Int`; PX8-I is **native lowering only**.
- Any mechanism gap **beyond** the contract → **hard-stop to the Architect**
  (`mentions` architect + leader) with the exact value-kind/site — never an
  implementer guess, never a widened/weakened match.

## Sequencing

1. Branch `wp/px8i-native-arbitrary-precision-int` off **`origin/main`** (the
   fetched ref, which must include PX8-H = `aab1f831` or later — verify
   `merge-base` == current `origin/main`).
2. **Handoff-gate the Runtime team** first (§2c): all three PX8-H retros in →
   compact L/I/Q → verify the drops → kick with this brief cited.
3. Implementer builds → Runtime QA → Runtime leader creates **one Decision** →
   **Architect §14** (resolve-on-cast). No CV lane.
4. On resolved APPROVE → **publisher non-doc-only** (crates → CI must go green,
   **including full-workspace exhaustiveness**) → verify byte-identical landing +
   **PIN** → chase retros → close the WP.
5. **Downstream:** PX8-I landed → **PX8-J** (continuation terminal-answer
   boundary; `docs/program/wp/PX8-J-terminal-answer-boundary.md`) — PX8-J owns
   the overlay real-write/differential and, once landed, **unblocks PX8-F**
   (rebase `c8b8cdb7` onto combined `main`; the unchanged `writeAll` fixture then
   performs **real writes** in both lanes = PX8-F native evidence). PX8-I does
   **not** directly unblock PX8-F. Sequence: PX8-I → PX8-J → PX8-F.
