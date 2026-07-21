# Q2b triage result — Team Kernel

Per `README.md`: a list of promise-class judgments, not edits. No test in
this file was changed. Assembled by kernel-leader from three sections (leader
+ kernel-implementer + kernel-qa), each independently triaging its own file
slice of the 72-test / 25-file `Q2-kernel.md` queue.

## Section: kernel-leader (26 tests)

| test | class | confidence | note |
|---|---|---|---|
| `linked_public_second_release_is_closed_and_the_handle_closes_once` | durable-invariant | high | Count `3` is derived from the fixed `DOUBLE_RELEASE` fixture's own structure (2 public releases + 1 bracket auto-settlement), not an arbitrary/growing census — stable under any extension that doesn't reshape this specific fixture. |
| `formats_blocks_with_two_space_relative_nesting` | compat-vector | high | "Two-space" is the pinned kenfmt layout contract for block nesting, not an incidental count — changing it requires a formatting-spec decision, not a test update. |
| `cc3_checked_code_has_zero_axiom_and_zero_trusted_base_delta` | durable-invariant | high | "Zero axiom / zero trusted-base delta" *is* the claim under test (trust-base honesty for this literate entry) — unlike a census that grows over time, the invariant is that it stays zero. |
| `structural_ord_instances_and_all_laws_are_checked_zero_delta` | durable-invariant | high | "Zero delta" is the load-bearing trust-base honesty claim; the law lemmas assert structural equalities (`assert_decl`), not literal counts. |
| `list_instance_routes_the_canonical_compare_into_raw_list_compare` | UNCLASSIFIABLE | low | Asserts against literate `.ken.md` **source text** (`contains(...)`) rather than the elaborated term or executed behavior — matches the R4 source-text smell exactly. Per the Reachability gate ("would this fail if the mechanism were deleted but the string left in a comment?" — yes, it would still pass), this doesn't clearly exercise the real routing mechanism. Whether a source-text pin here is an intentional literate-doc contract or should be replaced by a structural/behavioral assertion is a design call for the test's owner, not something I can settle in a triage pass. Re-verified against the FULL test body (all 20 lines) per the Q2b correction sweep — confirmed it really is only two `contains()` string asserts, nothing else. |
| `all_def_keywords_share_named_auto_and_sole_dictionary_bindings` | durable-invariant | high | Trusted-base-delta-is-empty check exercised across all four def-path keyword forms (fn/view/const/proc) — a relational invariant, not a count. |
| `dictionaries_scope_over_contracts_refinement_and_body_but_not_siblings` | durable-invariant | high | Scoping is checked both positively (resolves in contract/refinement/body) and negatively (specific `ElabError::UnresolvedCon` on sibling leak) — a real discriminating pair. |
| `semicolon_compatibility_and_fail_closed_naming_and_type_boundaries` | durable-invariant | high | Legacy-separator acceptance plus a specific `ElabError::ParseError` on a named collision case — typed variant, not a bare `is_err()`. |
| `goal_closing_over_pre_scrutinee_type_param_narrows_correctly` | durable-invariant | high | Structural assertion on the emitted `Term::Elim` (family id, method count tied to `List`'s own 2 constructors, not a census) — a mechanism probe on context-narrowing under a non-empty, multi-var pre-scrutinee context. |
| `ac1_wtree_wstyle_match_elaborates_with_pi_shaped_ih_domain` | durable-invariant | high | Deep structural assertion on the IH's `Term::Pi` domain/body shape; `methods.len() == 2` is fixed by `WTree`'s own declared constructor count, not a growing count. Comment documents this is a load-bearing red→green pin against a named pre-fix baseline. |
| `ac2_ill_typed_wstyle_arm_stays_kernel_rejected` | durable-invariant | low | Intent is a specific-variant check (`KernelRejected`), but the assertion is `matches!(err, Err(KernelRejected{..}) \| Err(_))` — the `Err(_)` fallback makes it accept *any* error, not just the named variant, weakening the R2 discrimination the pattern flagged. Still correctly durable against the main risk (an `Ok` where a reject is required); flagging the looseness rather than fixing it (Q2 is list-only). |
| `ac8_axiom_masking_a_law_is_rejected_by_the_zero_delta_check` | durable-invariant | high | `assert_ne!` on the structural `trusted_base()` set before/after an Axiom-backed instance field — a genuine zero-delta-hazard probe with an explicit adversarial scratch class, not a proxy. |
| `eff_row_union_two_effects` | durable-invariant | high | "Two effects" is a deliberate ≥2-distinct-effects discriminator (join vs. first/last-only bug), documented in the doc comment — not a count expected to grow. |
| `cap_two_distinct_caps_each_gated` | durable-invariant | high | Three-way (a)/(b)/(c) discriminating cases, each capability independently gated with a named `MissingCapability` variant check. |
| `higher_order_two_params_each_guarded` | durable-invariant | high | Same shape as `cap_two_distinct_caps_each_gated` — each candidate effect independently guarded, named `EffectEscapes` variant checked. |
| `row_poly_two_params_each_tracked` | durable-invariant | high | Row-variable join/escape checked with an explicit accept/reject discriminating pair (`is_subset_of` both ways), not a bare count. |
| `lower_elim_itree_wrong_method_count_rejected` | durable-invariant | high | Adversarial mis-lowering (1 method for a 2-constructor family) built by hand and asserted to fail kernel `infer` — a genuine mechanism probe, verdict-flips against the correct-lowering sibling test. |
| `lower_bind_swapped_methods_rejected` | durable-invariant | high | Computes the correct reduction first (sanity), then swaps methods and asserts a *different* value results — a real discriminator, not `is_err()`-only. |
| `classify_telescope_identifies_hof_effectful_params` | durable-invariant | high | Counts (`rv_count == 1`, `classified.len() == 3`) are derived directly from the fixed input telescope's own shape, not an incidental global census. |
| `classify_telescope_hof_effectful_cannot_be_silently_dropped` | durable-invariant | high | Explicit correct-vs-misclassified verdict-flip pair (`Err` vs the wrong-path outcome) documented in the doc comment as the discriminator. |
| `sigma_event_concretizes_sigma_member` | durable-invariant | high | Both directions checked (no orphan symbol, no dropped member) per the doc comment — a real discriminating pair over the B1 alphabet contract. |
| `no_event_outside_perform_point` | durable-invariant | high | K=0/1/2-Vis discriminator explicitly documented; counts are the actual property under test (event-per-perform-point), not a census. |
| `q_p_assertion_points_project_from_export` | durable-invariant | high | Doc comment states the discriminator explicitly (hard-coded list would show no change when the export changes); asserts the projection responds to a real Q/P state change. |
| `monitor_changes_when_t_changes` | durable-invariant | high | Verdict-flip pair (Monitor1 vs Monitor2 must differ) against a documented "hand-written monitor would be unchanged" bug class. |
| `monitor_verdict_never_promoted_to_proved` | durable-invariant | high | Asserts absence of a promotion path (`guarantees.is_empty()`); the discriminator (a build with a promotion path would show non-zero `WatchedInvariant`) is explicit in the doc comment. |
| `trace_events_are_checked_against_the_one_b1_alphabet` | durable-invariant | high | Positive case (real events accepted) paired with a constructed negative case (event swapped outside alphabet, specific `TraceContractError::EventOutsideAlphabet` expected) — genuine discriminating pair. |

## Section: kernel-implementer (22 tests)

| test | class | confidence | note |
|---|---|---|---|
| `client_match_hidden_ctor_rejected_at_surface` (es3_modules_acceptance.rs:124) | durable-invariant | high | discriminates surface-vs-kernel rejection layer, not just is_err() — explicitly panics if the error is `KernelRejected`, proving the ctor never reached the kernel. Re-verified against full test body per the Q2b correction sweep. |
| `private_name_access_rejected_at_surface` (es3_modules_acceptance.rs:150) | durable-invariant | high | same shape as above — asserts is_err() AND rules out KernelRejected, a real layer-separation invariant. Re-verified independently against source (Q2b correction sweep) — the `KernelRejected` exclusion is genuinely present in this test's own body, not just copied from the sibling row. |
| `three_import_forms_resolve_to_one_binding` (es3_modules_acceptance.rs:218) | durable-invariant | high | "three" names the grammar's fixed qualified/aliased/selective import forms (33§3.3), a closed set at time of writing — not a derived census; a future 4th form wouldn't turn this red, just leave it incomplete |
| `batch3_stage1_breaks_high_and_keeps_each_fitting_child_horizontal` (kenfmt_signature_layout.rs:107) | durable-invariant | high | R4 looks like a false-positive flag — `include_str!` here pulls real catalog `.ken.md` files and round-trips them through the actual production `format_ken`/`format_ken_md`, it's not scanning Rust source text as a proxy oracle |
| `ac2_tree_size_uses_both_ihs_and_computes_right_value` (l_match_ih_fix_acceptance.rs:29) | durable-invariant | high | the "3" is the size of a 3-node tree hand-built in this same test, an intrinsic fixture value, not a count derived from an evolving external set |
| `ac3_discriminating_pair_accepts_valid_rejects_ill_typed_sibling` (l_match_ih_fix_acceptance.rs:97) | durable-invariant | high | negative half uses broad `is_err()` (no exact variant) — property itself (inconsistent-motive match rejected) is a real invariant; assertion strength could be tightened in Q3+ |
| `ac4_no_regression_0_and_1_recursive_field_types` (l_match_ih_fix_acceptance.rs:123) | durable-invariant | high | "0"/"1" name fixed structural arity classes from the bug's own bisection (non-triggering cases), not an evolving census |
| `ac4_three_recursive_fields_also_elaborates` (l_match_ih_fix_acceptance.rs:159) | durable-invariant | high | "three" names a fixed regression-coverage arity (frame's bisection went to 3); re-verified independently against source (Q2b correction sweep) rather than trusting the "same reasoning" phrasing alone — the fixture (`Tri`, 3 recursive fields) is exactly as claimed |
| `is_even_is_odd_mutual_group_elaborates_as_one_group` (mutual_recursion_surface_acceptance.rs:30) | durable-invariant | high | "2" mirrors the fixed 2-function isEven/isOdd fixture defined in this same file, not an external census |
| `non_terminating_mutual_group_is_rejected_by_sct` (mutual_recursion_surface_acceptance.rs:61) | durable-invariant | high | broad `is_err()` negative conformance (no SCT-specific variant asserted); underlying property (non-terminating mutual group rejected) is a real soundness invariant |
| `ac2_types_are_distinct_nominal_opaque_primitives` (px3_abi_scalars.rs:52) | durable-invariant | high | the "3" is the length of this test's own local 3-element array (USize/ISize/CInt); the assertion is really pairwise-distinctness, not a pinned global inventory count |
| `ac3_manifest_max_is_ok_and_max_plus_one_is_err_for_every_scalar` (px3_abi_scalars.rs:97) | durable-invariant | high | R6 looks like a false-positive flag — "one" comes from "max_plus_one" (an arithmetic +1 boundary probe), not a spelled-out stateful census |
| `ac4_trusted_delta_is_the_exact_named_twelve_entries` (px3_abi_scalars.rs:177) | compat-vector | high | this pins the exact TCB/trusted_base delta size (3 ABI types + 9 conversion ops = 12) for PX3's closed feature scope — a trust-surface count is exactly the kind of value where the count IS the contract; a silent change here is a genuine soundness-relevant event, not routine |
| `ac3_raw_narrowing_is_private_and_ac5_refl_stays_rejected` (px3_abi_scalars.rs:243) | durable-invariant | high | raw-narrowing rejection asserts the exact variant (`UnresolvedCon`); the kernel `check(...).is_err()` half is fine broad since kernel typecheck has no partial-success state |
| `checked_resource_producer_emits_exactly_one_correlated_t_body` (px7f_resource_lifetime_export.rs:68) | compat-vector | high | pins an exact canonical export hash plus a documented "exactly one target-level template, not three synthetic atoms" design decision — textbook compat-vector |
| `checked_no_acquire_producer_preserves_the_pre_px7f_t_hash_route` (px7f_resource_lifetime_export.rs:144) | compat-vector | high | pins an exact canonical export hash (regression anchor) |
| `correlated_body_is_one_member_of_the_same_t_sequence` (px7f_resource_lifetime_export.rs:183) | durable-invariant | high | "2" mirrors this test's own fixed 2-item input (one ordinary + one resource temporal it constructs), an ordering/interleaving invariant, not an external census |
| `real_checked_buffer_denotation_emits_direct_delegated_body` (px8x_static_export_projection.rs:46) | compat-vector | high | pins an exact canonical export hash for a fixed fixture — explicitly documented in-file as a frozen content-addressed identity |
| `ill_typed_arrow_domain_is_still_kernel_rejected` (surface_arrow_in_expr_acceptance.rs:110) | durable-invariant | high | broad `is_err()` soundness spot-check (arrow-domain well-formedness must stay kernel-enforced); property is real, assertion could be tightened in Q3+ |
| `f1_mul_int_2_64_squared` (f1_bignum_acceptance.rs:50) | durable-invariant | high | digits name the golden boundary magnitude (2^64) under an independently-computed oracle value, not a derived/stale census — this file is explicitly praised in the advisory §4.2 for independent-oracle practice |
| `f1_product_chain_exceeds_2_1000` (f1_bignum_acceptance.rs:78) | durable-invariant | high | same reasoning — golden vector name states the tested magnitude class, not a census |
| `f3_legacy_add_sub_mul_unregistered_in_elaborator` (f2f3_acceptance.rs:108) | durable-invariant | low | genuine R4 source-text scan (`include_str!` of `numbers.rs`, checks absence of `reg_binop!("add"` etc.) — real limitation acknowledged in the test's own doc comment: a differently-shaped re-registration path would evade this scan silently. Property is durable in intent but the oracle mechanism has a known blind spot; a second reader familiar with `ken-elaborator`'s registration surface may want to confirm no such alternate path exists |

Summary (kernel-implementer's own count): 18 durable-invariant, 4 compat-vector, 0 UNCLASSIFIABLE.

## Section: kernel-qa (24 tests)

| test | class | confidence | note |
|---|---|---|---|
| `level_max_two_distinct_vars_do_not_collapse` | durable-invariant | high | soundness law: distinct incomparable level vars never collapse under max; holds under any correct level algebra |
| `level_max_zero_absorbed_by_var_at_same_offset` | durable-invariant | high | absorption law `max ℓ 0 = ℓ` at nonzero offset; algebraic invariant |
| `ac1_type_in_type_rejected` | durable-invariant | high | universe stratification; Type:Type must always be rejected |
| `ac1_hierarchy_well_founded` | durable-invariant | high | universe hierarchy strictly increasing via suc, never a loop |
| `ac4_elim_nat_type_checks` | durable-invariant | high | eliminator well-typedness rule; holds for any valid motive/method construction |
| `ac2_pair_intro_dependent` | durable-invariant | high | Σ-intro checks 2nd component at substituted type; core typing rule |
| `ac4_elim_vec_iota_vcons_var` | durable-invariant | high | ι-reduction with live IH, asserted via structural whnf equality, not just success |
| `ac4_elim_vec_iota_open_telescope_index_correct` | durable-invariant | high | regression pinning correct IH index substitution under an open telescope (subject reduction) |
| `ac5_positive_list_admitted` | durable-invariant | high | positivity checker admits genuinely strictly-positive decls; `is_ok()` IS the property here |
| `ac5_negative_bad_rejected` | durable-invariant | high | negative occurrence must always be rejected; `is_err()` w/o variant is correct since the promise is "never admitted," not "admitted for this specific reason" |
| `ac5_negative_under_pi_rejected` | durable-invariant | high | same reasoning; also documents a seed-case literal-term discrepancy already flagged to Spec in-file. Re-verified independently against source (Q2b correction sweep) rather than trusting the "same reasoning" phrasing alone — genuinely the same-shape `declare_inductive(...).is_err()` admission check. |
| `ac5_nested_double_positive_admitted` | durable-invariant | high | double-positive is strictly positive per §8.2; must be admitted |
| `ac5_nested_negative_in_application_rejected` | durable-invariant | high | negative occurrence hidden inside an application argument must be rejected |
| `ac5_d_in_own_indices_rejected` | durable-invariant | high | D in own index telescope must be rejected |
| `ac5_d_in_constructor_target_index_rejected` | durable-invariant | high | target-index guard is kernel-side, not only the surface GADT guard |
| `ac7_checking_terminates_k1` | durable-invariant | low | asserts `infer(...).is_ok()` on a fixed 5-term suite; the name promises "terminates" but nothing times out or measures termination — only success on a small fixed list is actually checked. Sound as far as it goes but narrower than its name; flag for a possible rename, not a fix this pass |
| `k2_seam3_nonomega_quot_elim_rejected` | durable-invariant | high | named adversarial exploit regression (Architect-found); non-Ω quotient-elim motive must be rejected |
| `absurd_discharges_operation_wrapped_contradictory_hypothesis` | durable-invariant | high | ex-falso from an impossible operation-wrapped hypothesis discharges any goal; success (`is_ok()`) IS the property |
| `real_artifact_five_op_observation_matches_interp_on_twin_roots` | compat-vector | high | pins this named scenario's exact stdout bytes + 5-op trace sequence as its own defined contract, not a frozen census that would break on unrelated growth — also independently exercises the native-tested confirmation gate over `PX5_PLANNED_NATIVE_TARGETS` |
| `change_mode_is_observed_and_matches_across_real_twin_roots` | durable-invariant | high | interp/native agree on filesystem delta for chmod; includes a real negative (mutated mode must fail `compare_canonical_exact`) — genuine discriminator, not just a happy path |
| `cwd_and_absolute_root_spellings_emit_byte_identical_observations` | durable-invariant | high | R5 false positive: `SystemTime::now()` is used only for unique tempdir naming (test isolation), not as an oracle or timing assertion — the actual property (cwd-relative vs absolute path specs observationally equal) is timing-independent |
| `real_captured_evidence_mutations_bite_while_return_proxy_stays_green` | durable-invariant | high | textbook oracle-independence/mutation test: 11 named adversarial mutations must each be caught by `compare_canonical_exact`, paired with a naive return-value proxy shown blind to all of them |
| `real_scope_denial_is_typed_and_precedes_any_host_action` | durable-invariant | high | denial ordering + typed rejection property; `exit_status == 44` and empty filesystem-delta are this scenario's own fixture literals, not a census |
| `checked_write_all_reaches_full_short_zero_progress_flip_and_error_prefixes` | compat-vector | high | R5 false positive: the large-stack thread spawn is not wall-clock/env coupling. Body asserts exact syscall-trace strings + write-partition structure per named mode (full/short/zero/progress/interrupted) against a real native build — this is the differential's normative contract, not a frozen count; the compound name describes distinct branches, not a quantity. Re-verified against the FULL test body (`run_write_partition`, ~100+ lines) per the Q2b correction sweep — confirmed the thread spawn is stack-size-only, no timing assertion anywhere. |

## Q2b correction sweep (evt_a4a2jwec5fgx)

Swept all 72 rows (leader + implementer + QA) for the named risk class: a
negative mechanism claim ("does NOT do X") or a note inheriting by "same
as"/"same reasoning" reference. Five candidate rows identified and
independently re-verified against the full test body (not just the opening):
`list_instance_routes_the_canonical_compare_into_raw_list_compare`,
`checked_write_all_reaches_full_short_zero_progress_flip_and_error_prefixes`,
`ac5_negative_under_pi_rejected`, `private_name_access_rejected_at_surface`,
`ac4_three_recursive_fields_also_elaborates`. **Corrected: 0** — every note
held up against the source.

## Summary counts (all 72 rows)

- durable-invariant: 64 (3 at low confidence)
- compat-vector: 7
- transition-sentinel: 0
- UNCLASSIFIABLE: 1
