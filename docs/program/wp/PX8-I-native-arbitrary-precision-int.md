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
> Chain position: **PX8-L ✅ → PX8-H ✅ → PX8-I (this) → PX8-F.**

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
authorization boundary. It deliberately **does not** dictate the cranelift
instruction sequence, the exact carrier handle, or helper-vs-inline lowering:
those are **implementer execution under Architect §14**. Restate the contract;
**do not design past it.** Any mechanism question the contract does not settle
(e.g. whether the widened carrier forces a Runtime-IR representation change,
how `Big` is threaded through a scalar join) is a **hard-stop to the Architect**
(`mentions` architect + leader), not an implementer guess — the PX8-H chain's
discipline: hard-stop with the exact value-kind/site, never widen or weaken.

## Mandated deliverables (each ends in a concrete implementable choice)

1. **Native `Int` value representation.** Widen the native scalar carrier
   (`RuntimeValue::Int` / `RuntimeGroundValue::Int` and every `Lowered` `Int`
   carrier) to `Small(i64) | Big(<canonical bignum handle>)` **bridging**
   `ken-foundation` `Value::BigInt`. **Choose the exact carrier** — reuse the
   foundation canonical type / store handle; do **not** introduce a parallel
   bignum type. State the chosen carrier and where `Big` values are interned.
2. **Arithmetic lowering.** For `add_int`/`sub_int`/`mul_int` over **dynamic**
   operands: `Small` fast-path via **overflow-checked** `i64` ops; on overflow,
   **promote before overflow** and compute exactly via the canonical model;
   **canonicalize `Big → Small` iff it fits.** Replace the `:1709` static-known
   refusal for these three ops. **State** helper-call (runtime libcall) vs inline
   emission and why.
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
- **Differential:** the unchanged PX8-F `c8b8cdb7` overlay's dynamic additions
  **agree with the interpreter**.

## Acceptance criteria (testable)

- `add_int`/`sub_int`/`mul_int`/`eq_int`/`leq_int` over dynamic operands lower
  and execute exact-Z; **every discriminator above is green**.
- The unchanged PX8-F `c8b8cdb7` `writeAll` overlay (run as a **throwaway
  overlay** — do **not** edit `c8b8cdb7` or the fixture) compiles/links/runs,
  **both** dynamic additions compute, reaches a **real `FsWriteAt`**, and the
  native run **agrees with the interpreter** (differential). This is PX8-I's
  downstream evidence; the H-P6/H-P6a obligation moves off PX8-H onto here + the
  PX8-F terminal.
- **No second numeric semantics:** the native path shares the `ken-foundation`
  canonical bignum model — `git grep` shows **no parallel bignum type** in
  `ken-runtime`.
- **No forbidden movement:** no kernel / host-ABI / wire / public-surface /
  trusted-primitive / `writeAll`-intrinsic / range-postulate / Cargo / spec /
  conformance change. *If* the carrier widening strictly requires a Runtime-IR
  representation change, that is itself a **hard-stop to the Architect**, not an
  implementer choice.
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
5. **Downstream:** PX8-I + PX8-H both landed → **unblock PX8-F** (rebase
   `c8b8cdb7` onto combined `main`; the unchanged `writeAll` fixture performs
   **real writes** in both interpreter and native lanes = PX8-F native evidence).
