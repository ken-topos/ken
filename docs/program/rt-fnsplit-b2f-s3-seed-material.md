# `RT-FNSPLIT-B2F` `S3`/`D3` — artifact-static seed material, and what the
# mutations measured

**Author:** `runtime-implementer` · **Branch:**
`wp/RT-FNSPLIT-B2F-functionization-live` · **Base:** `origin/main` = `6534e4a6`
(frame amended on `main` = `8cd4ad45`, blob `a94cec35`, docs-only — ⛔ no rebase
owed)

---

## 1. What landed

`B2R` declared `AbiCarrier::GroundValueCarrier` as
`AbiOwnership::BorrowedForActivation` from `AbiStorageOwner::ArtifactStatic`
and **deliberately minted nothing**. `declare_data`, `define_data` and
`DataDescription` had **zero** occurrences anywhere in `ken-runtime`, so this
was new emission machinery rather than a new call into existing machinery.

New `lowering/seed_material.rs`:

| piece | what it is |
|---|---|
| the encoding | total, self-describing, **relocation-free** over the whole `RuntimeGroundValue` family; wildcard-free match, so a new variant is a compile error rather than a silent mis-encoding |
| the mint | one **read-only** (`writable = false`) artifact-static data object per seed-environment entry |
| the caps | depth `32`, size `1 MiB`, both **rejecting rather than truncating**, each with a positive control immediately below the limit |
| the consumption seam | `lower_seed_capture` reads a scalar **out of that material** instead of folding an `iconst` |

⭐ **Read-only is the load-bearing part.** `Linkage::Local` +
`writable = false` is how `BorrowedForActivation` + `ArtifactStatic` stop being
prose: a borrower that cannot write cannot reclaim, cannot mutate, and cannot
hand ownership on. ⇒ Two declared modes become a property of the
**declaration** rather than a claim about the emitter's good behaviour.

⛔ **Minted from the environment, never from the plan.** Resolving which symbols
a unit captures needs an `origin -> expression` lookup, and **`AC-4` holds that
count at exactly one** (through `retained_body_occurrence`). ⚠ The cost, stated
rather than buried: an environment entry no capture reads is still minted, so
the object count is an **upper bound** on read material.

---

## 2. `AC-12` — what the emitted code does, per declared mode

⛔ Required to be **stated**, and ⛔ an assertion that reads a mode back out of
`AbiCarrier::ownership` discharges nothing — that re-measures a `const fn` over
a closed enum with itself.

| `AbiOwnership` | carriers | what emitted code does |
|---|---|---|
| `OwnedByFrame` | `ValueWord`, `ControlWord`, `TrapWord` | the value is materialized into the activation frame and dies with it |
| `BorrowedForActivation` | `GroundValueCarrier` | ⭐ **loads from a read-only artifact-static object; never writes it, never reclaims it** |
| `BorrowedForActivation` | `StoreHandle` | ⛔ **not emitted by this node yet** — `S5`/`S6`. Stated as absent rather than left to read as covered |
| `TransferredToCaller` | `ResultWord` | the unit body returns the result word and retains nothing |

---

## 3. ⭐ The mutations — three findings, one of them about my own instrument

Every mutation applied at its **natural production site**, restored
byte-identically, restore verified with `git diff --quiet` from a **clean
`HEAD`** (which is the only condition under which that oracle is valid).

### `M3` — corrupt the minted payload image

`push_word(out, *small as u64)` → `push_word(out, (*small ^ 1) as u64)` in
`encode_into`.

⭐ **Reddened two RUNTIME observations:**
`values::cranelift_runs_closure_seed_with_explicit_runtime_capture_environment`
and
`artifact::api::tests::program_runner_preflights_metadata_before_backend_lowering`.

⇒ ⭐ **The program's answer is a function of the minted bytes.** The borrow is
live, not cosmetic. ⚠ This was the open design question: `Lowered::Int` keeps a
`known: Some(v)` field, and if specialization had substituted `known` for the
loaded value in emitted code, the load would have been dead and the whole switch
decorative. **It is not.**

### `M5` — fold instead of borrowing

`self.artifact_static_payload(builder, symbol)?` →
`builder.ins().iconst(types::I64, *small)` in `lower_seed_capture`.

⭐ **Reddened exactly ONE test** — `AC-12`'s
`a_seed_capture_borrows_from_artifact_static_storage_rather_than_folding` — and
**nothing else noticed**, out of 495 others.

⚠ **That is the finding, not the pass.** Returning to constant folding is
invisible to the entire rest of the suite. ⇒ That single control is the **sole
mechanical defender of `D3`'s substance**, and it should be read that way by
anyone tempted to relax it.

### ⛔⛔ `M4` — delete the `define_data` call, leave the counter reachable

```rust
module.define_data(id, &description).map_err(…)?;   // ← deleted
#[cfg(test)] { defined += 1; }                       // ← still runs
```

⛔ **The `(declared, defined)` counter reported `(1, 1)`. GREEN.**

⭐ **A counter cannot detect the deletion of the call it counts.** ⚠ And this is
**one layer subtler than the defect repaired earlier in this WP**: there, the
increment was in the caller's loop and moving it *adjacent to the emitting call*
fixed it. Here the increment **is** adjacent — and still cannot see the call's
absence, because the counter and the call are **both my own code** and a
mutation can remove one and leave the other.

⇒ ⭐ **The fix is to ask a different party what it holds.**
`minted_seed_material_is_present_in_the_finalized_artifact` calls
`JITModule::get_finalized_data` and compares the module's own finalized memory,
byte for byte, against the image handed to `define_data`. Under `M4` it fails
with Cranelift's own diagnostic:

> `data object must be compiled before it can be finalized`

### ⚠ `M4`'s second finding — the mutation made the whole suite uninterpretable

⛔ **In a full-suite run, `M4` SIGSEGVs the test binary.** An undefined data
symbol is caught by **neither the module nor the counter** — only by the
hardware, when some *other* test runs the artifact against the garbage symbol.
The process dies and **every** control's result becomes unreadable, including
the one that would have caught it.

⇒ ⭐ **A mutation that produces an unrunnable artifact must be evaluated on the
ISOLATED control**, not on the suite. This is the *"a red baseline is no
measurement"* shape in a new substrate: here the baseline is not red, it is
**absent**, and a crash reads as a fail for every test at once.

---

## 4. `AC-2` — the two instruments, and which one carries the claim

Per the Steward's ruling that the census is **fail-open by construction**, the
division of labour is now stated **in-source**, adjacent to both:

| instrument | what it does | what it carries |
|---|---|---|
| ⭐ `b2f_last_unit_emission`, `b2f_last_seed_material_emission`, `b2f_last_seed_material_images` | count / read back what the compiled module **actually contains** | ⭐ **the population claim, entirely** |
| ⚠ the source-text census | searches for spellings someone enumerated | ⛔ **nothing.** A tripwire, retained unweakened |

⛔ Two counts side by side **read as corroboration** and these are not: one of
them fails open for every emission spelling nobody thought of.

### ⚠ `P6` was right about the counts and WRONG about the file

| | `P6` predicted | measured |
|---|---|---|
| `.declare_data(` / `.define_data(` | **1 / 1**, every other row `0/0` | ✅ **exactly that** |
| the file carrying them | `lowering/units.rs` | ⛔ **`lowering/seed_material.rs`** |

⭐ **Recorded rather than quietly corrected.** The material is minted in its own
module because units and seed material are **two populations on two growth
axes** — `Θ(n)` in the program versus `Θ(|seed environment|)`, which the program
does not affect — and one census row cannot carry both. ⚠ `P4` said in advance
that row placement was the likeliest thing to move; it moved.

---

## 5. ⛔ NOT CLAIMED

Stated as a **partition with its discriminator**, not as examples.

1. ⛔ **`D3`'s represented path covers SCALARS ONLY.** The discriminator is
   *does the variant carry an `ir::Value` in `Lowered`?* — `Bool` and
   `Int::Small` do and borrow from artifact-static material; `Bytes`, `String`,
   `Constructor`, `Record` and big integers do not and still lower through the
   compiler-side specialization lattice.
   ⚠ **This is a boundary, not a second authority** — ⭐ **no value has two
   paths.** The *encoding* covers all six; the *reader* covers two. Giving the
   aggregates an artifact-static representation needs a reader for the encoded
   aggregate, and ⛔ the runtime-`alloc` carrier is **not** a substitute: it
   produces activation-time storage for a slot declared `ArtifactStatic`, which
   would violate `AC-12` by construction.
2. ⛔ **`AC-11` is NOT discharged by any of this.** Its clause 1 wants a
   producer-tracing walk per `Parameter` / `Capture` / `Result`; nothing here
   traces a producer. That is `S4`.
3. ⛔ **`AC-3`'s four width invariants are NOT discharged.** Only the seed
   material's *alignment* is checked against its carrier's declaration.
4. ⛔ **Unit bodies cannot reach this material.** `declare_in_func` resolves the
   objects into **one** generated function; `units.rs` builds its own
   `Function` and would need its own call. Today only the root reads seed
   material, which is correct for `S3` and is `S6`'s to change.
5. ⚠ **The read-back oracle's *expected* side is this crate's own encoder
   output**, so it cannot catch an encoding wrong in the same way on both
   sides. That residual is covered by the encoder's tag / offset / nesting
   tests, whose expectations are written out independently of the encoder.
