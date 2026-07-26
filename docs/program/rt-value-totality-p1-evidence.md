# `RT-VALUE-TOTALITY-P1` — measurement, AC→control map, evasion table

Companion evidence for `wp/RT-VALUE-TOTALITY-P1`. The frame is authoritative;
this records what was **measured**, which control discharges each AC, and what
Phase 1 does **not** close.

## 1. `AC-V3a` — the partial-move measurement that selected the D3 mechanism

⛔ The frame labelled its own grep an **estimate** and required a real
measurement. A grep cannot see multi-line signatures, `mut` bindings, or macro
expansion, so the measurement was taken **with the compiler**: a temporary
`impl Drop for Value` was added and every consuming crate compiled with
`cargo check --all-targets`, collecting `E0509` ("cannot move out of type which
implements the `Drop` trait").

| crate | consumes `ken_runtime::Value`? | `E0509` sites |
|---|---|---|
| `ken-runtime` | owns it | **0** |
| `ken-interp` | yes | **0** |
| `ken-cli` | yes | **0** |
| `ken-elaborator` | yes | **0** |
| `ken-verify` | yes | **0** |
| `ken-foundation` | **no** — see §5 | n/a |

All five checks exited 0 with 0 errors **while the `Drop` impl was in place**, so
the zero is not a compile that never happened.

**Two-sided positive control**, because a zero-count negative check passes for
any reason:

| probe state | deliberate partial move | result |
|---|---|---|
| `impl Drop` present | yes | **`E0509`** at `values.rs:126`, exactly 1 |
| `impl Drop` removed | yes | **clean** |

⇒ The probe is live and the error is causally attributable to `Drop`.

**Mechanism selected: D3 family 1** — `impl Drop for Value` with an explicit
dismantling stack. The measured cost of its `E0509` movability constraint is
**zero sites**, and it leaves the public field *types* untouched, so no consumer
construction site moves. Family 2 (a newtype carrying `Drop` on the child
containers) would have changed public field types and forced churn in
`ken-interp`, `ken-cli` and `ken-elaborator` to buy nothing. Re-verified after
the real change landed: all five crates still `check` clean, 0 `E0509`.

## 2. `D` was measured, not chosen

Bisected out of process against the **landed** mechanisms before they were
replaced, at two thread stack sizes (ambient `ulimit -s` = 8192 KiB):

| thread stack | recursive `encode` | derived `Clone` | drop glue |
|---|---|---|---|
| 1 MiB | last ok 1121 / **died 1122** | 1252 / **1253** | 8142 / **8143** |
| 8 MiB | last ok 9031 / **died 9032** | 10074 / **10075** | 65486 / **65487** |

**`D = 131072`** exceeds every threshold at **both** stack sizes — 16x the 1 MiB
drop threshold and 2.0x the 8 MiB one — so it cannot be dismissed as an
artificially small stack. All six (mechanism, stack) pairs were confirmed to
abort at exactly this `D` (SIGABRT, exit 134).

Each control pins its own thread stack to a stated 1 MiB so `D`'s adequacy is a
property of a **declared** stack rather than of the ambient limit, which is not
guaranteed equal in CI.

**Reproduction.** Base `b445cd15`, worktree
`/workspaces/ken/.worktrees/runtime-implementer`, sanctioned invocation:

```sh
source scripts/ken-env.sh
scripts/ken-cargo test -p ken-runtime --test value_depth_totality
```

## 3. AC → control map

| AC | control | operand moved | where |
|---|---|---|---|
| `AC-V1` step 1 | the bisect in §2 | population | recorded here + test module header |
| `AC-V1` step 2 | `ac_v1_a_host_recursive_traversal_dies_on_this_population_at_depth_d` | **population** | `tests/value_depth_totality.rs` |
| `AC-V1` step 2 (all child positions) | `ac_v1_host_recursion_dies_at_depth_d_through_all_child_positions` | **population** | same |
| `AC-V1` step 3 | `ac_v1_new_encoder_emits_exact_closed_form_bytes_at_depth_d` | subject | same |
| `AC-V1` step 3 (all child positions) | `ac_v1_new_encoder_exact_bytes_at_depth_d_through_all_child_positions` | subject | same |
| `AC-V1b` | `ac_v1b_iterative_encoding_is_byte_identical_to_the_recursive_reference` + non-vacuity and coverage arms | subject | `src/canonical.rs` |
| `AC-V2a` | compile-fail control, §4 row A | subject (the type) | manual, recorded |
| `AC-V2b` | compile-fail control, §4 row B | subject (the type) | manual, recorded |
| `AC-V2c` | `ac_v2c_interning_a_nested_compound_mints_exactly_one_slot` | **population** | `src/store.rs` |
| `AC-V3a` | §1 | population | manual, recorded |
| `AC-V3b` | `ac_v3b_clone_then_drop_both_copies_at_depth_d` (+ all-child-positions twin) | subject | `tests/value_depth_totality.rs` |
| `AC-V3c` (Clone) | `ac_v3c_recursive_clone_dies_on_the_real_carrier_at_depth_d` | **population** | same |
| `AC-V3c` (drop) | `ac_v3c_derived_drop_glue_dies_on_the_analogue_at_depth_d` | **population**, on an analogue — see §5 | same |
| `AC-V3d` | `ac_v3d_drop_alone_is_total_at_depth_d` (+ all-child-positions twin) | subject | same |
| `AC-V4` | §6 | — | recorded here |
| `AC-V5` | §4 | — | recorded here |

Plus `harness_can_observe_a_survivor`, so the survival assertions are not
vacuous, and every survival scenario asserts a **completion marker** — a bare
exit code cannot distinguish *"did the work"* from *"never ran it."*

## 4. `AC-V5` — evasion attempts, one row per named AC

Every attempt was compile-preserving and actually **run**.

| # | AC | evasion attempted | outcome |
|---|---|---|---|
| 1 | `AC-V1` step 3 | **hybrid encoder**: iterative for `Record`, still host-recursive for `Constructor`/`Array`/`Map`/`Closure` | ⛔ **SUCCEEDED against the original control set** — repaired, see below |
| 2 | `AC-V1b` | move a `tag` constant so **both** differential sides shift together | survived `AC-V1b` as documented; **caught** by the integration oracle's own format copy |
| 3 | `AC-V2c` | detector-side: make `distinct_count()` constantly `1` | **caught** — the pin's pre-condition asserts the count is `0` before interning (plus 3 pre-existing store tests) |
| 4 | `AC-V3b` | degenerate `Clone` returning a leaf — clones fine, drops fine, wrong content | **caught** — the scenario asserts clone bytes equal the closed form, not merely "did not crash" |
| 5 | `AC-V3c` | die for the **wrong reason** — panic instead of exhausting the stack | **caught** — the assertion also requires the runtime's stack-overflow diagnostic on stderr |
| 6 | `AC-V3d` | cap the shared chain builder so depth never reaches `D` | **not caught by `AC-V3d`'s own assertion**; caught by the byte oracle and the death control, which share the population builder |

⚠ `AC-V2a` and `AC-V2b` are themselves mutation controls and correctly get no
second attempt — stated rather than omitted, since an omitted row and a skipped
attempt look identical.

### Row 1 was a real gap, and it is now closed

Attempting row 1 honestly showed that **every depth control used a unary
`Record` chain**, so a hybrid encoder passes all of them while leaving **four of
the five** original recursion sites intact. Repair: `mixed_chain`, which cycles
through all five child positions with its own independently-derived closed-form
byte expectation, and a host-recursive traversal over the same surface.

Proof the repair works, and that the original set could not:

| control | hybrid encoder (`Array` recursive) |
|---|---|
| `..._through_all_child_positions` (new) | **FAILED** — catches it |
| `...at_depth_d` (unary `Record`, original) | **still green** — blind to it |

### The compile-fail controls, verbatim

**Row A — `AC-V2a`.** `Record.fields` changed to `Vec<Rc<Value>>`:

```
error[E0277]: the trait bound `Vec<Rc<values::Value>>: OwnedChildren` is not satisfied
   --> crates/ken-runtime/src/canonical.rs:211:35
211 |             child_positions::push(fields, stack);
    |             --------------------- ^^^^^^ the trait `OwnedChildren` is not implemented
help: the trait `OwnedChildren` is implemented for `Vec<values::Value>`
note: required by a bound in `push`
```

4 errors total: this `E0277` is the pin firing; the other three are `E0308`
collateral from the type change (`detach_children`, `rebuild`, `Clone`).

**Row B — `AC-V2b`.** A throwaway variant with a `Vec<Value>` child produces
`E0004` non-exhaustive-patterns at **four** independent pins:

| site | pin |
|---|---|
| `canonical.rs:168` | `encode_header` — `D2` clause 1 |
| `values.rs:144` | `detach_children` — Drop's traversal |
| `values.rs:185` | `rebuild` — Clone's reassembly |
| `values.rs:303` | Clone's `Job::Visit` dispatch |

Both mutations were restored byte-identically and verified with
`git diff --quiet` (⚠ `--stat` always exits 0 and is not an emptiness test).

## 5. Honest residuals — what Phase 1 does NOT close

⛔ **No cycle-refusal control exists on this carrier, and that is correct, not an
omission.** A back-edge in `Value` is *unconstructible* (`evt_45x5dn9jcrhhq`),
so an AC requiring a cycle witness here is unsatisfiable and its only available
control would be detector-side. The property is pinned structurally by the
compiler instead, and the obligation was retargeted onto `RT-FNSPLIT-B2V`'s
`BoundaryPersistentImage`. Stated in the test module so a reader inherits the
reason rather than reading an unexplained absence.

**⚠ Four other derived traversals of `Value` are still host-recursive.**
Measured out of process at `D`, on the landed post-change code:

| traversal | at `D` = 131072 |
|---|---|
| derived `Debug` | **dies** (stack overflow) |
| derived `PartialEq` | **dies** (stack overflow) |
| derived `Ord` | **dies** (stack overflow) |
| derived `Hash` | **dies** (stack overflow) |

Phase 1's scope is `encode_canonical`, `Clone` and drop, and those three are now
total. ⛔ **But "`Value` traversals are total" is false**, and two things follow:

- **`Debug` has no cell in the frame's §7 residual table at all.** It is
  reachable from ordinary diagnostic code — a `{:?}` in a panic message or a log
  line — so a deep value can abort the process through a path nobody would read
  as a traversal.
- §7 *does* have a cell for `PartialEq`/`Eq`/`PartialOrd`/`Ord`/`Hash`, but it is
  scoped to those derives **disagreeing with canonical identity**. Their
  **totality** is a different property and is uncelled.

⛔ Not fixed here, deliberately: the frame assigns the derive list to Phase 2 and
forbids changing it. `Clone` was removed from the derive **only** because `D3`
mandates a hand-written iterative one; `Debug`, `PartialEq`, `Eq`, `PartialOrd`,
`Ord` and `Hash` are untouched.

**⚠ `ken-foundation` carries an independent twin with the same defect.**
`crates/ken-foundation/src/values.rs` declares its own `Value` and
`crates/ken-foundation/src/canonical.rs` its own recursive encoder, with the same
five recursion sites (`:95`, `:104`, `:128`, `:146`, `:169`). The crate has **no
`[dependencies]` at all** — std-only, deliberately self-contained — so it is
**not** a consumer of `ken-runtime::Value` and was correctly outside the `AC-V3a`
population. §7 assigns the twin to Phase 2; the kickoff forbids touching it.

**⚠ The `AC-V3c` drop-half control runs on an analogue, not on `Value`.**
`GlueTwin` exercises the same *mechanism class* — a nested owned collection torn
down by compiler-generated glue — but not `Value`'s own glue, which no longer
exists because replacing it is this WP's deliverable. The evidence for the
genuine article is the §2 bisect, measured against the landed code before it was
replaced. Stated rather than glossed: a pin that never exercises the violating
mechanism is not evidence about it.

## 6. `AC-V4` — corpus oracles that could see the new test target

`crates/ken-runtime/tests/` is new, so every corpus-wide oracle was enumerated.

| oracle | scans | binds here? |
|---|---|---|
| `crates/ken-cli/tests/ken_fmt.rs` | `.ken`, `.md` | **no** — never `.rs` |
| `crates/ken-cli/tests/library_documentation_gates.rs` | `.md`, `.toml`, `.ken` | **no** |
| `library/SOURCE-ATTESTATIONS` | 51 rows, 11 `crates/` rows | **no** — none of the four touched paths is cited |
| repo-tree walkers in `crates/*/tests` | — | **none exist**: no `read_dir` / `walkdir` / `.rs` enumeration anywhere under `crates/*/tests` |
| `.github/workflows/ci.yml` | — | **no** test-target enumeration, and **no `cargo fmt` gate** |
| Cargo autodiscovery | `crates/ken-runtime/tests/*.rs` | **yes, and it works** — no `autotests` key, no `[[test]]`; the target was discovered and 12 tests ran |

⇒ **No corpus-wide oracle binds on the new test target.** Written as a sentence
because a silent absence and a checked absence read identically.

The attestation-ledger check carries a **positive control**: the grep was shown
to detect a path that *is* cited (`crates/ken-runtime/src/cranelift_backend.rs`),
so "not cited" is distinguishable from "never looked". That attested file is
untouched and was not re-attested.

⚠ **Pre-existing, not mine:** `cargo fmt --check` reports 9 diffs in
`crates/ken-runtime/src/cranelift_backend/lowering/core/tests/control.rs`, last
touched by `c986d0a3` (`RT-FNSPLIT-B2R`). All four files this WP touches are
individually fmt-clean. Left alone deliberately — CI does not gate on fmt, and
reformatting an unrelated file would smuggle churn into a scoped diff.
