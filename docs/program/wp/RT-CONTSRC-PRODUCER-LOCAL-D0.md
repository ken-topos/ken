# `RT-CONTSRC-PRODUCER-LOCAL` `D0` — the delta-free parity baseline

**Measured at exact `179af86350ba7191935fcc9ff902bb166c954339`, on branch
`wp/RT-DECL-CLOSURE-PORT-typed-units`, with a clean tree and zero delta of my
own.** This file is committed *before* any `D1` code so that the record it
carries cannot have been produced by the change it is the baseline for.

Command:

```
./scripts/ken-cargo test -p ken-cli --test rt_parity_native -- --test-threads=1 --nocapture
```

Single-threaded deliberately: parallel test threads interleave captured output,
which is how a previous census in this campaign acquired torn records.

Suite total, for orientation only — ⛔ the total is **not** the evidence, and
`AC-1b` forbids using it as such: `1 passed; 6 failed; 0 ignored`, 126.66s.

## The seven rows, individually

| row | verdict at `179af863` | refusal text now raised |
|---|---|---|
| `buffer_freeze_malformed_span_is_unconstructible_at_the_landed_surface` | **green** | — (this is the green control) |
| `buffer_allocate_malformed_capacity_narrows_to_invalid_bounds` | red | `Match: scrutinee is not a constructor value` |
| `fs_read_at_malformed_offset_narrows_to_invalid_offset` | red | `ComputationalMatch: tree-producing match scrutinee is not Bool or a constructor` |
| `fs_read_at_malformed_offset_without_read_right_narrows_to_invalid_offset` | red | `ComputationalMatch: tree-producing match scrutinee is not Bool or a constructor` |
| `fs_read_at_malformed_window_narrows_to_invalid_bounds` | red | `ComputationalMatch: tree-producing match scrutinee is not Bool or a constructor` |
| `fs_write_at_malformed_offset_narrows_to_invalid_offset` | red | `ComputationalMatch: tree-producing match scrutinee is not Bool or a constructor` |
| `fs_write_at_malformed_offset_without_write_right_narrows_to_invalid_offset` | red | `ComputationalMatch: tree-producing match scrutinee is not Bool or a constructor` |

Every red row raises its text through the same path — the fixture reaches linked
native lowering and `checked_process_object` refuses at `ObjectEmission`:

```
Packaging(ObjectLinkerPackagingError {
  stage: ObjectEmission,
  field: "checked_process_object",
  reason: "unsupported runtime-IR lowering: <text above>" })
```

and the fixture thread then re-panics at `rt_parity_native.rs:468`.

Per-row fixture case names, so a later run can be compared row by row rather
than in aggregate:

| row | fixture case |
|---|---|
| `buffer_allocate_...` | `buffer-allocate-single` |
| `fs_read_at_malformed_offset_...` | `fs-read-at-offset-single` |
| `fs_read_at_..._without_read_right_...` | `fs-read-at-offset-overlap` |
| `fs_read_at_malformed_window_...` | `fs-read-at-window-single` |
| `fs_write_at_malformed_offset_...` | `fs-write-at-offset-single` |
| `fs_write_at_..._without_write_right_...` | `fs-write-at-offset-overlap` |

## The one asymmetry worth recording

**The `AC-1` linked row is the only row that does not fail like the others.**
`buffer_allocate_malformed_capacity_narrows_to_invalid_bounds` refuses at
`Match: scrutinee is not a constructor value`; the other five refuse at
`ComputationalMatch: tree-producing match scrutinee is not Bool or a
constructor`. Two distinct refusal sites, and `AC-1`'s row is alone on its side.

⇒ A later run that turns the five `ComputationalMatch` rows green tells you
nothing about the `Match` row, and the reverse. **Recorded here rather than
inferred later**, because the five-and-one split is invisible in the `1 passed /
6 failed` total that `AC-1b` already forbids relying on.

## Census unit

⛔ **This file states no closure-edge population and no first-`Open` statistic.**
Per the node's §6, any census this node produces states its unit and answers
*what does this edge require*. The population figures carried into this node
(34 case-binder-only, 4 effect-result-plus-case-binder, 1 `Construct`-only) are
the Architect's, over full required environment vectors, and are re-measured
under `D4`, not asserted here.

## Adjacent baselines at the same commit

Taken from the node's §1 and independently re-measured by me at this exact
commit before this node was cut: `ken-runtime` lib **718 passed / 2 failed**
(the two standing reds
`c2_ac4_runtime_host_result_selects_a_separately_generated_nested_payload` and
`two_same_shape_workers_are_distinguished`), `ken-elaborator` lib
**108 passed / 0 failed**.
