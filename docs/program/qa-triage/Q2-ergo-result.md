# Q2b triage result — Team Ergo

71 tests / 25 files. Read-only triage pass (Q2b) — no test edits made, no
build run, no PR opened. Assembled from three ring shares (ergo-leader,
ergo-implementer, ergo-qa); one row per test.

**Counts:** 65 durable-invariant, 5 compat-vector, 1 transition-sentinel,
0 UNCLASSIFIABLE.

## ergo-leader share (26 tests)

| test | class | confidence | note |
|---|---|---|---|
| ken-cli/tests/console_exec.rs::closed_stdout_is_an_io_failure_not_sigpipe_termination | durable-invariant | high | Flagged `include_str!` line is a supplementary regression guard; the load-bearing assertions spawn the real `ken` binary and assert exit 17 (not SIGPIPE) on real broken-pipe I/O — reaches the production mechanism. |
| ken-cli/tests/px7l_checked_host_recursive_bind.rs::delayed_capturing_generic_bind_agrees_across_real_executors | compat-vector | high | `effect_trace.len()==3` is subsumed by the following exact-sequence `assert_eq!` against real native+interpreted traces; the trace shape is the contract for this checked bind. |
| ken-cli/tests/px8ta_oriented_subcontinuation.rs::public_one_level_bracket_finishes_and_releases | durable-invariant | high | Depth-1 bracket-release via the real native/interpreted mechanism; "one_level" names the boundary case, not a frozen census. |
| ken-cli/tests/px8ta_oriented_subcontinuation.rs::public_two_three_level_brackets_finish_and_release_lifo | durable-invariant | high | Loops depths 2..=3, LIFO release generalizes beyond depth 1; name states the tested range. |
| ken-cli/tests/px8ta_oriented_subcontinuation.rs::px8ds_real_same_depth_path_rejects_flat_order_and_runs_exact_edges | durable-invariant | low | Real producer + real rejection path, but `.matches("depth=1").count()==2` couples to error-message wording rather than a structured field; semantic property is real, mechanism is fragile to message rewording. |
| ken-elaborator/tests/b1_acceptance.rs::proved_postcondition_projects_to_q | durable-invariant | high | Fixed-scenario structural count for a real V1→V3 discharge; half of the documented no-over-claim pair with open_hole_postcondition_rides_p_as_unknown. |
| ken-elaborator/tests/b1_acceptance.rs::open_hole_postcondition_rides_p_as_unknown | durable-invariant | high | Other half of the same no-over-claim pair. |
| ken-elaborator/tests/b1_acceptance.rs::removing_assume_shrinks_p_and_changes_hash | durable-invariant | high | Real discharge scenario, documented discriminator pair (EX-B1+EX-G1), not a frozen count. |
| ken-elaborator/tests/b1_acceptance.rs::alphabet_equals_perform_node_signatures | durable-invariant | high | Set-equality + closure check of Σ against the real checked perform-node projection — the guideline's preferred shape. |
| ken-elaborator/tests/b1_acceptance.rs::generators_carry_support_not_measure | durable-invariant | high | Fixed single-generator scenario verifying structural absence of a weight/measure field via the real serialized export. |
| ken-elaborator/tests/b1_acceptance.rs::delegated_obligation_never_promoted_to_proved | durable-invariant | high | Real emit_export; verifies the one-way T-only promotion gate (no path to Q) — an absence-gate soundness net. |
| ken-elaborator/tests/b1_exact_denotation_alphabet.rs::non_host_l5_perform_uses_its_typed_inductive_identity | durable-invariant | high | Minimal fixed program; real test is the Ping/Pong sibling-discriminator pair (distinct signatures, distinct hashes). |
| ken-elaborator/tests/b2_acceptance.rs::ordinary_inductive_admitted_by_k1 | compat-vector | high | `constructors.len()==9` pins the §3 LTL/μ constructor arity — grammar-arity contract, cited directly in qa.md's compat-vector examples. |
| ken-elaborator/tests/b2_acceptance.rs::no_modal_construct_in_kernel | durable-invariant | high | Source-text absence-of-construct check is the only expressible mechanism here; includes its own disconfirming/collision check (the "later" prose lexeme in obs.rs). |
| ken-elaborator/tests/b2_acceptance.rs::inert_to_conversion | durable-invariant | high | Same class of absence-of-construct check (conv.rs has no Temporal branch), same reasoning. |
| ken-elaborator/tests/b2_acceptance.rs::block_elaborates_delegated_and_visible | durable-invariant | high | Fixed one-clause-program structural count; substantive assertions are formula shape + delegated status through the real elaborate→export path. |
| ken-elaborator/tests/b2_acceptance.rs::value_projects_to_t_delegated_never_q | durable-invariant | high | Real elaborator + real B1 emitter backing a total/constant verdict-mapping soundness claim. |
| ken-elaborator/tests/b2_acceptance.rs::obligation_not_dischargeable_in_ken | durable-invariant | high | Composes three real checks (kernel source absence, real emit_export routing, temporal.rs function-absence) into one impossibility argument. |
| ken-elaborator/tests/b2_acceptance.rs::cross_case_verdict_mapping_is_constant | durable-invariant | high | Exhaustive sweep across the four temporal operators, each independently verified — the exhaustive-matches shape the guideline prefers. |
| ken-elaborator/tests/b2_acceptance.rs::temporal_symbols_consume_the_one_checked_b1_alphabet | durable-invariant | high | Real checked B1→B2 alphabet-membership accept path plus a sibling-symbol rejection path — a genuine discriminator pair. |
| ken-elaborator/tests/b2_acceptance.rs::temporal_spec_constructor_shape | compat-vector | high | Same §3 grammar-arity contract as ordinary_inductive_admitted_by_k1; docstring states it guards constructor-count drift. |
| ken-elaborator/tests/cat3_collections_package.rs::cat3_d1_structural_collections_package_elaborates_zero_delta | durable-invariant | high | "Zero delta" is the soundness property itself, not a fossil count; real elaboration of the actual catalog package over an exhaustive enumerated name list. |
| ken-elaborator/tests/cat3_collections_package.rs::cat3_d1_law_surfaces_are_proof_returning_not_prop_wrappers | compat-vector | high | Pins exact `Equal ...` lemma signatures in the real catalog source as the proof-returning contract; file docstring confirms it checks real package source, not a hand-copied snippet. |
| ken-elaborator/tests/cc2_text_codec_numeric_acceptance.rs::cc2_checked_code_has_zero_axiom_and_zero_trusted_base_delta | durable-invariant | high | Same "zero" soundness pattern as CAT-3; real elaboration of the actual .ken.md sources. |
| ken-elaborator/tests/cc2_text_codec_numeric_acceptance.rs::bijection_prerequisite_is_the_single_separately_homed_assumption | compat-vector | high | Pins `axiom ` line-count to exactly one as the documented single-assumption architecture decision for this prerequisite. |
| ken-elaborator/tests/cc4_diagnostic_core_acceptance.rs::checked_cc4_chain_has_zero_axiom_and_zero_trusted_base_delta | durable-invariant | high | Same zero-delta soundness pattern as CC2; real elaboration of the four real CC4 chain sources. |

## ergo-implementer share (22 tests)

| test | class | confidence | note |
|---|---|---|---|
| cc7_argparse_acceptance.rs::forge_parses_flags_raw_values_and_positionals_and_renders_derived_help | durable-invariant | high | count derives from the test's own fixed 5-arg input (→3 parsed units), not a growing census |
| cc7_argparse_acceptance.rs::two_independent_bad_arguments_accumulate_exact_nonzero_locations | durable-invariant | high | tests non-short-circuit accumulation; 2 = count of the test's own fixed bad inputs |
| cc7_argparse_acceptance.rs::adding_one_option_to_the_spec_changes_help_without_a_second_help_edit | durable-invariant | high | single-source-of-truth help-gen property; "one option" is the scenario setup, not a growing count |
| cc7_argparse_acceptance.rs::cc7_is_a_zero_trust_specialization_with_no_second_universe | durable-invariant | low | core claim (trusted_base before==after) is grounded on the real API; but bundles exact-substring pins over extracted literate source (forbidden/required token lists + one verbatim function-body string) that are brittle to cosmetic refactors preserving the same property — candidate to split out in a rework pass |
| coproduct_effect_signature_acceptance.rs::coproduct_is_the_live_two_param_effect_signature_type | durable-invariant | high | checks the fixed, already-landed Coproduct inductive's own arity/ctor shape + that `Sum` is vacated; "two"/"exactly two" are the type's own fixed shape, not derived |
| ds1_empty_dec_acceptance.rs::ac1_mechanism_probe_no_method_wrong_domain_rejected | durable-invariant | high | bare `is_err()` is correct here — the test's own purpose is "any kernel rejection proves the per-branch check fires," several legitimate error paths could catch the malformed method |
| ds1_empty_dec_acceptance.rs::ac3_trusted_base_delta_is_ordinary_inductive_admission_only | durable-invariant | high | grep-the-Rust-emission is the sanctioned technique for a trust-provenance claim (which admission fn was called) — invisible to any runtime check |
| ds5b_dependent_match_refinement_acceptance.rs::trusted_base_delta_is_empty_across_all_three_capabilities | durable-invariant | high | real `trusted_base()` before==after set-equality; "three" = this WP's fixed capability count, not a growing census |
| explicit_data_parser.rs::explicit_family_vec_preserves_constructor_signature_shape | durable-invariant | high | structural parse-shape counts intrinsic to the fixed literal source string |
| explicit_data_parser.rs::proof_carrying_constructor_signature_parses_as_telescope | durable-invariant | high | arg count(6) is the literal telescope arity in the fixed source |
| explicit_data_parser.rs::implicit_constructor_binder_is_preserved | durable-invariant | high | same pattern, fixed literal input |
| explicit_data_parser.rs::explicit_where_block_accepts_simple_default_result_constructors | durable-invariant | high | ctors.len()==2 matches the fixed 2-ctor literal source |
| explicit_data_parser.rs::legacy_data_accepts_named_constructor_field_sugar | durable-invariant | high | same pattern |
| explicit_data_parser.rs::explicit_where_simple_constructor_accepts_named_field_sugar | durable-invariant | high | same pattern |
| explicit_data_parser.rs::legacy_data_form_stays_simple_and_rejects_explicit_signatures | durable-invariant | high | same pattern, plus a negative/reject branch |
| kenfmt_b3_layout.rs::ac4_all_source_parentheses_and_precedence_are_preserved | durable-invariant | high | substantive checks are AST/token/trusted_base equality (meaning-preservation); `is_ok()` calls are incidental setup |
| kenfmt_b3_layout.rs::ac4_old_atom_boundary_parentheses_preserve_meaning | durable-invariant | high | same meaning-preservation pattern, plus idempotence |
| kenfmt_b3_layout.rs::ac6_reachable_fmt9_fences_remain_parse_preserved_after_horizontal_supersession | durable-invariant | high | reachability gate (`executed>0`) + AST-preservation/idempotence over conformance-oracle-driven fixtures; both flags are false positives — the `==1` is fixture-selection logic not an assertion, and the source-text parsing targets the conformance oracle markdown (fixture extraction), not implementation Rust source |
| kenfmt_b4_splicing.rs::ac1_extractor_exposes_all_four_roles_and_marker_spans_additively | durable-invariant | high | "four" = the extractor's own fixed canonical-role enum (Source/Ignore/Reject/Example); other counts derive from the fixed literal markdown fixture |
| kenfmt_b4_splicing.rs::ac2_parse_first_role_gate_has_both_fallback_and_hard_error_orientations | durable-invariant | high | tests formatter fallback/hard-error behavior across roles on fixed literal fixtures; R4 flag is a false positive — asserts formatter OUTPUT, not Rust implementation source |
| kenfmt_b4_splicing.rs::ac3_descending_splices_prevent_multi_fence_offset_drift | durable-invariant | high | `fences.len()==2` is intrinsic to the fixed two-fence literal input testing offset-drift |
| l6_acceptance.rs::decode_encode_roundtrip_provable | durable-invariant | high | obligation-shape + dischargeability property, well-documented in the test's own header; `obligations.len()==1` is intrinsic to a single `prove` declaration |

## ergo-qa share (23 tests)

| test | class | confidence | note |
|---|---|---|---|
| nc14_data_match_lowering.rs:138 `user_data_two_payload_binders_preserve_de_bruijn_order` | durable-invariant | high | name numeral names the ctor arity under test, not a census; property generalizes to any arity |
| v1_acceptance.rs:54 `requires_elaborates_to_pi_proof_arg` | durable-invariant | high | fixed 2-param fixture; Pi-chain len is this function's exact structural contract |
| v1_acceptance.rs:101 `ensures_emits_obligation_not_sigma` | durable-invariant | high | one ensures clause -> one obligation is the spec's own per-clause mapping; structural assert_eq on emitted term |
| v1_acceptance.rs:160 `non_omega_predicate_surface_error` | durable-invariant | high | rejection property durable; broad is_err() with no ElabError variant named — real but weak, Q3+ concern |
| v1_acceptance.rs:173 `result_in_ensures_resolves` | durable-invariant | high | fixed fixture, same per-clause mapping as sibling test |
| v1_acceptance.rs:186 `result_out_of_ensures_rejects` | durable-invariant | high | scope-rejection durable; same broad is_err() caveat |
| v1_acceptance.rs:208 `old_resolves_in_space_op_ensures` | durable-invariant | high | fixed fixture, one obligation for one ensures clause in a space proc |
| v1_acceptance.rs:226 `old_out_of_scope_rejects` | durable-invariant | high | soundness scope-rejection durable; broad is_err() caveat |
| v1_acceptance.rs:391 `obligation_hole_set_exposed_to_v2` | durable-invariant | high | two distinct ensures clauses -> two distinct holes; V1->V2 interface contract, not a growable count |
| v1_acceptance.rs:420 `prove_goal_obligation_and_postulate_binding` | durable-invariant | high | full discharge round-trip (open->cert->closed), not success-only |
| v1_acceptance.rs:453 `law_all_omega_fields_is_proposition` | durable-invariant | high | one law/two fields -> two obligations is the spec's per-field mapping |
| v1_acceptance.rs:515 `requires_on_first_of_two_params` | durable-invariant | high | named regression for a fixed de Bruijn shift bug (V1-fix); asserts the exact Lam chain structurally |
| i4_program_caps_step1.rs:5 `program_caps_is_authority_parametric_and_adds_zero_trust` | durable-invariant | high | fixed ProgramCaps shape + real before/after trusted_base() set check, not a count |
| i5_scoped_capability.rs:129 `traversal_denies_while_sibling_accepts_through_real_dispatch` | durable-invariant | high | fs_trace() 0-vs-1 delta proves denied path never reached real dispatch |
| i5_scoped_capability.rs:437 `posix_resolution_is_descriptor_relative_and_nofollow` | durable-invariant | high | R5 false positive — SystemTime/temp_dir used only for unique scratch naming, not a timing oracle; NoFollow/descriptor-relative property is stable |
| check.rs:1348 `let_check_accepts_valid_body_at_outer_expected_type` | durable-invariant | high | fixed fixture; broad is_ok() but accept property durable, paired with a sibling reject test in-file |
| check.rs:1357 `let_check_preserves_check_mode_for_intro_body` | durable-invariant | high | fixed fixture; checking-mode propagation into let-bound lambda body is durable |
| check.rs:1371 `universe_no_type_type` | durable-invariant | high | directly encodes no-Type:Type (AC-1); foundational soundness invariant |
| check.rs:1399 `k2_omega_formation` | durable-invariant | high | encodes Ω_l : Type(suc l) per spec 16 §1.1; fixed universe-formation fixture |
| ds6a_int_deceq_certificate.rs:96 `trusted_base_delta_is_exactly_the_two_certificate_postulates` | durable-invariant | high | real before/after trusted_base() set-diff vs expected set, plus Decl::Opaque provenance check — relation, not a raw count |
| sct_completeness_nested_split.rs:143 `two_level_nested_split_flat_sibling_recursion_accepts` | durable-invariant | high | soundness-accept discriminator, paired with the reject test below |
| sct_completeness_nested_split.rs:165 `recursing_on_a_deferred_ih_slot_stays_rejected` | durable-invariant | high | documented tripwire for a specific slot-collision failure mode; adversarial construction is durable |
| ir.rs:654 `seed_examples_are_observation_limited` | transition-sentinel | low | `examples.len()==5` is a raw count on a growable seed list, and the match over RuntimeObservation is already exhaustive by construction (only 2 variants) so it can't fail regardless of count — this is the exact "milestone census frozen as invariant" anti-pattern the QA playbook itself names as its motivating failure. Needs either a `[placeholder — reifies when seed #6 lands]`-style label, or drop the count in favor of non-empty+exhaustive-match. Not ergo-owned (Runtime/Foundation); routing, not fixing. |
