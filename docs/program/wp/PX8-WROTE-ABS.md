# PX8-WROTE-ABS — the interpreter's capped-short `Wrote` oracle

**The interpreter has capped-full read, capped-short read, and capped-full
write. The fourth cell — capped-short write — does not exist. Native has both
write cells. The corpus's own comment, twelve lines above the gap, says
capped-short is *"the load-bearing pair member."***

**Owner:** Team Verify (`verify-leader` + `verify-implementer` + `verify-qa`).
**Branch:** `wp/PX8-WROTE-ABS`. **Size:** S.
**Risk:** low — one test added to an existing module, no `src` behavior change.

**Status:** Steward frame, shovel-ready. ⭐ **On the Linux ABI I critical
path** — `PX8` does not close until this and `PX8-F-CAP-41` discharge, and 15
of the program's 19 nodes descend from `PX8`.

---

## 1. Fixed inputs

| path | blob at `origin/main = 12a5ef4f` |
|---|---|
| `crates/ken-interp/src/eval.rs` | `8ecf5d4b7ce063fa060c57411a6abd5c3ca2c797` |
| `crates/ken-runtime/src/cranelift_backend/lowering/core/tests/effects.rs` | `24edfa47e17f386d99c66ad2d22992ed9bc4dd91` |
| `spec/30-surface/38-ffi-io.md` | `56c3b3d5f1090f8920cc66286e0d7ba3729f0113` |
| `conformance/behavioral/buffer-io/seed-buffer-io.md` | `0364b230742e08f67fc59a2c2421221744b051e0` |

⚠ `38-ffi-io.md` is **LOCKED** and is the absolute oracle. ⛔ This WP does not
edit `spec/`, `conformance/`, or any `src/**` behavior.

---

## 2. The measurement

The `budget_eff_*` oracle family in `crates/ken-interp/src/eval.rs`:

| cell | interpreter | native |
|---|---|---|
| capped-**full** read | ✅ `:6469` | ✅ `effects.rs:~430` |
| capped-**short** read | ✅ `:6554` | ✅ `effects.rs:~440` |
| capped-**full** write | ✅ `:6629` | ✅ `effects.rs:453` |
| capped-**short** write | ⛔ **ABSENT** | ✅ `effects.rs:453` |

Native's test
`budget_eff_native_wrote_capped_full_and_short_reify_effective_not_raw_remaining`
asserts both write cells in one test, the short one being
`raw 8 / effective 4 / count 2 -> remaining 2`, labelled in-source
*"NOT 6 == raw 8 - count 2, the pre-fix defect this WP closes."*

⇒ **The interpreter has no corresponding assertion.**

### ⭐ The corpus already argues for this test, in its own words

`eval.rs:6461-6467`, the comment introducing the whole family:

> *"capped-full ALONE is satisfiable by the wrong shortcut `effective :=
> count` (both diverge from raw budget only because they happen to equal the
> count in the full-buffer case). capped-short is the load-bearing pair member
> — it is the only shape where `effective` and `count` differ, so it is the
> only shape that can catch an implementation that quietly substitutes one for
> the other."*

That reasoning was applied to the read pair and **not** to the write pair.
The wrong shortcut on the write side is the same one: the reifier arm at
**`eval.rs:5316-5326`** is green under capped-full because both formulae yield
`remaining == 0`. The operand is `:5322`:

```rust
let count = transferred.get();
let effective = transferred.effective_request();   // :5322 — the operand
let remaining = buffer_nat_value(effective.checked_sub(count)…)?;  // :5324
```

> ### ⛔ LOCATOR CORRECTED 2026-07-27 — this frame cited `eval.rs:4981-4997`
>
> **That range is wrong and it was wrong when written.** `:4981-4997` is the
> `FsReadAt` / `PrivateFsWriteAt` **request-argument narrowing** block
> (`narrow_host_u64` on offset/start/length) — it does not compute `effective`
> at all, so `effective := count` is not expressible there. Re-measured at
> `origin/main = 5fbbc67e`, where the `eval.rs` blob is **unchanged** from the
> `12a5ef4f` this frame was pinned at ⇒ ⛔ not drift, an authoring error.
>
> ### ⭐⭐ And the correction matters more than a line number
>
> **`:5303` is the `ReadSome` arm and carries a byte-identical expression** —
> `let effective = transferred.effective_request();`. ⇒ A ring that mutates
> "the `effective` computation" and sees red may have hit the **read** arm and
> reddened the **pre-existing** read tests (`:6469`, `:6554`), concluding
> `AC-1` discharged while the new write test never discriminated anything.
>
> ⛔ **The mutation must be on the `Wrote` arm at `:5322`, and the redden must
> be attributed by test name.** See `AC-1`.

⚠ **The source formula is presently right.** This is an *evidence* gap, not a
behavior gap — the Architect's clause-(a) verdict (`evt_163mfgjs7fkh8`)
classes it as *absolute-not-differential evidence not discharged*. ⛔ Do not
open this expecting to find a bug in `eval.rs`; expect to find that nothing
would notice if one appeared.

---

## 3. ⭐⭐ Steward-discharged — and one question I deliberately did NOT answer

### 3a. Scope is **A2a only**. A2b is split out.

`docs/program/issues/PX8-WROTE-ABS.md` carries two gaps. **A2b** — the five
PR-C error identities (`MalformedResource`, `InvalidBounds`,
allocation-failure-distinct-from-`BufferLimit`, unsupported-nonblocking
posture, host-I/O-failure-distinct-from-`Interrupted`) with no independent
reaching evidence — **needs a normative scoping call before it can be sized**,
per the Architect's own second route. It is filed as `PX8-ERRID-SCOPE` and is
**not yours**. ⛔ Do not widen into it, and ⛔ do not treat its absence as a
reason this WP cannot close.

### 3b. ⛔ The fixture construction is the WP, and I am not prescribing it

> ⛔⛔ **STOP — the dichotomy in this section is FALSE and was ruled so on
> 2026-07-27. Read the SUPERSEDED banner at the end of `§3b` before acting on
> anything below.** Neither branch is the mechanism; `D3`'s second form is
> ruled and `AC-6` is withdrawn. The text is kept only as the record of what
> was asked.

The discriminating shape requires **`count < effective < requested`**. On the
read path that is produced by a short source file (`:6554` reads 2 bytes into
a capacity-4 buffer against a length-8 request).

⚠ **I have not verified that the write path can produce it**, and I am not
going to guess in a frame. The open question is exactly:

> On `PrivateFsWriteAt`, is `effective` the buffer's **capacity** or its
> **installed window length**?

- If **capacity**: install a 2-byte window in a capacity-4 buffer, request 8
  ⇒ `count 2`, `effective 4`, `remaining 2` — discriminating, and the test is
  a direct mirror of `:6554`.
- If **installed window length**: `count == effective == 2 ⇒ remaining 0`, the
  same coincidence that makes capped-full non-discriminating, and **the cell
  is not merely missing — it is inexpressible on this path.**

⭐ **Both outcomes are acceptable deliverables.** See `§5 D3`.

> ## ⛔⛔ SUPERSEDED 2026-07-27 — the dichotomy above is FALSE
>
> **Ruled `evt_1grq3fcfkz4yy` on `verify-implementer`'s hard stop, measured at
> `origin/main = 06722d2b` and verified independently. Durable here because a
> ruling that lives only in the channel is not an input.**
>
> **Neither branch is the mechanism.** `effective` **is** capacity-backed
> (`effect_v1.rs:1784-1786`: `effective = min(requested, capacity - start)`), so
> branch (b) is false — **and branch (a)'s fixture is still unbuildable.**
> `count < effective` is **unreachable** on the interpreter write path, by a
> composition of four facts:
>
> | # | fact | site |
> |---|---|---|
> | 1 | `effective = min(requested, capacity - start)` — capacity-backed | `effect_v1.rs:1784-1786` |
> | 2 | the whole `[start, start+effective)` must be in the installed live window ⇒ a 2-byte window returns exact `InvalidBounds` **before any write** | `:1798-1800` → `:681-692` |
> | 3 | `InterpreterHostBackend` does **not** override `fs_resource_write_at`; it inherits the direct POSIX call ⇒ **no injectable short-write seam** | `eval.rs:4399-4697`, `effect_v1.rs:1356-1364` |
> | 4 | `TransferCountV1::new` is **`pub(crate)`** to `ken-host` ⇒ `ken-interp` cannot hand-construct `count 2 / effective 4` either | `effect_v1.rs:2188` |
>
> ### ⭐⭐ Why this frame got it wrong — the matrix's symmetry is an artifact
>
> `§2`'s 2×2 presented four cells as four instances of one shape. **They are
> three different shapes:**
>
> - **read** cells get their discriminating count **free from file size** —
>   `:6554` writes a 2-byte source and the real POSIX read returns 2;
> - **writes have no analogue** — POSIX `write()` to a regular file writes
>   everything or errors. There is no "short source" for a write;
> - **native's** cells test the **lowered arithmetic**, not a host reply —
>   `run_checked_bounded_nat_fixture(2, 0, 8, 4, …)` takes the scalars as
>   arguments.
>
> ⇒ ⭐ **Drawing them in one table made a genuinely absent capability look like a
> missing test.** ⛔ Do not read a matrix cell as a comparable instance without
> checking that the cells share a production route.
>
> ### ✅ Authorized: `D3` second form, and nothing else
>
> ⛔ **No write fixture. No seam. No `src/**` behavior change.** ⭐ A
> production short-write seam added so a test can force `count 2` is a
> production change driven by test convenience — banned, and unnecessary.
>
> ✅ The measurement **may** be committed as a comment adjacent to the `Wrote`
> arm (`eval.rs:5316-5326`), in the style of the existing `:6456-6465` note.
>
> ⚠ **Consequence routed above this WP:** if the cell is inexpressible, `PX8`
> clause (a)'s **universal** absolute-evidence claim cannot be discharged by
> adding a test here. Whether the clause covers a value with **no production
> route** is the Architect's call. ⛔ It does not gate this WP's close.

### 3c. ⭐ If it is inexpressible, that is a FINDING, not a failure

⚠ A test cell that *cannot* be constructed is byte-identical, to any reader,
to one that simply has not been written yet. This corpus has two other
instances of exactly that class already filed (`CONF-FMT8-LEVELTOK`, and
`SEC1-IFC-R3`'s synthetic `Disproved` verdicts).

⇒ If you establish that `count < effective` is unconstructible on the write
path, **say so with the measurement and stop** — do not manufacture a fixture
that reaches the shape by some other route, and ⛔ do not weaken the assertion
to something the coincidental shape can satisfy. Route it to me under `§8`.

---

## 4. ⛔ Banned shapes

- ⛔ **No change to any `crates/**/src/**` behavior.** The formula is correct;
  you are adding the assertion that would notice if it stopped being.
- ⛔ **Do not edit `spec/` or `conformance/`.** `38-ffi-io.md` is LOCKED and is
  the oracle you assert *against*. Editing a cited source moves its OID.
- ⛔ **Do not touch the three existing `budget_eff_*` tests**, their shared
  helpers (`nat_value`, `allocate_buffer`, `open_file`, `release_resource`,
  `rt_parity_root`), or the `:6450-6467` comment block beyond appending to it.
- ⛔ **Do not assert differentially against the native test.** Clause (a) is
  *absolute-not-differential*: the expected `remaining` must be stated as a
  literal derived from `38`, not as "whatever native produced." ⭐ A
  cross-lane equality check is a different property and does not discharge
  this one.
- ⛔ **No `--workspace` run.** Targeted only (`COORDINATION §12`).

---

## 5. Deliverables

- **`D1`** — the interpreter capped-short `Wrote` absolute oracle, in the same
  module and following `:6554`'s shape, asserting `remaining` as a literal
  with an in-source line naming the rejected pre-fix formula (mirror
  `:6554`'s *"not requested(8) - count(2) = 6"* phrasing against the write
  path's own numbers).
- **`D2`** — the `§38` clause the literal is derived from, cited by section in
  a comment. ⭐ A reader must be able to check the number against the spec
  without re-deriving it.
- **`D3`** ⚖️ **RULED 2026-07-27 — second form applies**
  (`evt_1grq3fcfkz4yy`).
  ⛔ `D1` is **not** available: `count < effective` is unreachable on the
  interpreter write path. **Deliver the written measurement**, citing all four
  sites in the `§3b` superseding table plus the read/native/write asymmetry.
  ⛔ The clause *"the mechanism that forces `effective` to the installed
  window"* is **withdrawn** — that mechanism does not exist. ⭐ The second form
  is a full deliverable, not an abandonment.
- **`D4`** — a one-line statement of whether the `:6450-6467` comment's
  "load-bearing pair member" reasoning is now true of **both** pairs, or
  remains true of the read pair only and why.

---

## 6. Acceptance criteria

- **`AC-1`** ⭐ **(load-bearing)** — the new test **discriminates**. **Control:**
  mutate the **`Wrote`** reifier arm at **`eval.rs:5322`** to the wrong shortcut
  (`let effective = count;`, or `remaining := requested - count`), show the new
  test **reddens**, and restore byte-identically. ⛔ A test that stays green
  under that mutation has measured nothing and does not discharge clause (a).
  ⚠ Confirm the redden is **your** test and not a build break — report the
  failing test name from the run output, not the exit code.
  ⛔⛔ **And confirm you mutated the WRITE arm.** `:5303` is the `ReadSome` arm
  and carries the **identical** expression; mutating it reddens the
  pre-existing read tests (`:6469`, `:6554`) and says **nothing** about your
  new write test. **Control:** name the failing test, and report that the
  capped-short *read* test at `:6554` is **unaffected** by your mutation.
  ⭐ A redden you cannot attribute to a named test is not evidence about your
  detector.

- **`AC-2`** — the capped-**full** write test, unmodified, **stays green**
  under the same mutation is **not** required and must not be claimed. ⭐ It is
  expected to stay green — that is the entire premise of this WP. **Control:**
  report its behavior under the mutation either way; if capped-full *also*
  reddens, the mutation was wider than intended and `AC-1` is unproven.

- **`AC-3`** — the asserted `remaining` is an **absolute literal** traceable to
  `38-ffi-io.md`. **Control:** quote the governing clause. ⛔ "Matches native"
  fails this AC.

- **`AC-4`** — scope. **Control:** `git diff --name-only` shows
  `crates/ken-interp/src/eval.rs` and nothing else.

- **`AC-5`** — targeted green. **Control:**
  `scripts/ken-cargo test -p ken-interp --lib` — the whole lib test suite, not
  a single `--test` filter, because these tests live in `src/eval.rs`'s inline
  module. ⚠ Re-derive build-slot availability first; `ken-cargo` blocks
  silently up to 30 minutes on lock contention.

- **`AC-6`** ⛔ **WITHDRAWN AS WRITTEN 2026-07-27** (`evt_1grq3fcfkz4yy`). It
  required naming *"the code that forces `effective` to the installed window"* —
  **nothing does; `effective` is capacity-backed** (`effect_v1.rs:1784-1786`).
  The AC presupposed a branch of the `§3b` dichotomy that is false, so it was
  not satisfiable by the true finding.
  **REPLACED — `AC-6′`:** the measurement names, by file and line, the code that
  makes `count < effective` **unreachable**. **Control:** all four sites of the
  `§3b` superseding table are cited — capacity-backed `effective`, live-window
  admission, the absent backend override, **and** the `pub(crate)`
  `TransferCountV1::new`. ⛔ Citing fewer than four is an incomplete census: each
  one independently closes a different route, so omitting one leaves that route
  looking open. ⛔ A prose assertion that it "appears not to be possible" still
  does not discharge this.

---

## 7. Contention

`crates/ken-interp/src/eval.rs` — **no live WP branch touches it.** Checked:
the three branches with `ken-interp` deltas (`wp/ABI-S3`, `wp/BUDGET-EFF`,
`wp/BUDGET-EXHAUST`) are all **merged** nodes. The seven active rings are on
`ken-runtime`/cranelift (Runtime), surface spaces (Language), `ken-kernel`
(Kernel), `ken-elaborator/tests` (Verify's current WP, Ergo), `spec/30`
(enclave), and `library/` (Doc).

⚠ **Re-measure at pickup** — `origin/main` moves.

---

## 8. Hard stop

⛔ Route to the Steward if:

- `count < effective` is unconstructible on the write path — ⭐ that is `D3`'s
  second form and a real result; deliver the measurement, do not improvise a
  fixture around it; **or**
- the mutation in `AC-1` reddens the capped-full test too, or reddens nothing
  — either means the control is not measuring the arm you think it is; **or**
- discharging `D1` appears to require editing `spec/`, `conformance/`, or any
  `src/**` behavior — it does not, and if it does, the gap is not the one this
  frame describes; **or**
- you find the source formula is actually **wrong** (not merely unasserted).
  ⭐ That would be a behavior gap, which is `PX8-F-CAP-41`'s class, not this
  one — stop and route it rather than fixing it here.
