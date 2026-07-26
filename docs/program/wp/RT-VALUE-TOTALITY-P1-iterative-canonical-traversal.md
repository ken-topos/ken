# `RT-VALUE-TOTALITY-P1` — the iterative canonical traversal

**Node:** `docs/program/issues/RT-VALUE-TOTALITY.md` · **Owner:** Team Runtime ·
**Size:** M · **Branch:** `wp/RT-VALUE-TOTALITY-P1` (cut fresh from
`origin/main`) · **Base:** see the kickoff message for the exact SHA and blob.

> ## ⛔ THIS IS PHASE 1 OF TWO, AND IT IS THE TOTALITY HALF ONLY
>
> Phase 1 makes the **existing** `Value` traversals non-recursive in the host
> stack. It does **not** touch the carrier split, the derive list, the closure
> arm, or `ken-foundation`. ⛔ **Do not remove `Value::Closure` in this WP** —
> that is Phase 2, its frame is the Steward's and is not yet written, and
> attempting it here makes this diff unreviewable.
>
> **Why this order.** Phase 2's checked projection must be **iterative and share
> this phase's mechanism** (node §3b pin 3 — *"no recursive adapter"*). So the
> mechanism has to exist first, or Phase 2 has nothing to share and will grow its
> own recursive traversal — the exact defect one layer out.
>
> **Why it is the critical path.** `RT-FNSPLIT-B2V` acceptance is blocked on deep
> **acyclic** adoption completing without host-stack growth. That is `AC-V1`
> below, and nothing else in the node is on B2V's path.

## 1. Fixed inputs — settled, ⛔ do not reopen

| input | object | what it settles |
|---|---|---|
| cycle contract | `evt_5pzxf6sm4z08` | host recursion may **not** be the totality mechanism; a deep **acyclic** chain adopts **without host-stack growth** and is **not** reclassified as malformed |
| closure boundary | `dec_3b1r19v59v20y`, landed `SPEC-CLOSURE-BOUNDARY` | ordinary closures are runtime-local and opaque — **Phase 2's** subject, not yours |
| carrier split | `dec_1dckq8c0f9xjv` (node §3) | the canonical/operational split and its five pins — **Phase 2's** subject |

⛔ **A depth-limit rejection does not discharge this WP.** The ruling requires a
deep acyclic chain to **succeed**. A `MAX_DEPTH` that returns a clean error is a
different (and forbidden) answer, and a reviewer reading only "no overflow" cannot
tell the two apart.

⚠ **Every "current state" claim in §2 is PERISHABLE.** Verify each against the
landed code at pickup. If a fixed input is false against the code, **say so and
escalate — do not quietly build around it.** Two seats caught a false base of
mine on 2026-07-26 by doing exactly that; it is the behaviour I want.

## 2. Measured substrate — measured at `origin/main = 7415dbd8`

### 2a. The recursion is in ONE function, and the store path is FLAT

`crates/ken-runtime/src/canonical.rs`, `impl Canonical for Value` at `:72`:

| line | variant | recurses on |
|---|---|---|
| 109 | `Value::Constructor` | each of `args` |
| 119 | `Value::Record` | each of `fields` |
| 147 | `Value::Array` | each of `elements` |
| 164 | `Value::Map` | each entry **value** (keys are already canonical bytes) |
| 190 | `Value::Closure` | each of `captured` |

⭐ **`store.rs:230 intern()` does NOT recurse and does NOT intern children.** It
calls `encode_canonical` **once** on the whole value and hashes the resulting
bytes (`fnv1a_64`). ⇒ **The entire deep-traversal surface of the adoption path is
this one function.** (`probe_or_insert` self-recurses once after a resize —
bounded by capacity, **not** by value depth. Out of scope.)

⭐ **And encoding is a STREAMING PRE-ORDER APPEND, not a postorder fold.** Each
arm writes its own tag/ids/arity into `out`, then children append after it. A
parent's bytes never depend on a child's bytes. ⚠ **The ruling's phrase
"postorder canonicalization" describes a shape this encoder does not need** — do
not build a postorder machine here. (`Clone`, in `D3`, *is* postorder. They are
different traversals; see §3c.)

`Value::Set` holds `BTreeSet<Vec<u8>>` — canonical bytes, no child values. Only
`Map`'s entry **values** are children.

### 2b. There is no depth guard, and the failure mode ABORTS THE PROCESS

`MAX_DEPTH` / `depth_limit` / `recursion_limit` / `worklist` / `tri-colour` all
return **nothing** in `canonical.rs` and `values.rs`. A deep acyclic `Value` does
not fail closed — it **overflows the host stack**, and a Rust stack overflow may
**abort the process** rather than unwind. ⚠ That is why several controls below
must run **out of process**: an in-process assertion cannot distinguish *"the
guard fired"* from *"the binary died."*

### 2c. `Value` is a PURE OWNED TREE — no indirection anywhere

`crates/ken-runtime/src/values.rs`, all 117 lines:

```
grep -nE "Rc<|Arc<|RefCell|Cell<|Box<|\*const|\*mut|SlotId|unsafe" values.rs
  -> NO MATCH
```

Every compound child is owned by value (`Vec<Value>`,
`BTreeMap<Vec<u8>, Value>`). ⚠ **What follows from this is HELD on the Architect
— see §5. Do not act on your own reading of it.**

### 2d. The test target you need does not exist yet, and the API reaches it

- **`crates/ken-runtime/tests/` does not exist.** All `ken-runtime` tests are
  in-crate `#[cfg(test)]`. The out-of-process controls need a new integration
  target — you are creating `crates/ken-runtime/tests/`.
- ✅ **The public API is sufficient for it:** `lib.rs:38 pub use
  canonical::Canonical` and `lib.rs:66 pub use values::{Sign, Value}`, and
  `pub trait Canonical` with a public `encode_canonical`. An integration test can
  construct a deep `Value` and encode it **without any new `pub`**. ⛔ Do not
  widen visibility to test this; the seam is already cut.

## 3. Deliverables

### D1 — ONE iterative canonical encoder, byte-identical to today

⛔ **Keep the public signature `fn encode_canonical(&self, out: &mut Vec<u8>)`
unchanged.** It is `pub`, re-exported, and consumed by `ken-interp` and
`ken-foundation`. This WP is not an API change.

Replace the five recursive calls with a single non-recursive driver over an
explicit work stack. The concrete shape, since the encoder streams:

```rust
enum Step<'a> {
    Val(&'a Value),   // emit this value's own header, then push its children
    Raw(&'a [u8]),    // emit len-prefix + these already-canonical bytes
}
```

- `Step::Val(v)` — emit exactly the header bytes the landed arm emits (tag, ids,
  arity/length), then push children **in reverse** so they pop in declaration
  order.
- `Step::Raw(b)` — needed only for the `Map` key that must precede each entry
  value. `Set` elements are emitted whole in the header step.
- Scalars and `String`/`Bytes` push nothing.

⛔ **`minimal_limbs` (`:62`) and the NFC normalization (`:123`) stay exactly where
they are and keep behaving identically.** They are encoder-time normalizations,
they are **known** to disagree with the derived `Eq`/`Ord`/`Hash` (node §3c), and
that disagreement is **Phase 2's** to resolve. ⛔ Do not "fix" it here and do not
change the derive list.

### D2 — the cycle face — ⛔ HELD, see §5

Not releasable until the Architect rules. ⚠ **It does not block D1 or D3** —
start there.

### D3 — `Clone` and DROP are total at the same depth

`values.rs:10` derives `Clone`; automatic **drop glue** recurses through the
nested `Vec<Value>` / `BTreeMap<_, Value>` owners. ⛔ **Drop cannot return an
error**, so a total encoder does not make deallocation total: a value shallow
enough to construct can overflow while being **dropped**.

#### ⚠ D3 IS COUPLED TO D1's TEST, AND THIS WILL BITE YOU FIRST

**You cannot construct a deep `Value` in a test without also dropping it.**
Construction in a loop is iterative and fine; **teardown is not.** So a naïve
`AC-V1` test dies in teardown with the encoder working perfectly, and the
symptom looks like an encoder failure.

⇒ Either (i) isolate the encode face with `ManuallyDrop` / `mem::forget` (leaking
in a test process that is about to exit is acceptable and should be **commented
as deliberate**), or (ii) land D3 before D1's deep test. ⛔ Do not debug a
teardown overflow as an encoder bug.

#### The mechanism choice is yours, and a MEASUREMENT selects it

⚠ **A manual `impl Drop for Value` makes `Value` non-partially-movable
(`E0509`):** any `match v { Value::Constructor { args, .. } => args }` that moves
a field **out** of a `Value` stops compiling.

⚠ **My grep found no by-value `Value` parameters in `crates/ken-runtime/src`, and
that grep is NOT exhaustive** — it misses multi-line signatures, `mut` bindings,
and `ken-interp` / `ken-foundation` / `cranelift_backend`. **Treat it as an
estimate, not a measurement.** `AC-V3a` requires you to measure it properly
before choosing.

Two families; pick with the measurement in hand and **state which and why**:

1. **`impl Drop for Value`** with an explicit dismantling stack — `mem::replace`
   each child with a leaf variant, push onto a `Vec`, drop iteratively. Cost:
   the `E0509` movability constraint above, across every consuming crate.
2. **Iterative drop on the child CONTAINER**, not on `Value` — a newtype wrapping
   the child collections carries the `Drop`, leaving `Value` itself `Drop`-free
   and partially-movable. Cost: it changes public field **types**, so
   `ken-interp` and `ken-foundation` construction sites move with it.

⛔ **Neither cost is a blocker and neither is free. Do not pick on taste — pick
on the measured population and report both numbers in the handoff.** If the
measurement makes both look wrong, that is a hard-stop with evidence, not a
reason to leave drop recursive.

`Clone` is the one **postorder** traversal here: assemble with an explicit stack
of pending parent frames plus a completed-children buffer. ⚠ It is not the same
machine as D1's streaming emitter; do not force them into one.

## 4. Acceptance criteria

⛔ **Each face gets its own isolated control. Bundling them means one control's
green is read as covering mechanisms it never exercised.**

⛔⛔ **NAME THE OPERAND ON EVERY CONTROL.** A control has two operands — the
**detector** and the **population it is claimed to reach**. Mutating the detector
proves it is wired to *something*; only mutating the **population** proves
**reach**. On `KW-ORACLE-CLOSURE` (2026-07-26) a detector-side mutation was run
in place of the frame's population-side one, it reddened, and the report *"each
control reddened its intended named test"* was **literally true** while the
defect the WP existed to fix sat under a green control. Per AC below, **record
which side you moved.** See `agent/playbooks/tools/pin-a-property.md §10`.

### `AC-V1` — deep ACYCLIC encoding succeeds, with a POSITIVE byte oracle

1. **Establish `D` by measurement, and record it.** Bisect the depth at which the
   **landed recursive** encoder overflows, out of process. Record the measured
   threshold **and** the `D` you choose above it.
   ⛔ **A `D` you picked because it "seems deep" is not evidence.** If `D` never
   reaches the old limit the test passes without exercising anything, and the
   green means nothing.
2. **The old encoder must be SHOWN to die at `D`**, out of process, asserting on
   the **process outcome**. ⭐ This is the population-side proof that the
   population is adequate — it is the load-bearing control of this AC, not a
   nicety.
3. **The new encoder at `D` must produce EXACTLY the closed-form expected
   bytes.** A unary chain has a predictable encoding — `D` repetitions of
   `[tag::RECORD, type_id_le, arity=1_le]` then the leaf. Assert the **whole byte
   string**.
   ⛔ *"Completed without overflowing"* is a **negative** check that passes for
   any reason, including an encoder that emitted nothing. Assert the positive.
4. ⛔ Not discharged by a depth-limit rejection, and ⛔ not by reclassifying the
   value as malformed.

### `AC-V1b` — the encoding is byte-identical to today for everything else

Freeze a `#[cfg(test)]` copy of the **pre-change recursive** encoder as a
reference and assert **byte equality** across a corpus covering every variant,
both compound orderings, empty and non-empty collections, nested `Map` values,
and `Value::Closure` captures (still present in Phase 1).

- **Operand for the control: the SUBJECT (the new emitter), not the detector.**
  Perturb the new emitter — drop one arity prefix, reorder two children — and the
  differential **must** redden.
- ⚠ The reference copy cannot run at `AC-V1`'s `D` (it overflows by
  construction). **This corpus is shallow-to-moderate on purpose**; `AC-V1`'s
  step 3 is what covers depth. Say so where the corpus is defined, so a reader
  does not read this AC as covering `D`.

### `AC-V2` — the cycle face. ⛔ HELD, see §5

⛔ **Do not invent a control for this before the ruling lands.** With the
population question open, the only available control is detector-side, which is
exactly the substitution named at the head of §4.

### `AC-V3` — `Clone` and DROP are total at `AC-V1`'s `D`

**`AC-V3a` (do this FIRST — it selects the mechanism).** Measure the by-value
partial-move population of `Value` across **every** consuming crate — at minimum
`ken-runtime`, `ken-interp`, `ken-foundation`. Report the count and the sites.
⛔ Do not choose between D3's two families before this number exists.

**`AC-V3b`.** At depth `D`: clone the value, then drop **both** copies, out of
process, asserting on the **process outcome**.

**`AC-V3c` — population-side positive control.** The **landed** derived
`Clone` / drop glue must be shown to die at `D`, out of process. ⭐ Without it,
`AC-V3b` passing is compatible with `D` being too shallow to have ever mattered.

**`AC-V3d`.** Drop specifically, isolated from clone — a constructed-then-dropped
value at `D` with no encode and no clone in the test body. Drop cannot signal
failure, so it needs its own arm.

### `AC-V4` — the new test target is enumerated against every corpus oracle

You are creating `crates/ken-runtime/tests/`. ⚠ **A new file in a globbed
location must satisfy every corpus-wide oracle, and those live in crates this WP
never touches — targeted per-crate validation cannot see them, so they surface as
red CI at publish, after review.**

Grep for every test that enumerates test targets or crate test files, **name each
one in your handoff**, and state for each whether it binds here. ⛔ *"The
formatter gate"* is rarely the only one. If the answer is genuinely "none bind,"
**write that sentence explicitly** — a silent absence and a checked absence read
identically to a reviewer.

### `AC-V5` — per-pin evasion attempt, ENUMERATED

For **each** of `AC-V1` step 3, `AC-V1b`, `AC-V3b`, `AC-V3c`, `AC-V3d`: attempt a
**compile-preserving** evasion that satisfies the assertion while violating the
property, and record the result **per AC, in a table, one row each**.

⛔ **Not "each pin" as a quantifier you resolve** — five named rows. A per-pin
reminder without enumeration gets satisfied by the most salient control and
silently skips the rest; that is measured, on `RT-FNSPLIT-B2O`.

## 5. ⛔ THE ONE HELD FACE — routed, not open for you to settle

`evt_cp65d0f7rwwe` asks the Architect **one** question: does the cycle clause bind
on `ken-runtime::values::Value`, where §2c measures that no sharing, interior
mutability, indirection, or slot handle exists in the type — or only on a carrier
that can hold a cycle?

⛔ **Do not answer it, do not build either branch, and do not treat §2c's
measurement as its answer.** The measurement is mine and it is sound; the
**inference** from it is the Architect's, and I have explicitly asked it to
discount mine. The frame will be amended with the ruling **before** this WP is
released, and an amendment that is not on a fetchable ref has not happened.

Second-order, ruled in the same turn: whether `AC-V1` needs **any** depth limit.
⛔ Until that lands, **build no `MAX_DEPTH`.**

## 6. Validation — ⛔ TARGETED ONLY

```sh
source scripts/ken-env.sh
scripts/ken-cargo test -p ken-runtime
scripts/ken-cargo test -p ken-runtime --test <new-target-name>
```

Add `-p ken-interp` **only if** the public `Value` shape moves (D3 family 2) —
`ken-interp` constructs `Value::Record` directly at `lib.rs:640+`, and a
`store.rs`/reifier-visible change needs the **full** `-p ken-interp` suite.

⛔ **NEVER `--workspace`** (operator hard rule, `COORDINATION §12`). The
full-workspace build, the `--locked` gate, and conformance run **in CI on
GitHub**. "No regression" here means **green in CI**.

## 7. What Phase 1 does NOT discharge — every residual gets a cell

| residual | where it goes |
|---|---|
| `Value::Closure` still exists and is still canonically encoded | **Phase 2** — a live spec violation, standing since `SPEC-CLOSURE-BOUNDARY` landed, and Phase 1 does not make it worse |
| `PartialEq`/`Eq`/`PartialOrd`/`Ord`/`Hash` on the whole enum, incl. closures | **Phase 2** (`AC-V4`, `AC-V8` in the node) |
| the derives disagreeing with canonical identity via `minimal_limbs` + NFC | **Phase 2** (node §3c) |
| the false memcmp-exact doc text in `values.rs` / `canonical.rs` | **Phase 2** (node `AC-V6`) — ⚠ it stays **operative and false** for this WP's duration, deliberately and visibly |
| `ken-foundation`'s twin closure-inclusive validation model | **Phase 2** (node `AC-V10`) |
| the operational → canonical checked projection | **Phase 2** (node `AC-V9`), which is **why this phase exists first** |
| `ir::RuntimeValue` deriving `PartialEq`/`Eq` across `ClosureRef` | **Phase 2** — pin 2 failing on the operational carrier right now |
| `RECUT 2`'s phase-closure artifact re-derivation | ⛔ **unrelieved** by this node or this phase — still a hard gate on B2V |

## 8. Standing

- ⛔ **Contention:** this WP rewrites `crates/ken-runtime/src/canonical.rs` and
  `values.rs` and adds `crates/ken-runtime/tests/`. Intersect that set against
  every WP **in flight**, not just the frontier, before release.
- **Report an unpushed ref and KEEP GOING** — you have no GitHub credential by
  design, the Steward pushes, and raising it is not gating on it.
- Wrap markdown at 80 columns. Treat every §2 current-state claim as perishable.
- Hard-stop count for this node stands at **0**; next research pull at the
  **3rd**. ⚠ The `RT-NATIVE-FNSPLIT` chain stands at **10** with catch-up armed
  at **#11** — if a hard-stop here is *the same wall* that chain kept hitting, it
  counts on **both**.
