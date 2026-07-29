# `RT-SCALE-B` D1--D3 empirical evidence

## Binding and scope

The governed source and completed-emission mechanism are bound to
`origin/main` at
`370281d0519c7503ee38caa6bf132cc0bd929176`. The frame is
`docs/program/wp/RT-SCALE-B-emission-scaling-verdict.md`, blob
`1f7ca6e514c550f7c256eb44a89c7641abfec812`.

The historic n=4 figures, including approximately 103 seconds, approximately
4 GiB, and 1,482/1,525 states/edges, are **NON-COMPARABLE**. They did not use
this source, completed phase boundary, or metric schema. The absolute rows
below stand alone.

The permanent harness is
`rt_scale_b_governed_n3_through_n7_collect_every_d2_metric`. It runs each
depth in a separate `prlimit` process with:

- 30 seconds of CPU;
- 4 GiB of address space;
- the product's 8 MiB stack; and
- a 45-second parent-side termination deadline.

A forced failure and a missing-result worker establish the third
`could_not_determine` outcome. A worker must finish emission and publish every
numeric field before the parent accepts its row.

## Absolute completed-emission rows

The following rows are one bounded run of the permanent harness. Wall time and
peak RSS are observations, not deterministic assertions.

| n | wall ns | RSS KiB | semantic states | defined units | emitted units | all functions |
|---:|---:|---:|---:|---:|---:|---:|
| 3 | 36,018,784 | 16,940 | 96 | 4 | 4 | 41 |
| 4 | 34,293,497 | 16,984 | 127 | 5 | 5 | 42 |
| 5 | 38,958,791 | 16,748 | 158 | 6 | 6 | 43 |
| 6 | 66,355,579 | 16,928 | 189 | 7 | 7 | 44 |
| 7 | 64,922,191 | 17,020 | 220 | 8 | 8 | 45 |

| n | CLIF instructions | CLIF bytes | descriptor construction | descriptor comparison | DFG values | instructions | blocks |
|---:|---:|---:|---:|---:|---:|---:|---:|
| 3 | 5,285 | 202,933 | 25 | 25 | 3,467 | 5,285 | 1,767 |
| 4 | 6,469 | 247,932 | 31 | 31 | 4,158 | 6,469 | 2,216 |
| 5 | 7,653 | 292,931 | 37 | 37 | 4,849 | 7,653 | 2,665 |
| 6 | 8,837 | 337,930 | 43 | 43 | 5,540 | 8,837 | 3,114 |
| 7 | 10,021 | 382,929 | 49 | 49 | 6,231 | 10,021 | 3,563 |

| n | static nodes | edges | planned helpers | persistent nodes | evidence records | fixed K |
|---:|---:|---:|---:|---:|---:|---:|
| 3 | 96 | 112 | 208 | 148 | 112 | 8 |
| 4 | 127 | 149 | 276 | 190 | 149 | 8 |
| 5 | 158 | 186 | 344 | 232 | 186 | 8 |
| 6 | 189 | 223 | 412 | 274 | 223 | 8 |
| 7 | 220 | 260 | 480 | 316 | 260 | 8 |

| n | logical | environment | continuation | path | cleanup | affine | source return | recursive frames |
|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| 3 | 18 | 5 | 18 | 18 | 3 | 3 | 3 | 18 |
| 4 | 23 | 6 | 23 | 23 | 4 | 4 | 4 | 23 |
| 5 | 28 | 7 | 28 | 28 | 5 | 5 | 5 | 28 |
| 6 | 33 | 8 | 33 | 33 | 6 | 6 | 6 | 33 |
| 7 | 38 | 9 | 38 | 38 | 7 | 7 | 7 | 38 |

For every row, source-return resume nodes and owned-resume edges equal n,
terminal outgoing edges equal zero, and stack bytes equal 8,388,608.
Helper-key, activation-frame, and store-node widths are respectively
12/32/16 bytes. Static-node and persistent-node IDs are each four bytes.
Helper-key, activation-frame, and store-node schema counts are each one.

## First and second finite differences

The permanent harness emits both vectors for every numeric field. The
completed structural and emission rows are:

| metric | first differences | second differences |
|---|---|---|
| semantic states | `[31, 31, 31, 31]` | `[0, 0, 0]` |
| defined units | `[1, 1, 1, 1]` | `[0, 0, 0]` |
| emitted units | `[1, 1, 1, 1]` | `[0, 0, 0]` |
| all functions | `[1, 1, 1, 1]` | `[0, 0, 0]` |
| CLIF instructions | `[1184, 1184, 1184, 1184]` | `[0, 0, 0]` |
| CLIF bytes | `[44999, 44999, 44999, 44999]` | `[0, 0, 0]` |
| descriptor construction | `[6, 6, 6, 6]` | `[0, 0, 0]` |
| descriptor comparison | `[6, 6, 6, 6]` | `[0, 0, 0]` |
| DFG values | `[691, 691, 691, 691]` | `[0, 0, 0]` |
| total instructions | `[1184, 1184, 1184, 1184]` | `[0, 0, 0]` |
| blocks | `[449, 449, 449, 449]` | `[0, 0, 0]` |
| static nodes | `[31, 31, 31, 31]` | `[0, 0, 0]` |
| edges | `[37, 37, 37, 37]` | `[0, 0, 0]` |
| planned helpers | `[68, 68, 68, 68]` | `[0, 0, 0]` |
| persistent nodes | `[42, 42, 42, 42]` | `[0, 0, 0]` |
| evidence records | `[37, 37, 37, 37]` | `[0, 0, 0]` |
| logical/continuation/path depth | `[5, 5, 5, 5]` | `[0, 0, 0]` |
| environment/cleanup/affine/return depth | `[1, 1, 1, 1]` | `[0, 0, 0]` |
| source-return nodes/edges | `[1, 1, 1, 1]` | `[0, 0, 0]` |
| recursive frames | `[5, 5, 5, 5]` | `[0, 0, 0]` |

Native-Int functions, boundary-value functions, recursive-descent roots,
functionized root adapters, fixed K, all widths, all schema counts, terminal
outgoing edges, and stack bytes have first differences `[0, 0, 0, 0]` and
second differences `[0, 0, 0]`.

The resource observations are deliberately not required to be affine:

| metric | first differences | second differences |
|---|---|---|
| wall ns | `[-1725287, 4665294, 27396788, -1433388]` | `[6390581, 22731494, -28830176]` |
| peak RSS KiB | `[44, -236, 180, 92]` | `[-280, 416, -88]` |

These five points do not establish an exponent. The structural invariants
discriminate; the table corroborates.

## Structural invariants

1. **No flattened environment, pending, or path member in helper identity:
   satisfied.** `PlannedHelperKey` is exactly a transition or edge tag plus its
   static ID. The permanent control fixes its payload at 12 bytes. Adding a
   third dynamic member widened it to 16 bytes and failed at the named
   invariant-1 assertion; the mutation was restored byte-identically.
2. **Constant ID and node payload width: satisfied.** Every width and schema
   count is pairwise equal across n=3..7.
3. **Affine total persistent nodes: satisfied.** The first difference is 42
   and every second difference is zero.
4. **At-most-affine logical chain depth: satisfied.** The first difference is
   five and every second difference is zero. Constant chain depth is not
   required; each frame carries one fixed-width persistent ID.

## Production Cranelift denominator

The empirical denominator is the closed set of production definitions emitted
by this compilation:

| emitter | row count | disposition |
|---|---:|---|
| `native_int_clif.rs` local graph | 7 | included |
| `boundary_value_clif.rs` local graph | 29 | included |
| `lowering/units.rs` unit bodies | n+1 | included |
| `lowering/units.rs` root adapter | 1 | included |
| `lowering/core.rs` RecursiveDescent root | 0 | mutually exclusive and asserted absent |

Imports have no body in the module. Test probes, generated C starters, and
linker stubs are not production Cranelift emitters. Every included category
records through the same completed-function seam. Misclassifying all seven
native-Int definitions as boundary-value definitions failed at the exact
native-Int population assertion and was restored byte-identically.

## D3 five-category differential coverage

The five labels are coverage categories, not a named probe suite. Every named
positive control below compiles and runs the same source through the completed
native representation and the live interpreter, then compares the relevant
public observation.

| category | landed test functions | completed-representation result |
|---|---|---|
| normal | `px8l_recursive_decl_native.rs::{dynamic_zero_seed_takes_the_base_case,dynamic_multistep_seed_preserves_updated_parameter_order}`; `px8f_buffer_native.rs::linked_checked_write_all_observes_short_progress_and_matches_interpreter` | 3/3 recursive-declaration file and 1/1 checked-buffer file passed |
| abrupt | `rt_parity_native.rs::{buffer_allocate_malformed_capacity_narrows_to_invalid_bounds,fs_read_at_malformed_offset_narrows_to_invalid_offset,fs_read_at_malformed_window_narrows_to_invalid_bounds,fs_read_at_malformed_offset_without_read_right_narrows_to_invalid_offset,fs_write_at_malformed_offset_narrows_to_invalid_offset,fs_write_at_malformed_offset_without_write_right_narrows_to_invalid_offset}`; `rt_escape_second_resource_native.rs::r2_cross_buffer_freeze_fails_closed_with_invalid_bounds` | 7/7 parity file and 6/6 escape/resource file passed; the named cases agree on exact narrowed errors |
| trap | **NO COVERAGE — open residual** | runtime-local `native_execution_differential.rs::tests::trap_observation_is_first_class_unavailable_native_lane` passed 1/1, confirming native trap comparison remains first-class unavailable |
| join | `rt_escape_second_resource_native.rs::{escaped_resource_used_by_fanning_host_op_matches_interpreter,escaped_buffer_used_by_fanning_host_op_matches_interpreter,nat_fanout_escaped_resource_matches_interpreter}` | 6/6 file passed; the named dynamic Result/Nat fan-out controls fork mutually exclusive arms and union at rejoin |
| affine | `rt_escape_second_resource_native.rs::{escape_one_used_matches_interpreter,escape_resource_plus_plain_matches_interpreter}` | 6/6 file passed; the named controls compare exact bracket observations while preserving checked-frame consumption and sibling isolation |

The trap residual is routed to `RT-EFFECT-DIFF`. This node does not build a
comparator, fixture schema, or second corpus.

The commands used for the category map were:

```text
scripts/ken-cargo test -p ken-cli --test px8l_recursive_decl_native -- --test-threads=1
scripts/ken-cargo test -p ken-cli --test px8f_buffer_native -- --test-threads=1
scripts/ken-cargo test -p ken-cli --test rt_parity_native -- --test-threads=1
scripts/ken-cargo test -p ken-cli --test rt_escape_second_resource_native -- --test-threads=1
scripts/ken-cargo test -p ken-runtime --lib native_execution_differential::tests::trap_observation_is_first_class_unavailable_native_lane -- --exact --test-threads=1
```

## D5 terminal verdict

The independent D4 model reconciles the completed D1--D3 evidence as outcome
**(a)**:

> Outcome (a): the completed FunctionizedUnits representation is empirically
> affine in every deterministic material and structural population; the
> independent analytical model predicts Theta(n) emitted material and no
> inherent semantic product for this governed family. The observed wall-time
> and peak-RSS samples are noisy and are not used to claim an exponent. The
> historical n=4 observation remains NON-COMPARABLE. This is a
> representation-growth verdict, not a completeness or verification claim;
> RT-EFFECT-DIFF and the recorded trap differential residual remain open.

The D4 constants-reduction plan remains the required follow-on. It is not
evidence that any historical constant was reproduced.

## `AC` to control map

| AC | control and evidence |
|---|---|
| `AC-B1` | Forced-indeterminate and omitted-result subprocess controls in `rt_scale_b_governed_n3_through_n7_collect_every_d2_metric`. |
| `AC-B2` | The same test requires all 44 numeric fields in every completed row. |
| `AC-B3` | The parent parses every row and emits first and second differences for every numeric field. |
| `AC-B4` | Four independent structural assertions plus this absolute table; no exponent is inferred from the five points. |
| `AC-B4a` | Typed emitter-category counters, exact 7/29/1/unit counts, and a zero RecursiveDescent count. |
| `AC-B5` | The five-row D3 coverage map above; trap is explicitly `NO COVERAGE — open residual`. |
| `AC-B6` | `docs/program/rt-scale-b-d4-analytical-model.md`; Architect-owned D4. |
| `AC-B7` | Outcome **(a)**, reconciled at `docs/program/rt-scale-b-d4-analytical-model.md`'s D5 boundary and stated verbatim above; wall time and RSS do not establish an exponent. |
| `AC-B8` | This artifact reports absolute values and labels the historic datum NON-COMPARABLE. |
| `AC-B9` | This complete map; the D3 trap row retains the frame's exact `NO COVERAGE — open residual` spelling and route. |

The completed governed rows measure recursive lowering frames
`[18, 23, 28, 33, 38]`. D4 separately consumes RT-SCALE-A's frame measurement
`[14, 18, 22, 26, 30]`, as the frame requires. Both are genuine production
stack measurements under the same guard, but their source fixtures differ:
the completed Scale-B source adds a closure-body `Let`, a direct induction
hypothesis call, and the canonical four `BufferFreeze` operands. Neither
series is substituted for the other and their coefficients are not compared;
the difference is fixture shape, not emission overhead.
