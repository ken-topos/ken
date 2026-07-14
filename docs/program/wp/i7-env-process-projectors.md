# I-7 — `Process.Environment` + `Process.WorkingDirectory` (the pure projectors)

**Team:** Runtime · **Size:** S · **Branch:** `wp/i7-env-process-projectors`
**Base:** `origin/main @ 7b4adf5b` (post-I-6) · **Gate:** Program I (Milestone B)
**Unblocks:** **CC8** (Foundation, currently HELD) — it consumes
`process_environment`.

## 0. What this WP is, in one line

Project the **second and third fields of the already-landed `ProcessInput`
ABI** into Ken, exactly as CC6a projected the first. **That is the whole WP.**

## 1. ★★ TWO TRAPS. READ THESE BEFORE YOU READ ANYTHING ELSE. ★★

### Trap 1 — **THE CLOCK IS NOT IN THIS WP, AND GREPPING FOR IT WILL LIE TO YOU.**

The CLI contract (§7) scopes the original "I-7" as *"env read/enumerate, cwd,
**clocks**."* **I have split the clock out into its own WP (I-8). Do not build
a clock. Do not build a `now`. Do not add an effect.**

You will be tempted, because **`rg Clock crates/` returns about ten hits.**
Every one of them is a decoy:

- `crates/ken-elaborator/src/effects/row.rs:14` — a **doc comment** listing
  *example* effect names: *"A named effect: `FS`, `Clock`, `Console`, `Net`,
  `Rand`, `Counter`, …"*
- `crates/ken-elaborator/tests/effects.rs:60,68,98,104` — **synthetic test
  fixtures**. They do `("now", EffectRow::singleton("Clock"))` — i.e. they feed
  the *string* `"Clock"` into the row-inference engine, which is **generic over
  effect names**. `"Clock"` there is an arbitrary made-up label, exactly like
  `"Counter"`.

**There is no `IOOp` clock arm, no `HostHandler` clock method, no `now`
primitive, no driver arm, no `PosixHost` time source.** The clock is **zero
percent built**, and the grep hits make it look half-built. *(This is the
"grep the emission, not the name" failure mode — the effect-row engine is
name-agnostic, so an effect "exists" in a test the moment someone types its
name in a string literal.)*

**If you find yourself adding an effect to this WP, STOP — you are in I-8.**

### Trap 2 — **DO NOT BUILD AN ENVIRONMENT *LOOKUP*. IT IS THE `Axiom` WALL.**

The obvious next function after `process_environment` is
`lookup_env : Bytes → List (Prod Bytes Bytes) → Option Bytes`. **Do not write
it.** A lookup must compare keys, so it needs **`DecEq Bytes`** — and byte
equality is precisely the thing that **cannot be proved today**: `bytes_eq` is a
`PrimReduction::Op`, **opaque to kernel conversion**, so `Refl` cannot discharge
any equation about it, and the only route available is an **`Axiom`** —
i.e. a real `trusted_base()` entry, per call site.

**That is exactly the trap that has already cost four packages** (CAT-5's
`SourceLength`, CC3's `ArgByteLength`, …). It is being fixed *right now* by
**SUB-1** (Team Language, `wp/sub1-bytes-structural-view`), which lands the
`bytes_to_list`/`list_to_bytes` structural bridge. **The lookup is CC8's, and
CC8 is held until SUB-1 lands.**

**⇒ I-7 needs NOTHING from SUB-1, and must not reach for it.** Projection is
pure structure. Keep it that way, and this WP has **zero TCB cost** and can run
fully in parallel with SUB-1.

> **If you write `Axiom` anywhere in this WP, you have gone wrong.** Stop and
> report, per the standing hard-stop authority (see §6).

## 2. Settled inputs — pinned, do not relitigate

**Verify each against the landed code at `origin/main`, not against this
line — this section is perishable.**

### 2.1 The `ProcessInput` ABI already carries everything you need

`crates/ken-elaborator/src/prelude.rs:1417`:

```
data ProcessInput = MkProcessInput (List Bytes) (List (Prod Bytes Bytes)) Bytes
```

| field | type | meaning | status |
|---|---|---|---|
| 0 | `List Bytes` | argv | **projected** by CC6a (`process_arguments`) |
| 1 | `List (Prod Bytes Bytes)` | environment pairs | **captured, never projected** ← *this WP* |
| 2 | `Bytes` | working directory | **captured, never projected** ← *this WP* |

**The data is already there.** CC6a's `match` already binds `environment` and
`working_directory` and passes them through untouched — it simply never named
them. **You are naming two existing fields, not plumbing new data.**

### 2.2 Determinism is ALREADY solved — do not re-solve it

I-6 extracted `run_program<H: HostHandler>` into `crates/ken-cli/src/lib.rs`.
The ambient reads (`std::env::vars_os()`, `current_dir()`) live **only in
`main.rs`'s thin wrapper**, which *passes them in* as parameters. The landed
library's `std::env` count is **0** — the Architect verified it.

**⇒ env and cwd are already injectable.** A `CaptureHost` test already supplies
them. **There is no host wiring to do in this WP.** This is the reason env/cwd
are *not* effects: they are **immutable program inputs captured once at the
boundary**, not observations of a changing world. (The clock is the opposite —
which is why it is a real effect, and why it is I-8.)

### 2.3 The pattern to mirror, exactly

`catalog/packages/Process/Arguments.ken.md` (CC6a) is your template. It is
**pure Ken in a literate `.ken.md` catalog package** — projector, replacer,
round-trip proof. Follow its shape, its byte-preservation posture, and its
naming.

## 3. Mandated deliverables

### 3.1 `catalog/packages/Process/Environment.ken.md`

```ken
fn process_environment (input : ProcessInput) : List (Prod Bytes Bytes) = …
fn replace_process_environment
      (environment : List (Prod Bytes Bytes)) (input : ProcessInput) : ProcessInput = …

proof round_trip for process_environment … : Equal …
```

### 3.2 `catalog/packages/Process/WorkingDirectory.ken.md`

```ken
fn process_working_directory (input : ProcessInput) : Bytes = …
fn replace_process_working_directory (working_directory : Bytes) (input : ProcessInput) : ProcessInput = …

proof round_trip for process_working_directory … : Equal …
```

**Both round-trip proofs are structural** — projection after replacement is a
`match` on a constructor, which **ι-reduces**, so they close by `Refl`. **This
is the honest case where `Refl` genuinely does work** — it is a constructor
projection, *not* a primitive op. If a proof will not close, that is a real
finding: report it, do not reach for `Axiom`.

### 3.3 Acceptance test

Mirror `crates/ken-elaborator/tests/cc6a_process_arguments_exit_acceptance.rs`.
Name the new globals and assert the round-trips at the **elaborated** level.

## 4. Acceptance criteria

1. **AC1 — the fields are reachable.** `process_environment` and
   `process_working_directory` project fields 1 and 2 of `ProcessInput`, and are
   callable from Ken.
2. **AC2 — byte preservation, on genuinely invalid UTF-8.** An environment
   **key *and* value** containing real invalid-UTF-8 bytes survives
   projection → replacement → projection **byte-identically**. Use actual
   invalid bytes (CC6a's test does; copy its construction), **not** a
   well-formed string you merely call "raw".
3. **AC3 — the round-trip proofs are `Refl`-closed, not postulated.**
   **`trusted_base()` MUST NOT GROW.** Assert this: **zero new `Axiom`, zero
   new `Decl::Opaque`, zero new primitive.** This is the WP's headline property.
4. **AC4 — the other two fields are untouched.** Projecting env must preserve
   argv and cwd bit-for-bit, and vice versa. (CC6a's `match` already does this;
   prove it didn't regress.)
5. **AC5 — no new effect, no new host op.** Grep the diff: **no `HostHandler`
   change, no `IOOp` variant, no `effects/row.rs` change, no `Clock`.**
   *A green diff that added an effect would still fail this WP.*
6. **AC6 — no regression.** Green **in CI** (never a local `--workspace` run —
   `COORDINATION.md §12`).

## 5. Do-not-reopen guardrails

- **The `ProcessInput` ABI is settled.** Three fields, that shape, byte-typed.
  Do not add a field, do not re-type `Bytes → String`, do not "improve" the
  environment carrier into a `Map`. Bytes in, bytes out; **decoding is always an
  explicit caller choice** (CC6a's posture — inherit it).
- **No `String` decoding.** Environment keys/values are `Bytes`. A UTF-8 view is
  a *consumer's* decision (CC8's), and it needs the SUB-1 bridge.
- **No lookup, no `DecEq Bytes`, no `Axiom`** (§1, Trap 2).
- **No clock, no effect, no `now`** (§1, Trap 1) — that is **I-8**.
- **No `Map`/`Dict` substrate.** Do not mint a new collection to hold env pairs;
  `List (Prod Bytes Bytes)` is what the ABI carries and what CC8 expects.

## 6. Standing hard-stop authority

**This frame's §2 "current state" claims are perishable, and this frame was
written by the enclave, not by someone who read every line of the runner.**

If any pinned anchor contradicts the landed code — a field order, a type, a
name, an assumption that a projection is expressible — **STOP BEFORE EDITING**,
keep the tree clean, and post the exact `file:line` anchors plus any bypass you
could have taken but didn't.

**Two WPs in a row have been saved this way** (I-5's seam, and I-6's inherent
`mint_fs_cap`, which killed a false pin in *my own* frame **pre-edit**). **That
is not insubordination; it is the job.** I would rather eat a stopped WP than
merge a frame's wrong assumption.

## 7. Why this is split from the clock (rationale, for the record)

The contract bundles "env, cwd, clocks" into one line item. **They are two
different kinds of thing, with different dependents and different cost:**

| | env + cwd (**I-7**) | clock (**I-8**) |
|---|---|---|
| **Kind** | pure projection of a captured, immutable input | a real **effect** — an observation of a changing world |
| **Cost** | ~zero: two `match`es, two proofs, zero TCB | a full vertical: op + `HostHandler` + `PosixHost` + `CaptureHost` + effect row + driver arm |
| **Determinism** | **already solved** (I-6 injects them) | **the whole point** — needs a fixed clock in `CaptureHost` |
| **Needed by** | **CC8, which is BLOCKED and Foundation is IDLE** | nothing in the current DAG |

**Gating CC8 behind a clock effect that CC8 never calls would be inventing a
dependency.** I-6's own lesson was that *shipping the honest small thing beat
inventing an `L` to justify a line item* — this is the same call, made
deliberately rather than accidentally.
