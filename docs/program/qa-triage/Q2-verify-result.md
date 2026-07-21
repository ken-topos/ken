# Q2b triage result — Team Verify

72 tests / 25 files, split leader/implementer/QA (24 each, by whole file).
List only — no test file was changed in this pass.

## Combined counts (all three shares)

- durable-invariant: 68
- compat-vector: 3
- transition-sentinel: 1
- UNCLASSIFIABLE: 0

## Leader share (24 tests) — DONE

| test | class | confidence | note |
|---|---|---|---|
| `ken-cli/tests/deterministic_app.rs:106 checked_application_exposes_the_real_dispatch_observation` | durable-invariant | high | Trace length (2) and per-entry operation/capability/request are derived from one fixed injected fixture and checked structurally, not a frozen census. |
| `ken-cli/tests/i8_clock_effect.rs:78 fixed_clock_repeats_one_value_and_traces_each_read` | durable-invariant | high | Tests `CaptureHost`'s fixed-clock determinism (same value, both reads traced); name describes the property under test, not a frozen count. |
| `ken-cli/tests/i8_clock_effect.rs:115 clock_package_is_structural_zero_trust_and_declares_no_ordering_law` | durable-invariant | high | Checks `trusted_base()` unchanged plus literal absence of Axiom/postulate/primitive/ordering tokens in the literate package — a real structural zero-delta check. |
| `ken-cli/tests/px8x_single_schema_observation.rs:50 linked_route_exposes_real_ordered_bindings_and_filters_reserved_input` | durable-invariant | high | Effect-trace length/sequence/operation are derived from one fixed linked resource program and checked structurally, not a census. |
| `ken-elaborator/src/erasure.rs:6002 erased_constructor_parameter_and_live_ih_argument_emit_one_runtime_marker` | durable-invariant | high | Arity/seed counts are fixture-derived (one constructed ctor application, one IH call seed); asserts the lowered `RuntimeExpr` shape structurally, not just success. |
| `ken-elaborator/tests/ax2_axiom_named_postulates.rs:46 repeated_expression_axioms_share_the_owner_label_but_not_identity` | durable-invariant | high | Two `Axiom` occurrences in a fixed two-line source; asserts shared name but distinct `GlobalId`s — a real provenance property, count is fixture-derived. |
| `ken-elaborator/tests/ax2_axiom_named_postulates.rs:86 standalone_api_requires_and_preserves_its_caller_owner` | durable-invariant | high | Same shared-label/distinct-identity property, exercised through the standalone elaborate_expr API; count is fixture-derived. |
| `ken-elaborator/tests/cc1_nonempty_validation_acceptance.rs:107 cc1_checked_code_has_zero_axiom_and_zero_trusted_base_delta` | durable-invariant | high | Checks literal absence of "Axiom" across the whole extracted source and every example/reject fence, **and** a real `env.trusted_base()` before/after `BTreeSet` `assert_eq!` after elaborating both files — a genuine API-level zero-delta check, not text-grep alone (correction: an earlier note here wrongly claimed the test stopped at the text scan; it does not — the `trusted_base()` diff is the back half of the test body). |
| `ken-elaborator/tests/cc6a_process_arguments_exit_acceptance.rs:245 structural_slice_location_keeps_nonzero_argument_and_range` | durable-invariant | high | Expected `(2, 3, 5)` triple is computed from fixed const Nat/Bytes literals in the test source and checked structurally via `ctor_args`, not a frozen census. |
| `ken-elaborator/tests/cc6a_process_arguments_exit_acceptance.rs:351 cc6a_has_zero_trust_delta_and_no_new_carrier_or_string_hop` | durable-invariant | high | Same text-grep zero-Axiom/primitive-absence pattern as `cc1:107`, plus its own independent `env.trusted_base()` before/after `assert_eq!` after elaborating both `.ken.md` files — a real API-level zero-delta check (correction: the prior note here inherited `cc1:107`'s wrong "text-grep only" claim by reference without independently reading this test's back half; both tests actually call `trusted_base()`). |
| `ken-elaborator/tests/ds4_list_combinators_acceptance.rs:26 all_five_combinators_and_their_laws_are_real_globals` | durable-invariant | high | Presence check over a named list of 12 symbols (5 combinators + their laws); stays green under addition, goes red only if a named symbol is removed/renamed — intended-extension-safe. |
| `ken-elaborator/tests/ds4_list_combinators_acceptance.rs:107 ac8_off_by_one_range_length_rejected` | durable-invariant | high | AC8 discriminator: an off-by-one witness for `range_length` must be rejected; asserts TypeMismatch/KernelRejected, a real negative-arm test. |
| `ken-elaborator/tests/es4_classes_acceptance.rs:173 classes_are_transparent_structure_records_zero_delta` | durable-invariant | high | Checks `Eq`/`DecEq`/`Ord` are real Transparent record types and never enter `trusted_base()` — a real structural + soundness-adjacent invariant. |
| `ken-elaborator/tests/es4_classes_acceptance.rs:288 eq_bool_is_a_complete_zero_delta_instance` | durable-invariant | high | Each law field checked non-Opaque (real proof) and `trusted_base_delta` empty modulo the structural `record_nil_val` sentinel. |
| `ken-elaborator/tests/es4_classes_acceptance.rs:359 int_ord_instance_is_audited_delta_not_zero_delta` | durable-invariant | high | The honest-postulate counterpart to `:288` — verifies Int's law fields ARE opaque (audited-delta, not silently claimed zero-delta). |
| `ken-elaborator/tests/es4_classes_acceptance.rs:575 char_ord_laws_reject_missing_law_field` | durable-invariant | high | Real discriminator — an `Ord Char` instance omitting `total` must be rejected. Note: the negative arm relies on `r.is_err()` alone (R2 pattern) rather than a named error variant; the promise is durable, but the oracle could be tightened to a specific `ElabError`/`KernelError` variant in a later pass. |
| `ken-elaborator/tests/fr1_zero_ctor_data_acceptance.rs:15 explicit_family_zero_ctor_data_elaborates_and_eliminates` | durable-invariant | high | Confirms a surface-authored zero-constructor explicit-family `data` elaborates and its empty-match eliminator works; a real capability check. |
| `ken-elaborator/tests/fr1_zero_ctor_data_acceptance.rs:32 legacy_zero_ctor_data_elaborates_and_eliminates` | durable-invariant | high | Same property for the legacy `data D =` spelling. |
| `ken-elaborator/tests/fr1_zero_ctor_data_acceptance.rs:47 both_spellings_parse_to_zero_constructor_decls` | durable-invariant | high | `decls.len() == 1` is derived from parsing exactly one literal declaration per spelling; ctor-emptiness checked structurally for both surface forms. |
| `ken-elaborator/tests/ktr2_data_lowering_diagnostics.rs:33 data_type_references_follow_declaration_order` | durable-invariant | high | `ids.len() == 2` is derived from a fixed two-declaration fixture; the forward-reference rejection variant is checked too. |
| `ken-elaborator/tests/ktr2_data_lowering_diagnostics.rs:50 ordinary_legacy_type_zero_data_still_elaborates` | durable-invariant | high | Trivial sanity (`id.0 > 0`) that a Type-0 constructor argument still elaborates; not count-sensitive despite the flag. |
| `ken-elaborator/tests/ktr2_data_lowering_diagnostics.rs:90 explicit_type_one_family_accepts_type_payload` | durable-invariant | high | Confirms the explicit Type-1 escape hatch remains valid; name states the capability ("Type one family"), not a frozen count. |
| `ken-elaborator/tests/l3_strings_roundtrip_acceptance.rs:320 derived_list_char_surface_out_of_scope` | transition-sentinel | high | Explicitly labelled in-file: pins that the derived `List Char` surface (`concat`/`slice`/`char_at`/etc.) is NOT YET callable ahead of the Team Language slice-2 landing, so that landing shows as a visible diff rather than a silent no-op. Named boundary + retiring event present in the comment — a properly-labelled sentinel, not a latent gap. |
| `ken-elaborator/tests/l7_acceptance.rs:464 verified_component_foreign_call_and_roundtrip_proof` | durable-invariant | high | `obligations.len() == 1` is derived from proving exactly one law in the fixture; demonstrates the G6 honest P/Q trust split structurally via `trusted_base_delta` membership. |

## Implementer share (24 tests) — DONE

Scope: `explicit_data_elaboration.rs` (7), `l3a_recursive_view_smoke.rs` (3),
`lc_acceptance.rs` (4), `n3_import_exclusion.rs` (1), `px8f_buffer_io_surface.rs` (1),
`sec2_acceptance.rs` (6), `structural_deceq_acceptance.rs` (1), `ken-verify/src/canonical.rs` (1).

| test | class | confidence | note |
|---|---|---|---|
| `ken-elaborator/tests/explicit_data_elaboration.rs::non_indexed_explicit_family_elaborates_and_constructor_is_usable` | durable-invariant | high | Params/ctor/arg counts derive from the fixed local `data Box` fixture in the test itself — checks elaboration preserves declared arity, not an external growing census. |
| `ken-elaborator/tests/explicit_data_elaboration.rs::legacy_named_constructor_field_sugar_lowers_to_positional_constructor` | durable-invariant | high | Same pattern — counts derive from the fixed `Point` fixture; verifies field-sugar lowers to the same arity. |
| `ken-elaborator/tests/explicit_data_elaboration.rs::explicit_where_named_constructor_field_sugar_lowers_to_positional_constructor` | durable-invariant | high | Same — fixed `PairBox` fixture, arity preserved through the `where`-style sugar. |
| `ken-elaborator/tests/explicit_data_elaboration.rs::indexed_vector_family_records_indices_and_constructor_targets` | durable-invariant | high | Counts (1 param, 1 index, 2 ctors) derive from the fixed `Vector` fixture; tests indexed-family bookkeeping matches declared shape. |
| `ken-elaborator/tests/explicit_data_elaboration.rs::proof_carrying_constructor_telescope_elaborates_with_prior_binders_in_scope` | durable-invariant | high | `args.len()==5` derives from the 5 explicit telescope params in the fixed `CheckedSource` fixture. |
| `ken-elaborator/tests/explicit_data_elaboration.rs::legacy_simple_data_still_elaborates` | durable-invariant | high | `constructors.len()==2` derives from the fixed 2-ctor `MaybeNumber` fixture. |
| `ken-elaborator/tests/explicit_data_elaboration.rs::indexed_impossible_constructor_may_be_omitted_from_non_empty_vector_match` | durable-invariant | high | `indices.len()==1`/`methods.len()==2` are structural Elim invariants (one index per declared index, one method per ctor of the fixed 2-ctor Vector) — not a frozen external count. |
| `ken-elaborator/tests/l3a_recursive_view_smoke.rs::poly_id_elaborates` | durable-invariant | high | Regression guard for a specific historical bug (IH-lambda-domain weakening); `is_ok()` is the correct oracle — no informative Ok payload beyond "typechecks". |
| `ken-elaborator/tests/l3a_recursive_view_smoke.rs::poly_id_list_elaborates` | durable-invariant | high | Same regression-guard reasoning, parameterized-list case. |
| `ken-elaborator/tests/l3a_recursive_view_smoke.rs::poly_match_return_elaborates` | durable-invariant | high | Same, guards the parameterized-match-return variant of the IH-domain bug specifically. |
| `ken-elaborator/tests/lc_acceptance.rs::ac2_orphan_check_accept_and_reject` | durable-invariant | high | Reject arm already matches `ElabError::OrphanInstance{..}` by name; accept arm's `is_ok()` has no informative payload to check further. |
| `ken-elaborator/tests/lc_acceptance.rs::ac4_property_vs_structure_sort_discriminant` | durable-invariant | high | `ClassKind::Structure`/`Property` and `ElabError::OverlappingInstances{..}` are named-variant checks. |
| `ken-elaborator/tests/lc_acceptance.rs::ac6_sct_wellfounded_accepted_cyclic_rejected` | durable-invariant | high | Real discriminator pair — accept arm `is_ok()`, reject arm names `ElabError::NonTerminatingInstances{..}`. |
| `ken-elaborator/tests/lc_acceptance.rs::ac7_derive_kernel_rechecks_candidate` | durable-invariant | high | Same pair shape — reject arm names `ElabError::KernelRejected{..}`. |
| `ken-elaborator/tests/n3_import_exclusion.rs::parser_distinguishes_item_rename_from_module_alias_and_rejects_hiding` | durable-invariant | high | `items.len()==1` derives from the literal `import M (foo as bar)` source parsed in-test — parser arity fidelity to source, not a census. |
| `ken-elaborator/tests/px8f_buffer_io_surface.rs::buffer_span_producer_closure_resolves_public_constructors` | durable-invariant | high | `constructors.len()==1` reflects BufferSpan's actual sealed single-constructor design (SPAN-SEAL) — a real structural fact, not an arbitrary frozen count. |
| `ken-elaborator/tests/sec2_acceptance.rs::world_action_without_capability_rejected` | durable-invariant | high | Reject arm names `EffectError::MissingCapability{effect}` with field check; flip arm `is_ok()` has no further payload. |
| `ken-elaborator/tests/sec2_acceptance.rs::no_row_view_is_inert_rejected` | durable-invariant | high | Reject arm names `EffectError::EffectEscapes{witnesses}` with content check. |
| `ken-elaborator/tests/sec2_acceptance.rs::uses_unpassed_capability_rejected` | durable-invariant | high | Same `MissingCapability{effect}` pattern as the world-action case. |
| `ken-elaborator/tests/sec2_acceptance.rs::attenuated_cap_at_weak_sink_accepts` | durable-invariant | high | Accept half of the documented ⊑-orientation discriminator pair (paired with strong-sink-rejects); `is_ok()` has no further payload on the accept side. |
| `ken-elaborator/tests/sec2_acceptance.rs::attenuated_cap_at_strong_sink_rejects` | durable-invariant | high | Reject half of the pair — already exhaustively matches `CapError::AuthorityInsufficient{required,available}` with exact field asserts. |
| `ken-elaborator/tests/sec2_acceptance.rs::capabilities_cross_case_sweep` | durable-invariant | high | Cross-case regression sweep re-affirming A/C/F gates together; the two novel composed cases name `AuthAndFlowResult::FlowRejected`/`CapRejected` explicitly. |
| `ken-elaborator/tests/structural_deceq_acceptance.rs::structural_instances_are_checked_transparent_and_zero_delta` | durable-invariant | high | "zero_delta" is a real soundness contract (structural DecEq liftings must add no new trust-base postulates beyond record_nil), not a placeholder count a legitimate extension would organically grow. |
| `ken-verify/src/canonical.rs::every_canonical_mutation_bites_while_same_runner_proxy_stays_green` | durable-invariant | high | Differential sweep over an explicit `CanonicalMutation` list: comparator must detect every listed mutation while a decoy runner-only proxy stays green; `is_err()`/`agrees()` suffice since detection-completeness (not specific error content) is the property under test. Note for the list's owner: it's a hand-maintained `Vec`, not an exhaustive match over `CanonicalMutation` — a future variant needs manual addition here (maintenance note, not a defect in the test as written). |

### Implementer counts
- durable-invariant: 24
- compat-vector: 0
- transition-sentinel: 0
- UNCLASSIFIABLE: 0

## QA share (24 tests) — DONE

Scope: `ken-host/src/abi_v1.rs` (13), `ken-interp/tests/px8x_single_schema_observation.rs` (1),
`ken-kernel/tests/k1p5_wstyle.rs` (8), `ken-kernel/tests/obs_eq_termination_congruence.rs` (2).

| test | class | confidence | note |
|---|---|---|---|
| `crates/ken-host/src/abi_v1.rs::whole_file_create_or_keep_preserves_a_concurrently_appearing_file` | durable-invariant | high | Asserts CreateOrKeep policy semantics under a real racing-appearance fixture — always must preserve the winner regardless of extension. |
| `crates/ken-host/src/abi_v1.rs::caller_control_trap_keeps_terminal_primary_and_appends_cleanup_failure` | durable-invariant | high | Terminal-primacy + structural `ResourceErrorV1::ReleaseFailed` variant match with bound identity; the `effect_trace.len()==1` is a fixed count of this fixture's own actions, not a growing census. |
| `crates/ken-host/src/abi_v1.rs::generated_effect_layout_matches_every_live_wire_record` | compat-vector | high | Sizes/aligns/offsets ARE the ABI contract (byte-identity), plus tag/error/limit ID pins. `HOST_EFFECT_ABI_V1_CATALOG.len()==22` and the native-availability relation both change only on a deliberate op-set contract change — a legitimate compat-vector bump, not decay. |
| `crates/ken-host/src/abi_v1.rs::resource_errors_use_the_distinct_fully_initialized_reply_projection` | compat-vector | high | Pins the exact wire projection (tag/detail/resource_error fields) each `ResourceErrorV1` variant encodes to — normative wire format. |
| `crates/ken-host/src/abi_v1.rs::posture_failure_prevents_context_publication` | durable-invariant | low | Real invariant (posture `Err` ⇒ context publication `is_err()`), but only checks `is_err()` — doesn't name the resulting error variant, so it can't discriminate a wrong-but-still-`Err` outcome. Owner should confirm whether a more specific variant assertion is warranted. |
| `crates/ken-host/src/abi_v1.rs::scripted_effective_uid_discriminates_the_native_startup_posture` | durable-invariant | high | Three-way discriminator (uid=0/no-allow rejected+named variant `RootExecutionDenied`; uid=0/allow accepted; uid≠0/no-allow accepted) — a genuine policy invariant, not a snapshot. |
| `crates/ken-host/src/abi_v1.rs::root_denial_writer_needs_no_live_process_context` | durable-invariant | high | Structural decode of the pre-context terminal observation: plan_hash, terminal_value, named `TerminalErrorV1::RootExecutionDenied`, empty effect_trace — all invariant regardless of extension. |
| `crates/ken-host/src/abi_v1.rs::scripted_home_failures_use_real_init_and_pre_context_terminal_writer` | durable-invariant | high | Parameterized over all 7 `HomeRootResolutionFailureV1` variants — exhaustive-by-construction coverage of the failure space, each checked structurally end-to-end. |
| `crates/ken-host/src/abi_v1.rs::scripted_home_success_mints_only_after_one_real_resolution` | durable-invariant | high | `calls.get()==1` asserts the real lookup fires exactly once (no redundant/duplicate resolution) — a genuine no-double-lookup invariant, not a census; followed by reading real resolved data through the minted capability. |
| `crates/ken-host/src/abi_v1.rs::full_native_init_uses_only_the_scripted_observer_seam` | durable-invariant | high | Denied (uid=0, no allow) vs allowed (uid=0, allow) discriminating pair; denied path structurally checked (null context, named `RootExecutionDenied`, empty trace). |
| `crates/ken-host/src/abi_v1.rs::wrong_generation_is_capability_denied_before_filesystem_access` | durable-invariant | high | Malformed-capability dispatch structurally asserted to reach `CapabilityDeniedV1::MalformedCapability` and never touch the filesystem — fail-closed-before-access invariant. |
| `crates/ken-host/src/abi_v1.rs::change_mode_rejects_file_type_bits_before_dispatch` | durable-invariant | high | Invalid mode bits rejected pre-dispatch: named `InvalidInput` error, empty effect_trace, file content untouched — fail-closed invariant. |
| `crates/ken-host/src/abi_v1.rs::native_resource_open_metadata_release_and_stale_use_share_one_real_context` | durable-invariant | high | `effect_trace.len()==4` is the fixed count of this test's own 4 dispatch calls (open+2×metadata+release), not an evolving census; asserts stale-use-after-release structurally yields `ResourceErrorV1::Closed`. |
| `crates/ken-interp/tests/px8x_single_schema_observation.rs::real_buffer_bracket_exposes_ordered_target_bindings` | compat-vector | high | Fixed hand-written `.ken` fixture program — its exact ordered effect trace (`BufferAllocate` then `ResourceRelease`, same binding identity across allocate/release) is a known-answer vector for that specific program, not a growing census. |
| `crates/ken-kernel/tests/k1p5_wstyle.rs::ac1_tree_w_style_admitted` | durable-invariant | high | Structural-admission invariant: this W-style shape must always be admitted under K1.5 positivity, regardless of future extension. |
| `crates/ken-kernel/tests/k1p5_wstyle.rs::ac1_w_type_admitted` | durable-invariant | high | Same admission invariant for the canonical `W A B` shape at a level variable. |
| `crates/ken-kernel/tests/k1p5_wstyle.rs::ac1_negative_bad_rejected` | durable-invariant | high | Rejection invariant with named variant (`KernelError::PositivityViolation`) — a genuine soundness boundary, not a snapshot. |
| `crates/ken-kernel/tests/k1p5_wstyle.rs::ac1_branching_domain_not_d_free_rejected` | durable-invariant | high | Same class: named-variant rejection of a D-at-negative-polarity branching domain. |
| `crates/ken-kernel/tests/k1p5_wstyle.rs::ac2_two_motive_levels_accepted` | durable-invariant | high | Structural equality on normalized reducts at two genuinely distinct motive levels (Type 0 vs large elimination into Type 0) — non-degenerate, checks actual computed value not just success. |
| `crates/ken-kernel/tests/k1p5_wstyle.rs::ac4_k1_suite_regression` | durable-invariant | high | Regression invariant (Nat elim still reduces correctly; negative occurrence still rejected) — holds under any future K1.5 change, not a one-time snapshot. |
| `crates/ken-kernel/tests/k1p5_wstyle.rs::ac2_indexed_wstyle_method_type_agreement` | durable-invariant | high | `infer(...).is_ok()` on a calibrated adversarial case (dependent motive + postulated witness) built specifically to fail under the de Bruijn cutoff bug this test guards — a genuine mechanism-correctness invariant. |
| `crates/ken-kernel/tests/k1p5_wstyle.rs::qa_wstyle_double_pi_branching_telescope` | durable-invariant | high | Structural shape assertions (`peel_pi` arity, `iota_reduct` head/args) on a fixed 2-Π hand-built fixture — fixed by the fixture's own construction, not a project-wide count that would need bumping on unrelated growth. |
| `crates/ken-kernel/tests/obs_eq_termination_congruence.rs::allkeys_two_predicate_spellings_converts` | durable-invariant | high | Positive conversion invariant for the `conv_struct` congruence-first fast path — genuinely-true equality must always be accepted. |
| `crates/ken-kernel/tests/obs_eq_termination_congruence.rs::allkeys_distinct_predicate_stays_rejected` | durable-invariant | high | Discriminating negative control: a genuinely non-convertible pair must stay rejected — proves the fast path's fall-through preserves soundness. |

### QA counts
- durable-invariant: 21
- compat-vector: 3
- transition-sentinel: 0
- UNCLASSIFIABLE: 0
