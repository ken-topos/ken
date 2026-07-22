# RT-SPLIT slice 5 — semantic-subject ledger for the residual `mod tests`

Required by the Architect's ruling `evt_3xvn8g7n5rv7m`. Durable here rather
than in-thread, because an in-thread ruling is not a durable deliverable.

## The rule this ledger applies

A test's subject is the **production mechanism whose behavior its assertions
discriminate** — not the entrypoint it calls through.

- What behavior does the assertion distinguish?
- Which production mechanism could change, harness unchanged, to flip it?
- `run_example_*` / `compile_expr*` are universal gateways and supply **no**
  subject evidence in either direction.
- A lexical reference to a private name is corroborating evidence, never a
  prerequisite.

The slice-5 partition was **first** computed lexically ("does the test name a
moved private?"). That predicate is complete only over lexical references, and
the Architect ruled it cannot serve as the omnibus partition. This ledger is the
semantic re-run over all **75** residual tests.

## Headline

**46 of the 75 residual tests are lowering-subject.** The lexical predicate
found 35 movers; the semantic re-run found 46 more, and slice 5 moves all of
them (@architect `evt_4p2c7tc37j84a` — no 5b; §10.5 makes a slice one
production module **plus its tests**).

> ⛔ **This headline previously read "47" while the rows below summed to 46.**
> 47 was the *pre-correction* figure: when the `px8i_local_helpers_*` row was
> reclassified `effects → artifact`, the table and per-bucket counts were
> updated and **the headline was not**. Caught by @architect. Every count in
> this file is now recomputed from the rows programmatically rather than
> carried by hand — *a total is an assertion; the rows are the measurement*
> (§7's own heuristic, which this document was written to discharge).

| destination | count | disposition |
|---|---:|---|
| `constructors` | 30 | lowering-subject — moved in slice 5 |
| `effects` | 10 | lowering-subject — moved in slice 5 |
| `control` | 6 | lowering-subject — moved in slice 5 |
| `values` | 0 | zero *additional* residual rows; its 12 ruled tests already landed |
| `artifact/api` | 27 | stays for slice 7 |
| `artifact` | 2 | stays for slice 6 |
| **UNCERTAIN** | **0** | — |
| **total** | **75** | |

**Reconciliation, verified post-move:** `75 = 46 + 29` · `110 = 81 + 29` ·
crate-wide `#[test]` `145 → 145` · subject-module gain `81` = omnibus loss `81`.

## ⛔ One correction applied against the generated ledger

`px8i_local_helpers_reject_invalid_zero_stale_and_wrong_arena_slots` (`:5108`)
was generated as `effects` on the strength of its arena-slot bookkeeping. **That
contradicts a landed ruling** — `evt_3xvn8g7n5rv7m` names it explicitly, with
`px8i_jit_and_object_construct_identical_local_helper_clif`, as directly
discriminating JIT/object helper construction and **artifact-subject for
slice 6**. Reclassified to `artifact`; the counts above reflect the correction
(`effects` 13 → 12, `artifact` 1 → 2).

Worth recording *why* it slipped: the classifier reasoned from the assertions
alone, which is the ruled method, and the assertions genuinely do exercise arena
slots. The ruling turns on **which mechanism owns the helper**, which the
assertions under-determine. A semantic classifier is not self-sufficient against
an explicit prior ruling — the ruled rows must be applied as fixed points, not
re-derived.

## Two boundary cases flagged, not silently resolved

1. **`px8i_jit_and_object_construct_identical_local_helper_clif` (`artifact`)
   and `px8i_local_helpers_reject_invalid_...` (`artifact`)** both exercise
   `native_int_clif`. With the correction above they now land together, which is
   the outcome the ruling implies.
2. **`recursive_declaration_shape_change_hits_typed_boundary` (`:3810`) —
   RESOLVED, stays `control`** (@architect `evt_4p2c7tc37j84a`). The invariant
   is stability of recursive-call argument representation across the active
   recursive declaration, enforced by the core recursive-declaration call sites
   consuming `same_recursive_argument_shapes`. The `Int` nested in
   `Option::Some` is only the **witness** that makes the recursive shape change;
   it does not make the subject value lowering.

### Three destination corrections (@architect `evt_4p2c7tc37j84a`)

Each had **the observer mistaken for the owner** — the same error as calling a
public entrypoint the subject:

| row | was | now | why |
|---|---|---|---|
| `nested_computational_payload_kind_rejects_specifically` | values | constructors | the Int guard observes; the nested producer/aggregate path owns |
| `constructor_field_host_result_stays_on_ordinary_dynamic_match` | effects | constructors | discriminates the constructor-field bridge excluding an effect-produced field |
| `pattern_default_trap_is_observation_not_backend_error` | effects | constructors | ordinary constructor-match default selection; no host/IO/bounded-Nat mechanism |

## Rows — lowering-subject (move)

| test | line | asserted behavior | owning mechanism | dest |
|---|---:|---|---|---|
| dynamic_host_result_producer_wrong_arity_rejects_specifically | 2694 | Ok case with 0 binders rejects | `dynamic_host_result_producer_case` binder-count check | constructors |
| dynamic_host_result_producer_result_kind_mismatch_rejects_specifically | 2710 | scalar and Exit trees must not merge | `record_merge_kind` disagreement check | constructors |
| dynamic_host_result_producer_well_formed_control_emits | 2726 | well-formed Ok/Err arms lower and merge | `dynamic_host_result_producer_case` + merge path | constructors |
| nested_computational_producer_well_formed_control_emits | 2735 | inner Bool producer composes through outer eliminator | `lower_computational_producer_expr` | constructors |
| nested_computational_outer_arity_rejects_specifically | 2744 | 0-binder case rejects 1-arg value | producer arity check | constructors |
| nested_computational_malformed_recursive_position_rejects_specifically | 2760 | out-of-range recursive position rejects | recursive_positions bounds check | constructors |
| nested_computational_final_merge_kind_rejects_specifically | 2776 | scalar and ExitCode arms must not merge | `record_merge_kind` | constructors |
| nested_computational_payload_kind_rejects_specifically | 2792 | non-Int payload into sub_int rejects | `lower_int_binop` Int-only guard | constructors |
| heterogeneous_eliminator_well_formed_control_emits | 2808 | Bool producer composes through nested ordinary frames | `requires_heterogeneous_deforestation` gate | constructors |
| constructor_field_selected_case_composes_before_field_lowering | 2826 | selected trailing field stays structural | constructor-field bridge | constructors |
| constructor_field_composes_through_computational_consumer | 2835 | field composes via computational consumer | constructor-field bridge | constructors |
| constructor_field_recursive_ih_offset_selects_argument_binder | 2905 | IH prefix does not shift selected field | binder-offset computation | constructors |
| constructor_field_middle_binder_preserves_trailing_environment_order | 2944 | run yields 34, env order preserved | bridge env/sibling ordering | constructors |
| constructor_field_binder_shift_mutation_recovers_exact_refusal | 3009 | wrong sibling refuses "not a constructor value" | `lower_expr` Match constructor-scrutinee check | constructors |
| constructor_field_bridge_removal_recovers_exact_refusal | 3025 | eager Let materialization reproduces refusal | same Match-arm check | constructors |
| constructor_field_outer_arity_rejects_before_field_lowering | 3058 | outer field-count mismatch rejects early | dispatch `argument_binders != args.len()` | constructors |
| constructor_field_missing_case_owns_default_before_fields | 3074 | unmatched constructor takes frame default | dispatch `None => Lowered::Trap(default)` | constructors |
| constructor_field_aggregate_unconsumed_sibling_stays_ordinary | 3101 | non-scrutinizing sibling stays ordinary | `immediate_binder_eliminator` | constructors |
| constructor_field_host_result_stays_on_ordinary_dynamic_match | 3141 | effect-produced field not bridged into splice | deforestation excludes `RuntimeExpr::Effect` | constructors |
| dynamic_constructor_dispatches_ordinary_continuation_with_mixed_arities | 3365 | mixed nullary/unary alternatives lower | `select_dynamic_constructor_case` arity check | constructors |
| dynamic_constructor_dispatches_producer_continuation_with_all_frames | 3374 | dispatch preserves active computational frame | `lower_dynamic_constructor_match` frame threading | constructors |
| dynamic_constructor_ordinary_continuation_preserves_bool_kind | 3383 | dynamic Bool usable by enclosing consumer | scalar/Bool kind preserved across arm merge | constructors |
| dynamic_constructor_binder_arity_rejects_exactly | 3392 | arity error keyed on identity not slot | identity-based case lookup | constructors |
| direct_host_result_closure_match_keeps_established_dynamic_lane | 3411 | Effect-produced HostResult stays ordinary | `select_ordinary_case` dispatch | effects |
| call_returned_host_result_keeps_established_dynamic_lane | 3420 | call-returned HostResult resolves ordinary | same dispatch via Call wrapper | effects |
| match_selected_call_returned_host_result_keeps_established_dynamic_lane | 3438 | survives intervening static Match arm | same dispatch | effects |
| recursive_computational_host_result_keeps_established_dynamic_lane | 3477 | recursive wrapper lands ordinary | same dispatch via recursive traversal | effects |
| recursive_computational_aggregate_traverses_ordinary_frame | 3486 | recursive leaf traverses active ordinary frame | recursive-position/env threading | constructors |
| heterogeneous_bridge_removal_recovers_exact_ordinary_match_refusal | 3503 | removing splice reproduces refusal | deferred-case splice vs eager Let | constructors |
| heterogeneous_frame_environment_and_binder_order_are_preserved | 3539 | end-to-end 41-7=34 through splice | `DeferredConstructorCaseEnvironment` threading | constructors |
| heterogeneous_final_merge_kind_rejects_specifically | 3597 | dynamic arms disagreeing on kind reject | `record_scalar_merge_kind` | constructors |
| heterogeneous_ordinary_arity_rejects_specifically | 3695 | binder/arity mismatch rejects via bridge | ordinary frame binder check | constructors |
| heterogeneous_nested_payload_kind_rejects_specifically | 3720 | sub_int rejects non-Int nested payload | PrimitiveCall Int-only operand check | constructors |
| recursive_declaration_shape_change_hits_typed_boundary | 3810 | recursive self-call with changed native shape rejects | native-arg representation consistency | control ⚠ |
| checked_join_marker_without_exact_plan_site_rejects_before_emission | 3871 | live join marker without exact site rejects | `CheckedJoinSite` marker validation | control |
| process_lowering_without_checked_root_authority_rejects_before_cfg | 3903 | process compile without root authority errors | distinguished-root authority check | control |
| checked_marker_census_rejects_duplicate_call_and_slot_occurrences_before_cfg | 4072 | duplicate occurrences reject, distinct reasons | checked-marker census ledger | control |
| valid_root_plus_missing_marked_scalar_cut_rejects_before_emission | 4112 | unconsumed marked scalar-cut site rejects | `NativeJoinPlanV1` consumption validation | control |
| self_consistent_appended_orphan_join_site_rejects_before_emission | 4151 | appended non-root site rejects as orphan | orphan/unconsumed-site validation | control |
| px8n_fs_write_at_arm_rejects_over_bound_reply_before_observation | 2103 | over-bound write rejects before Nat observable | `mint_validated_progress_nat` bounds | effects |
| px8n_fs_read_at_arm_distinguishes_eof_and_short_read_some | 2114 | zero-length ReadEof vs nonzero ReadSome | FsReadAt reply-decode discriminator | effects |
| px8n_fs_read_at_arm_rejects_over_bound_span_before_observation | 2130 | over-bound span rejects before Nat observable | `mint_validated_progress_nat` bounds | effects |
| px8i_host_narrowing_rejects_negative_and_over_u64_before_dispatch | 2141 | out-of-range host int rejected pre-dispatch | `narrow_native_int_u64` validity | effects |
| px8i_positioned_start_and_metadata_promote_u64_above_i64_max | 2154 | narrowed start kept; metadata promotes to Big | `narrow_native_int_u64` + `lower_unsigned_u64_int` | effects |
| unsupported_effect_is_distinct_from_backend_failure | 5221 | Console Effect errors Unsupported("Effect") | expr-lowering `RuntimeExpr::Effect` arm | effects |
| pattern_default_trap_is_observation_not_backend_error | 5250 | default-arm trap surfaces as observation | Match default-arm trap codegen | constructors |

## Rows — stays for slices 6/7 (29)

`artifact` (2): `px8i_jit_and_object_construct_identical_local_helper_clif`
(`:5080`), `px8i_local_helpers_reject_invalid_zero_stale_and_wrong_arena_slots`
(`:5108`) — both `native_int_clif`.

`artifact/api` (27): `program_runner_preflights_metadata_before_backend_lowering`,
the four `nc22_*` differential/preflight rows, the sixteen `nc8_certificate_*`
recompute rows, and the six `*_metadata_rejects_before_backend_lowering`
pre-lowering-gate rows. Every one asserts certificate / preflight / differential
/ outward-orchestration behavior, which §10.2 routes to `artifact/api/tests.rs`.
