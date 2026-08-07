# RT-WORKER-FIXTURE-DECODE — restore the dark AC-5 detector, and name what token 9 is

**Owner:** runtime · **Size:** M · **Gate:** none ·
**Depends on:** `RT-SRCBODY-BIND-ORDER` (merged) ·
**Node:** `docs/program/issues/RT-WORKER-FIXTURE-DECODE.md`

`two_same_shape_workers_are_distinguished` is documented as `AC-5`'s
target-redirect red. It carries three `assert_ne!` comparisons and reaches none
of them: its first statement panics. The row now ships `#[ignore]`d, so the
property it advertises is guarded by nothing.

**The deliverable is a live detector, not a green row.** Un-ignoring a row whose
first statement panics restores a red; weakening the `expect` restores a lie.

## 0. Posture

**This frame makes the node `ready`. It does not release it.** The fleet is
single-threaded (operator, 2026-08-07) and Runtime is executing
`RT-CARRIER-BYTESPAN-OBSERVE`. Nothing here is startable until the Steward kicks
it.

## 1. Base and fixed inputs

### 1a. The governing base is `main`

Base is `origin/main`, **not** the `RT-SRCBODY-BIND-ORDER` branch. The publisher
squashes, so `21fd46dc` and `fb99d0fc` — the refs every provenance claim in the
node was measured at — are not ancestors of `main`. Cut
`wp/RT-WORKER-FIXTURE-DECODE-<slug>` from `origin/main` at kickoff and re-measure
`D0` there.

### 1b. Anchors, all verified in source at `89916fc1`

Every line number below was read out of `git show origin/main:<path>` while
writing this frame. Label them MEASURED HERE.

| anchor | coordinate |
|---|---|
| the ignored row | `crates/ken-runtime/src/cranelift_backend/lowering/core/tests/constructors.rs:5816` |
| its `#[ignore]` | `constructors.rs:5815` |
| the expression builder | `constructors.rs:5712` (`two_same_shape_workers`) |
| the shared helper | `constructors.rs:5772` (`run_worker_fixture`) |
| the compile step, which SUCCEEDS | `constructors.rs:5777` |
| the run step | `constructors.rs:5778` (`.run(None)`) |
| the panic site | `constructors.rs:5779` (`.expect("the worker fixture runs")`) |
| the live sibling caller | `constructors.rs:5895` (`nested_worker_depends_on_both_levels`) |
| the failure variant | `cranelift_backend/surface.rs:192` |

### 1c. THE HELPER IS NOT BROKEN. This is the frame's central correction.

**The node's title says "the worker fixture cannot run." That is false as a
general statement, and building on it sends the first hour to the wrong file.**

`run_worker_fixture` has **exactly two callers**:

| caller | line | state |
|---|---|---|
| `two_same_shape_workers_are_distinguished` | `:5816` | `#[ignore]`d — dies at `:5779` |
| `nested_worker_depends_on_both_levels` | `:5895` | **live, un-ignored, passing** |

Both go through the same `compile_expr_for_lowering_tests`, the same `.run(None)`,
and the same decode path. The sibling was among the 778 passing rows at base
`21fd46dc` and the 783 at `fb99d0fc` — the two-ended census named exactly two
failures, and this was not one of them.

⇒ **Do not repair `run_worker_fixture`, and do not open with the decode path.**
The discriminator is the expression, and a working differential is already in the
tree.

### 1d. `token` is the native RETURN VALUE, not an error code

`compiled.rs:132`:

```rust
let token = native(process_root, services);
```

`NativeResultDecode { token: 9 }` therefore reads: **the compiled code returned
`9`, and the decoder could not turn `9` into a value.** It does not encode a
reason, an arm, or a category.

**Nothing in this node may be inferred from the numeral 9** beyond that. In
particular `9` is not an index into anything until `D1` says which decoder was
selected.

### 1e. Eight producers across five decoder kinds — the signature names no arm

`compiled.rs` raises this one variant at eight sites:

| line | arm | fires when |
|---|---|---|
| `135` | no decoder | `self.decoder` is `None` |
| `168` | `Int` | `native_int_arena().decode_final_export()` is `None` |
| `194` | `Boundary` / `PersistentGround` | `activation.finish(..)` errors |
| `197` | `Boundary` / `PersistentGround` | `activation.finish(..)` is `None` |
| `200` | `Boundary` / `PersistentGround` | `observe_adopted_ground` is `None` |
| `204` | `Boundary` `_ =>` | the word's tag is none of the three known tags |
| `211` | `Table` | `result_table` has no entry for the token |
| `213` | `TrapOnly` | unconditionally |

**A panic message reading `NativeResultDecode { token: 9 }` is consistent with
all eight.** Naming the arm is `D1`, and it is the whole diagnosis.

Decoder selection sites, so `D1` has somewhere to look:
`lowering/mod.rs:18181-18216` (`Int`/`Bool`/`Table`, by `Lowered` shape),
`lowering/units.rs:4113` (`Boundary`), `:4131` (`ProcessStatus`), `:4149`
(`TrapOnly`), `lowering/core.rs:1308`/`:1312`.

### 1f. Where the two fixtures differ — structural fact, NOT a diagnosis

Read this as an input to `D1`, not as an answer. It is the only structural
difference this frame asserts, and it is asserted about the **expressions**, not
about the failure.

- `two_same_shape_workers` (`:5712`) yields a `RuntimeExpr::Construct` over
  `"ctor:fixture::Pair::Both"` whose two args are worker `Call`s — a constructed
  aggregate.
- `nested_workers` yields a `Call` producing an `Int`.

They also differ in worker topology: two workers bound in one environment
(`:5712`) versus one worker nested inside another. **Both differences are live and
this frame does not rank them.** `D1` decides by measurement which one the
selected decoder turns on; picking either by inspection is the cheap wrong move.

> **Steward's hypothesis, recorded so the ring can kill it cheaply and is not
> anchored by it:** `intern_result` (`mod.rs:18383-18387`) assigns tokens from a
> compile-time counter and emits them as an `iconst`, so under `Table` the
> returned word is a constant and a miss means the code returned something other
> than the interned constant. That makes `:204` — an unrecognized `Boundary`
> tag — a plausible arm. **I did not run it. A dead hypothesis measured beats a
> live one assumed; report the arm you observe, not the arm named here.**

### 1g. What the row is currently claimed to prove

`constructors.rs:5816` and its doc comment claim `D5` and `AC-5`'s
target-redirect red: two same-shape workers are genuinely distinguished, so a
call resolving to the other's body is a redirected target. The three `assert_ne!`
comparisons are the stated evidence.

**They have never executed at any ref this program has measured.** Whether `AC-5`
was ever discharged is `D5`, and the honest answer may be "by nothing."

### 1h. Every anchor is perishable

Treat every fixed input above as perishable. If one turns out false against the
landed code at your base, **say so and escalate — do not quietly build around
it.** Four of the sibling node's fixed inputs were stale when it was released;
that is the expected rate, not an anomaly.

## 2. Deliverables

### `D0` — the delta-free baseline

On the cut branch, with no source change: run
`scripts/ken-cargo test -p ken-runtime --lib --no-fail-fast` and record absolute
passed/failed/ignored. Then run the ignored row by name with `--ignored` and
record its **exact** panic string.

Also record the sibling `nested_worker_depends_on_both_levels` **passing** in the
same run. That pair is the differential the rest of the node stands on; if it does
not reproduce, stop and report — §1c is then false and the node is re-scoped.

State the expected counts **before** running. `0 passed` is a failed measurement,
not a pass.

### `D1` — name the arm

Determine which of the eight `compiled.rs` sites raises the failure, and which
`ResultDecoder` the module carries for this expression. Report both.

Report it as an observation with its provenance — how you established it (a
temporary instrument is fine and is expected to be reverted), not as an inference
from the numeral or from §1f.

### `D2` — the fork ruling, and it may end the node

With the arm named, classify the defect into exactly one of:

- **(a) the fixture's expectation is wrong** — the expression asks for something
  the backend does not claim to support. In-node: repair the fixture, keeping the
  three comparisons meaningful.
- **(b) the decode path is incomplete** — the expression is legitimate and the
  decoder cannot carry its result. In-node if the repair is confined to
  `compiled.rs`/decoder selection and changes no lowering semantics.
- **(c) lowering is wrong** — the emitted code returns something it should not.
  **HARD STOP. Route to the Architect and stop.** That is a lowering-semantics
  change and it is not this node's to make.

**A fixture-only repair that leaves a real decode bug is the cheap false fix
here, and it is the specific outcome this deliverable exists to prevent.** State
the classification and the evidence for it before touching either file.

### `D3` — the repair

Conditional on `D2` landing (a) or (b). Confined to whichever `D2` named.

### `D4` — re-arm the detector, population-side

Un-ignore the row, delete the `#[ignore]` at `:5815` and the dark-state
annotation block at `:5789-5813`, and prove the restored detector is **live**,
not merely green. See `AC-3` for the required mutation and its operand.

**The `///` doc comment at `:5783-5788` is a different comment and stays** — it
states the property the row exists to prove. But re-read it against what the
restored row actually proves and correct it if they differ. It is the *leading*
comment; the block below it is the *nearer* one, and a restoration that updates
only the nearer one strands the leading claim, which is the sentence future
readers quote.

### `D5` — say what covers `AC-5` today

State plainly whether `AC-5`'s target-redirect red was ever discharged, and what
covers the property at the end of this node. **Route the answer; do not absorb
it.** If the answer is "nothing did, and this row now does," that is the
deliverable. If the answer is "nothing does, and this row still does not," say
that — `AC-5` belongs to a released node and its discharge claim is then false
in that node's record, which is the Steward's to correct.

### `D6` — the capture-order axis, CONDITIONAL

Only once `D4`'s detector is live.

`RT-SRCBODY-BIND-ORDER` reversed the Parameter run and left the Capture run in
descriptor order (`units.rs:4060-4068`), under
`source_body_binding_order` (`:3689-3699`), which returns `true` for
`CallableDeclaration` **and `ClosureBody`** — and `ClosureBody` is the unit kind
carrying a non-empty capture run. The covering comment at `:4057-4059` appeals to
descriptor order, **which is the ground that same commit refuted for the sibling
run.** The shape recurs at `:2600-2605`. All confirmed in source at `89916fc1`.

Use the restored detector to decide whether the capture run is correct by
construction or wrong, **with the elaborator's de Bruijn assignment across a
closure environment as the evidence — not the test's colour alone.** A green row
does not distinguish "captures are already right" from "this fixture does not
vary captures"; `two_same_shape_workers`'s `swap_second` parameter is what makes
it a discriminator, and `AC-4` requires showing it is.

**Not asserted as a bug and not to be repaired as one. There is no repro.** If
the measurement says the capture run is wrong, that is lowering semantics: HARD
STOP, route to the Architect, and this node ends there.

### `D7` — correct the comment at `:4057`

Whichever way `D6` resolves, `:4057-4059` must say so. If captures are correct by
construction, that fact belongs in the comment — the current sentence does not say
it, it appeals to the refuted ground. If `D6` hard-stops, record that the question
is open and route it; do not leave the refuted justification standing unmarked.

## 3. Acceptance criteria

Each row names its property, its operand, and the control. Record the result
**per row**, including the residual arms.

### `AC-1` — the differential is real

> **MEASURED:** at the `D0` base, `two_same_shape_workers_are_distinguished`
> fails with a recorded exact string and `nested_worker_depends_on_both_levels`
> passes, in one run.
> **CLAIMED:** the failure is a property of the expression, not of
> `run_worker_fixture`.
> **THE GAP:** both rows must reach `:5778`. A sibling that passes by not
> reaching the run step proves nothing.

Discharge by quoting both rows from the same `--no-fail-fast` output with the
executed count, plus evidence the sibling reaches `.run(None)`.

### `AC-2` — the arm is named, not inferred

> **MEASURED:** which `compiled.rs` site raised the failure, and the
> `ResultDecoder` selected for this expression.
> **CLAIMED:** `D2`'s classification rests on the actual arm.
> **THE GAP:** eight sites raise one variant. An arm asserted from the message
> text, from the numeral, or from §1f is not measured.

Assert the **exact** site, never `is_err`. Report how it was established. If a
temporary instrument was used, confirm it was reverted with `git diff --quiet`
(`--stat` always exits 0 and is not an emptiness test).

### `AC-3` — the restored detector reaches its population

> **MEASURED:** with the row un-ignored and passing, a **population-side**
> mutation reddens it.
> **CLAIMED:** the three `assert_ne!` comparisons execute and discriminate.
> **THE GAP:** a detector-side mutation — weakening an `expect`, narrowing a
> comparison — proves the row is wired to something, never that it reaches the
> workers.

**The operand that must move is the fixture expression, not the test body.**
Concretely: make `two_same_shape_workers`'s two workers genuinely identical so
the baseline and a swapped configuration must compare **equal**, and require the
row to redden. Restore byte-identically and verify with `git diff --quiet`.

Report, as its own field of the handoff: **the property · the operand that moved
· the observed boundary.** A handoff saying "the control reddened its intended
named test" does not answer this and will be sent back.

Count the anchor **before** mutating and compare against a predicted post-count;
do not re-match a needle the replacement may still contain.

### `AC-4` — the row varies captures

> **MEASURED:** the `swap_second` path produces a different linked result from
> the baseline.
> **CLAIMED:** the row is a discriminator for capture order, so `D6` can use it.
> **THE GAP:** the row can be green while `swap_second` changes nothing
> observable, in which case it discriminates bodies only and `D6` has no
> instrument.

If `swap_second` does not move the result, `D6` is **unbacked** — say so and stop
rather than reasoning from the row's colour. That is a legitimate outcome and it
has a cell here.

### `AC-5` — no un-ignore without a live detector

> **MEASURED:** the `#[ignore]` at `:5815` and the dark-state annotation at
> `:5789-5813` are gone, and `AC-3` discharged on the same SHA.
> **CLAIMED:** the row guards the property again.
> **THE GAP:** deleting the annotation while `AC-3` is undischarged reproduces
> the exact condition this node was filed for, with a green row on top.

`AC-3` and `D4` land in the same commit or neither does.

### `AC-6` — no regression, in CI

Workspace-green **in CI**, never a local `--workspace` run (`COORDINATION §12`).
Locally: `-p ken-runtime`, scoped, through `scripts/ken-cargo`.

Report absolute passed/failed/ignored at the candidate **and** at the base. "No
new red" measured against a mid-node checkpoint is not the reading required here.

### `AC-7` — per-pin evasion

For **each** of `AC-1`, `AC-3`, `AC-4`: attempt one compile-preserving evasion and
record it. Where you cannot construct one, say **why the surface is closed**,
grounded on the visibility of the reachable surface — not on the files you
happened to read. Three rows, three recorded attempts; a single attempt against
the most salient row does not discharge this.

### `AC-8` — the residual

State what this node leaves review-guarded rather than mechanically guarded, and
name **every** such arm. If `D2` ruled (c) or `D6` hard-stopped, the residual
includes the routed question and the node closes with it open. That is a complete
discharge of this row, not a failure.

## 4. Banned scope

- **Do not weaken or delete either `expect` in `run_worker_fixture`.** Making the
  panic go away is not making the fixture run.
- **Do not repair the row by relaxing the three `assert_ne!` comparisons**, and
  do not reduce them to two.
- **Do not modify `nested_worker_depends_on_both_levels`.** It is the control.
- **Do not change lowering semantics.** `units.rs:4060-4068`, `:2600-2605`, and
  `source_body_binding_order` are read-only in this node. A finding against them
  routes to the Architect (`D6`).
- **Do not re-baseline any measurement.** Expect reds and attribute each
  individually.
- **Do not touch the four `px4b` rows, the 30-row `#[ignore]` population, or any
  row owned by `RT-CARRIER-BYTESPAN-OBSERVE` or
  `RT-CARRIER-PRODUCER-OCCURRENCE`.** `c2_ac4_...` at `constructors.rs:2549` is
  the sibling node's, not this one's, even though it sits in the same file and
  was annotated in the same commit.

## 5. Hard stop

Stop and route, rather than continuing, if any of:

- `D0` does not reproduce the §1c differential.
- `D2` rules **(c)** — lowering is wrong. To the **Architect**.
- `D6` measures the capture run as wrong. To the **Architect**.
- The repair for (a) or (b) reaches outside the file `D2` named.
- `AC-4` shows the row does not vary captures — `D6` has no instrument.

A hard stop inside the hour is a good outcome. Neither a releasable increment nor
a hard stop is the bad one.

## 6. Contention

- **Files:** `constructors.rs` and, conditionally, `compiled.rs`. Both are
  `crates/ken-runtime`, which is `RT-CARRIER-BYTESPAN-OBSERVE`'s crate.
  **This node must not run concurrently with that one** — it is sequenced behind
  it, which the single-threaded posture already enforces.
- **`library/`:** untouched. No Librarian review, no attestation fold.
- **Reviewers:** Architect on the merge Decision (`crates/`). No Spec vote —
  nothing under `spec/` or `conformance/`.
- **Corpus oracles:** no file is added to a globbed corpus, so the
  `adding-a-file-to-a-globbed-corpus-trips-oracles-you-did-not-enumerate` fan-out
  does not apply. If `D3` adds one, enumerate its derived consumers before
  predicting the affected population.
