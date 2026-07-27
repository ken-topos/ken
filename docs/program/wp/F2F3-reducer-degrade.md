# WP F2/F3 — reducer: degrade-not-wrap + retire legacy arms

**Phase-2 tranche #3 (BUILTINS PROVIDE). Team: Runtime. Base: `origin/main`.**
Steward frame. Cite ADR 0009 (capability-supply: bracket-the-untrusted;
wrong-value fix in the tested-not-trusted `ken-interp` ring, **not** a kernel
change). Kernel is untouched — this is the same posture as F1 (`bb40654`) and
Decimal/Char (`4eea2072`): a wrong *evaluator value* fixed, never a false proof.

## Objective

Close two ratified correctness defects in the interpreter's primitive reducer
(`crates/ken-interp/src/eval.rs`, `prim_reduce`), both already adjudicated
3-way in Phase-1 and specified in **`spec/10-kernel/18a-primitive-registry.md`
§5** (the F1–F5 severity+AC table, ~L138–160):

- **F2 — bare fixed-width silently wraps (runtime face non-compliant).** The
  bare `add_intN`/`sub_intN`/`mul_intN` and `add_uintN` arms reduce via
  `iN::wrapping_*` **unconditionally**, ignoring the no-overflow obligation —
  violating `35 §3` *undischarged ⇒ degrade (panic/`Unknown`), never wrap* and
  18a §5's degrade discipline. This is the **runtime face** of the overflow
  obligation; the **static face** (the `NoOvf` obligation emission, `22 §2.4`)
  already exists and is **not** in scope here.
- **F3 — legacy i64 `add`/`sub`/`mul` (unregistered, dead-but-live wrap arms).**
  `eval.rs:799–801` still reduce `("add"|"sub"|"mul",[Int,Int])` via
  `i64::wrapping_*`, and `eval.rs:1401` still gives them arity 2 — yet they are
  **unregistered** (no `reg_binop!`/`declare_primitive` in
  `crates/ken-elaborator/src/numbers.rs` — grep-confirmed empty). Dead-but-live:
  retire them.

## Code sites (verified on `origin/main` — grep to confirm, lines drift)

- **F2 arms:** `eval.rs` ~L745–760 — `("add_int8",[a,b]) => fixed_binop_i8(a,
  b, i8::wrapping_add)` … through `add_uint64`. These are the obligation class.
- **F2 helpers:** `eval.rs` ~L632–687 — `fixed_binop_i8` … `fixed_binop_u64`
  (`op: fn(iN,iN)->iN`, with a `_ => EvalVal::Neutral` fallthrough — the natural
  stuck target).
- **F2 modular carve-out — LEAVE AS-IS:** `eval.rs` ~L763+ —
  `("wrapping_add_int8",…)` etc. The sanctioned modular class is the **only**
  path that may wrap; `18a §5`/`35 §3` reserve `wrapping_*`/`+%`/`Wrapping[T]`
  to it. Do not touch these.
- **F3 arms:** `eval.rs:799–801` (header comment names them at ~L712) + the
  arity entry `eval.rs:1401` (`"add" | "sub" | "mul" => 2`).

## Hard ACs (each a gate)

1. **(F2, soundness/runtime-face)** Bare fixed-width `add/sub/mul_intN`,
   `add_uintN` reduce via **checked** arithmetic. On overflow they **degrade,
   never wrap** — the degrade face is a stuck `EvalVal::Neutral` (F4-consistent
   "exact-or-stuck: a missing value, never a wrong one") **or** a loud panic per
   `35 §3`; **the implementer proposes the exact degrade face and the Architect
   rules it.** The invariant that gates: for overflowing operands the arm does
   **not** yield the wrapped value.
2. **(F2, discriminating — use the EXISTING oracle)** Wire the already-authored
   `seed-numbers.md` cases **AC3** (`+ : Int32` undischarged→degrade vs
   discharged→total) and **AC4** (bare `+` vs explicit `+%` on the *same*
   overflowing operands) as executable conformance tests and make them green.
   These are non-degenerate pairs authored **independently** of this build (the
   expected "degrade, not the wrapped value" comes from the spec, not the impl)
   — so this is **not** green-vs-green. The pair must **flip**: bare-op degrades
   WHILE `+%` on the same operands still wraps to the modular value.
3. **(F2, non-regression)** The sanctioned modular class
   (`wrapping_*_intN`/`+%`) still wraps — byte-unchanged behavior; a case pins
   it.
4. **(F3, retire)** Delete the `eval.rs:799–801` legacy arms and their arity
   entry (`eval.rs:1401`). Guard-test proves both faces: (a) **unregistered** —
   no `declare_primitive` mints `"add"/"sub"/"mul"` (grep the elaborator); (b)
   **unreduced** — `prim_reduce("add",[Int,Int])` returns `EvalVal::Neutral`
   after deletion. Both halves, or a live latent wrap survives.
5. **(soundness, whole-WP)** **Kernel diff empty** — `git diff --stat
   crates/ken-kernel/` returns nothing; `trusted_base()` unchanged (F2/F3 are
   interp-ring reductions still left STUCK by the kernel, `PrimReduction::Op`
   awaiting K3 — no promotion here). **Workspace-green landing** (K7 discipline:
   QA re-runs `./scripts/ken-cargo test --workspace` independently, not
   implementer-trusted).

## Oracle discipline (18a §3)

F2's oracle is the **boundary operand** (the overflow edge) against the spec's
degrade rule — a defining behavior, non-circular. F3's oracle is the
**guard-test** (unregistered ∧ unreduced). Neither aliases the native path.

## Out of scope / defer (verify by absence, like decimal-char)

- **F5** (`leq_int` reduce arm) — already landed with Decimal/Char (the
  `leq_int` pull-up, `4eea2072`); **not** re-touched here.
- `div_int`/`mod_int` runtime-obligation face, `neg_int` demote, the 6
  conversions — later tranches (18a §5 GAP rows).
- The static `NoOvf` obligation face — already delivered; F2 is the runtime
  face only.

## Flow (thin — COORDINATION §9)

`runtime-leader → runtime-implementer → runtime-qa → Architect (soundness) + CV
(conformance) → Steward`. One pass each. A mid-WP soundness fork → Architect;
a conformance fork → CV; a scope/lane fork → Steward. No new parties, no
verbatim relays, no cc-the-room.
