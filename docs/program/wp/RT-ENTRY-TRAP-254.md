# RT-ENTRY-TRAP-254 — localize the explicit entry trap, then repair it

**A linked native program that should observe its raw argv, environment and cwd
bytes and return one of them as its exit code instead traps at the entrypoint.
The diagnosis is DISCHARGED and the trap is attributed. What remains is that the
attribution stops one level short: the root process-sentinel discards the
identity that would say WHICH default fired. Build that instrument, localize,
repair.**

**Owner:** Team Runtime. **Size:** M — diagnosis DISCHARGED; now instrument, localize, repair.
**Node:** `docs/program/issues/RT-ENTRY-TRAP-254.md`.
**Risk:** low. The instrument must not change which programs trap, and that is
`AC-6`. The repair is bounded by `D4`; if it exceeds about an hour it returns to
the Steward rather than being absorbed.

**Authority:** Steward, on the tip measurement `evt_2h8wm2ff99ayq` and the
provenance probe `evt_fxgentgrpw6g`. Filed because this was the one attributed
tip failure with no owning node, and an unowned failure is what gets lost.

---

## 1. Base and fixed inputs

**Cut from current `origin/main`**, which now carries the
`RT-DECL-CLOSURE-PORT` / `RT-CONTSRC-PRODUCER-LOCAL` merge. Do not continue
`wp/RT-DECL-CLOSURE-PORT-typed-units` — it was squashed and is not an ancestor.

**Treat every fixed input below as perishable. If one turns out false against
the landed code, say so and escalate — do not quietly build around it.**

### The failing row, and it ships marked `#[ignore]`

`crates/ken-cli/tests/px4b_native_production.rs`,
`public_source_observes_raw_argv_environment_cwd_bytes_in_field_order`, Linux
only. It was annotated under the operator's 2026-08-06 publish ruling and
**measures nothing while the attribute is present.**

⇒ **`D0` un-skipped it and is DISCHARGED (section 3).** The attribute is back on
at `21fd46dc`. **Nothing may be asserted against an ignored row**, so `D5` must
remove it in the same commit that greens the row.

### The observed signature

```text
ken native trap: explicit entry trap
exit Some(1), where Some(254) is expected
```

### Provenance: branch-introduced, with no green/red boundary

| SHA | result |
|---|---|
| `e6b4a13b` merge base | GREEN |
| `3015aafd` main | GREEN |
| `b9189ee9` | GREEN |
| `c7410b79` | RED — `ken native trap: malformed borrowed process input` |
| `21fd46dc` tip | RED — `ken native trap: explicit entry trap` |

**No last-green/first-red pair exists for this signature.** The test is red
continuously from `c7410b79`, so under skip-not-bad discipline every commit
carrying the older trap is a *skip* by construction. The answerable question was
when the signature **changed shape**: last `malformed borrowed process input` at
`fb663bf3`, first `explicit entry trap` at `9cea8a5e`, adjacent and verified
`9cea8a5e^ == fb663bf3`, both endpoints re-probed directly rather than inherited
from the bisect log.

## 2. What the exit code does and does not tell you

### 2a. The exit `1` carries no information. Do not investigate it.

The linked shim in `crates/ken-runtime/src/object_linker_packaging.rs`
(near `:2107`) ends:

```c
long long value = ken_nc23_entrypoint(frame, services);
...
if (value == -1) fputs("ken native trap: malformed borrowed process input\n", stderr);
else if (value == -2) fputs("ken native trap: entrypoint returned a malformed ExitCode\n", stderr);
else if (value == -3) fputs("ken native trap: malformed ExitCode::Failure payload\n", stderr);
else if (value == -4) fputs("ken native trap: explicit entry trap\n", stderr);
else if (value < 0)   fputs("ken native trap: unknown terminal sentinel\n", stderr);
if (value < 0) return 1;
return (int)value;
```

**Every negative sentinel collapses to exit 1.** The exit code cannot
distinguish `-1` from `-4`; only the stderr line can.

⇒ **"exit 1 instead of 254" is a consequence, not the defect.** The single fact
that matters is that `ken_nc23_entrypoint` returned **`-4`**. A turn spent on
the exit code is a turn spent on the shim behaving exactly as designed.

### 2b. `254` IS the correct expectation, and this is settled here

The node listed *"decide whether `254` is still the correct expectation, or
whether the test encodes a contract the branch legitimately changed"* as an open
obligation. **It is answerable from the test and it is answered: the expectation
is correct.** Do not re-open it.

The test sets `K` to the single byte `0xfe` with `env_clear()`, passes argv byte
`0xff`, and asserts:

```rust
assert_eq!(first.status.code(),  Some(254));  // 0xfe -- the raw ENV byte, returned
assert_eq!(second.status.code(), Some(253));  // the distinct fallback arm
assert_ne!(first.status.code(), second.status.code());
```

`254` and `253` are **legitimate non-negative program exit codes**: the program
observes a raw process byte and returns it, and `return (int)value` passes any
non-negative value straight through. **The value 254 is producible by this shim
today** — `0xfe` is exactly what the environment holds.

⇒ **The program is meant to compute and return a byte. It traps instead.** The
question is why the entrypoint reaches an explicit trap, not whether 254 was
ever right.

**The `assert_ne!` is a pre-existing control worth knowing about:** it means the
two arms must *differ*, so collapsing both to one value cannot pass. Do not
weaken it.

### 2c. The inference that is BANNED

**Do not conclude this belongs to the byte-span family because the test name
contains "bytes."**

The name is `..._observes_raw_argv_environment_cwd_bytes_in_field_order`, and
folding it into `RT-CARRIER-BYTESPAN-OBSERVE` on that basis is exactly the
vocabulary inference the Architect refuted on this campaign
(`evt_7v61ed5pn9q3t`), where a signature was matched against a function that did
not exist at the commit in question and the words agreed only because they were
generic.

**Measured position:** a byte-span observer **cannot** clear this trap.
Byte-span is a lowering refusal at a host-effect seat; this is a runtime
sentinel from a program that compiled. Whether the two share a root cause is
**unmeasured and must be measured, not argued from the name.**

## 3. Diagnosis: DISCHARGED 2026-08-06. What it returned.

`D0`-`D2` are complete (`evt_29m0gnx2r43jw`), tree byte-identical to `21fd46dc`
with all five ignores restored. **Do not re-run them.**

- **`D0`** — one executed failing test, exact stderr `ken native trap: explicit
  entry trap`, exit `Some(1)` versus expected `Some(254)`. Signature unchanged.
- **`D1`** — the `-4` is emitted at `lowering/mod.rs:16468`, `emit_current_trap`,
  under `TrapExitAuthority::Root { process_sentinel: true, source_authorized:
  true }`, reached from `seal_source_trap_branch` for
  `Specialized(Lowered::Trap(..))`.
- **The borrowed-input hypothesis is REFUTED**, and reported as such: `-1` comes
  from separate require/validation emitters and `emit_current_trap` has **zero**
  `-1` emissions.
- **The trap is reached AFTER host observation** — a 98-byte `KETRACE2` trace
  exists before termination. **Bounded correctly by the ring and kept bounded
  here: that establishes observation was RECORDED, not that each field was
  DECODED.**

### The Architect's population ruling, and it closes the biggest open question

**`evt_m36y2zegby7m`.** This row is **NOT** an activation of the `AC-1`
source-machine carried-match mechanism, and the ruling is by path, not by
vocabulary:

`process_discriminator` is called as a functionized declaration closure;
`RuntimeExpr::Call` routes a `DeclarationClosure` to
`call_declaration_closure_unit`; the declared-unit call makes every specialized
input a carrier word at the call boundary, so parameters install as
`LoweringOperand::Carried`; an ordinary `RuntimeExpr::Match` on a `Carried`
scrutinee dispatches to the **generic** `lower_carried_match`.
**`lower_source_carried_match` has one caller on the relevant path —
`SourceContinuation::MatchScrutinee` — and this path never constructs or resumes
that continuation.**

⇒ **The activation gate has NOT fired. This node stays independent.** Do not
fold it into the source-machine mechanism and do not rescope
[[RT-CARRIER-BYTESPAN-OBSERVE]]'s `D6` register on this evidence.

## 4. The recut — the ATTRIBUTION IS INCOMPLETE, and the missing instrument is the first deliverable

**Steward, 2026-08-06, on the ruling above.** The Architect named the remaining
gap precisely:

> Because the root process-sentinel arm discards `identity.abi_word()`, the
> present run does not localize which nested ordinary match default fired; that
> remaining attribution gap must not be converted into a source-machine
> attribution.

**So the `abi_word` discard is not a separate tidiness item. It is the reason
attribution stopped**, and the sibling branch already shows the shape: it
encodes the identity in `ROOT_TRAP_TOKEN`. **Build the instrument, then measure,
then repair.** Sizing the repair before the instrument exists is guessing, and
guessed sizes on this campaign have been wrong every time.

### `D3` — make the root process-sentinel carry `identity.abi_word()`

Follow the sibling `ROOT_TRAP_TOKEN` encoding rather than inventing a second
one. **This is an instrument, and it must not change which programs trap** —
only what a trap reports.

### `D4` — localize the default

With `D3` in hand, name **which nested ordinary match selects a closed default**,
at `file:line`, and why this program's carried process-input-derived scrutinee
reaches it.

**State the population.** The finding is about the **generic** carried/borrowed
match family, not the source-machine one. Any sentence that drifts back to
`lower_source_carried_match` is the exact conversion the Architect forbade.

### `D5` — repair, or hard-stop and return

If `D4` bounds a repair to about an hour, do it. **If it does not, stop and
return the bound** — the Steward re-cuts. Do not absorb a large repair.

### `D6` — the stale comment the ruling exposed

`core.rs:10416-10419` still says **nothing in production emits a carried
scrutinee**. **The px4b functionized-unit path refutes that sentence** (Architect,
`evt_m36y2zegby7m`). Correct it to what is now measured.

**This is folded here rather than given its own node** because this unit's work
is the very path that refutes it and its author is already in that file.
**It is a comment on the GENERIC helper and it is NOT an `AC-1` activation
erratum** — do not write it as one, and do not let it merge the two nodes.

## 5. Acceptance criteria

- **`AC-6` (`D3`) — the sentinel carries the identity, and NO program changes
  which way it terminates.** Show a program that trapped before still traps, and
  a program that returned a value still returns the same value. **An instrument
  that alters the population it measures is not an instrument.**
- **`AC-7` (`D3`) — the encoding is the sibling's, not a second one.** Name the
  shared encoder. Two spellings of one token drift, and the drift is invisible
  because each arm is exercised by different programs.
- **`AC-8` (`D4`) — the default is named at `file:line`**, with the exact
  scrutinee and case set. **"A nested match" is not a localization.**
- **`AC-9` (`D4`) — the report states the family as GENERIC**, and contains no
  claim about `lower_source_carried_match`.
- **`AC-10` (`D5`) — if the row greens, it greens by un-ignoring**, and the
  `#[ignore]` attribute is removed in the same commit. A green suite that still
  carries the attribute has discharged nothing. Report `passed / failed /
  ignored` as three numbers.
- **`AC-11` (`D6`) — the corrected comment states the measured path**, and does
  not claim an activation.
- **`AC-12` — the other four `#[ignore]` attributes are untouched.**

## 6. Banned scope

- **Do not change the test's expectations.** Making it expect `1`, or relaxing
  `assert_ne!`, converts a real defect into a green row. `254` is correct: the
  program observes a raw process byte and returns it, and `return (int)value`
  passes non-negative values straight through.
- **Do not change the shim's `if (value < 0) return 1;` collapse.** That is a
  process-ABI change and it would make `-2` indistinguishable from a legitimate
  exit `254`. If the diagnosis argues for it, that is a finding for the
  Architect.
- **Do not attribute anything here to `lower_source_carried_match`.** Ruled
  `evt_m36y2zegby7m`.
- **Do not fold this node into byte-span**, and do not touch the byte-span
  capability, the seat table, or the class dispatch.
- **Do not un-skip the other four rows.**

## 7. Hard stop

Stop and report, with the concrete evidence, if:

- `D3` cannot encode the identity without changing which programs trap;
- `D4` cannot localize the default even with the instrument — that is a finding
  about the instrument, and it returns to the Architect; or
- `D5`'s repair exceeds about an hour.

**Do not absorb any of these and do not work around them.**
