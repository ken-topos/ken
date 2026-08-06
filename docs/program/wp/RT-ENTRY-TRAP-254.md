# RT-ENTRY-TRAP-254 — attribute the explicit entry trap, then return for sizing

**A linked native program that should observe its raw argv, environment and cwd
bytes and return one of them as its exit code instead traps at the entrypoint.
This unit finds out why. It does not repair it.**

**Owner:** Team Runtime. **Size:** S — diagnosis only.
**Node:** `docs/program/issues/RT-ENTRY-TRAP-254.md`.
**Risk:** low as framed. The repair is unsized by construction and is re-cut by
the Steward on this unit's return.

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

⇒ **`D0` un-skips it.** Nothing in this frame may be asserted against an ignored
row.

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

## 3. Deliverables

### `D0` — un-skip and confirm the live failure

Remove the `#[ignore]` from this one test, run it, and record the **exact**
stderr line and exit code. Confirm the signature is still
`explicit entry trap` and not something that has since changed shape.

**If it now fails differently, that is the finding** and it outranks continuing
the plan. If it passes, say so immediately — the row would have recovered as a
side effect of something else, and that is worth more than this whole unit.

Leave the other four `#[ignore]` attributes alone; they belong to
[[RT-CARRIER-BYTESPAN-OBSERVE]].

### `D1` — attribute the `-4`

**Name the emitter and the condition.** Which path in the emitted entrypoint
returns `-4`, and what makes this program take it.

The node recorded a hypothesis worth testing rather than assuming: the `-1`
sentinel is rendered from borrowed-input validation paths (Architect,
`evt_7v61ed5pn9q3t`). **Confirm or refute that `-4` shares that emitter.** A
refutation is as good a result as a confirmation; say which.

State whether the trap is reached **before or after** the program observes any
process bytes. Those are different defects: a trap before observation means the
entry path never ran, and a trap after means observation ran and something it
produced was rejected.

### `D2` — return for sizing

**Stop and hand back.** State the emitter, the condition, whether observation
ran, and what a repair would have to change. **Do not repair anything**, and do
not propose a size — the Steward re-cuts on this return.

**A guessed size on this campaign has been wrong every time it was guessed**,
which is why this unit ends here rather than continuing into a fix.

## 4. Acceptance criteria

- **`AC-1` (`D0`) — the live signature is recorded from an un-skipped run**,
  quoted exactly, with the executed test count asserted. **A green run that
  executed nothing is indistinguishable from a pass**, and an ignored test
  reports neither.
- **`AC-2` (`D1`) — the emitter of `-4` is named at `file:line`**, with the
  condition that selects it. **"It traps" is not an attribution.**
- **`AC-3` (`D1`) — before-or-after observation is stated**, with the evidence
  that decides it, not an inference from the sentinel's name.
- **`AC-4` (`D1`) — the borrowed-input-emitter hypothesis is explicitly
  confirmed or refuted**, and reported either way. An untested hypothesis
  reported as "not investigated" is an acceptable answer; silence is not.
- **`AC-5` (`D2`) — the four `RT-CARRIER-BYTESPAN-OBSERVE` rows are still
  ignored and untouched**, and this unit's diff contains no production change.

## 5. Banned scope

- **Do not change the test's expectations.** Making it expect `1`, or relaxing
  `assert_ne!`, converts a real defect into a green row and destroys the only
  signal anyone has. This is the cheapest available "fix" and it is forbidden.
- **Do not change the shim's `if (value < 0) return 1;` collapse.** Passing
  sentinels through as exit codes is a process-ABI change; it would also make
  `-2` indistinguishable from a legitimate exit `254`. If the diagnosis argues
  for it, that is a finding for the Architect, not a deliverable here.
- **Do not repair the trap.** Attribution first; the repair is a separate cut.
- **Do not touch the byte-span capability, the seat table, or the class
  dispatch.**
- **Do not un-skip the other four rows.**

## 6. Hard stop

Stop and report, with the concrete evidence, if:

- `D0` shows a different signature, or the row passes;
- the `-4` emitter cannot be named without changing production code; or
- attribution turns out to require the byte-span capability after all — which
  would be a real finding and would need to be **measured**, since the node's
  standing position is that it does not.

**Do not absorb any of these and do not work around them.**
