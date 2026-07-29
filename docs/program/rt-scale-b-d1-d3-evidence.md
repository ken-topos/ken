# `RT-SCALE-B` D1--D3 empirical evidence

## Binding and scope

The governed source and completed-emission mechanism are bound to
`origin/main` at
`38e054d47c3f9a01e1adf844b4a632e60087a4c4`. The frame is
`docs/program/wp/RT-SCALE-B-emission-scaling-verdict.md`, blob
`d62bab7587121b3ff6c7427aec7bf619f0675977`.

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

## D3 differential baseline

**NO CONTROL — open residual.** The frame requires the exact pre-existing
normal/abrupt/trap/join/affine suite and a baseline recipe naming its probe
functions. Neither the frame nor the merged source identifies that five-probe
mapping. Runtime has routed the identity gap to the frame/design owner and
will not invent a substitute suite or fabricate a baseline.

## `AC` to control map

| AC | control and evidence |
|---|---|
| `AC-B1` | Forced-indeterminate and omitted-result subprocess controls in `rt_scale_b_governed_n3_through_n7_collect_every_d2_metric`. |
| `AC-B2` | The same test requires all 44 numeric fields in every completed row. |
| `AC-B3` | The parent parses every row and emits first and second differences for every numeric field. |
| `AC-B4` | Four independent structural assertions plus this absolute table; no exponent is inferred from the five points. |
| `AC-B4a` | Typed emitter-category counters, exact 7/29/1/unit counts, and a zero RecursiveDescent count. |
| `AC-B5` | **NO CONTROL — open residual.** |
| `AC-B6` | `docs/program/rt-scale-b-d4-analytical-model.md`; Architect-owned D4. |
| `AC-B7` | **NO CONTROL — open residual.** D5 final routing is not yet complete. |
| `AC-B8` | This artifact reports absolute values and labels the historic datum NON-COMPARABLE. |
| `AC-B9` | This map; open residuals use the frame's exact spelling. |

The completed governed rows measure recursive lowering frames
`[18, 23, 28, 33, 38]`. D4 separately consumes RT-SCALE-A's frame measurement
`[14, 18, 22, 26, 30]`, as the frame requires. Neither series is substituted
for the other; their source/phase distinction remains explicit pending the
final D5 reconciliation.
