# Q2-language triage result

71/71 tests classified. List only — no test edits, no build (reading pass
per `README.md`). Compiled from leader (24), language-implementer (23), and
language-qa (24) shares.

**Counts: 66 durable-invariant, 0 compat-vector, 0 transition-sentinel, 5 UNCLASSIFIABLE.**

## Leader share (24 tests, 10 files)

| test | class | confidence | note |
|---|---|---|---|
| `linked_checked_write_all_observes_short_progress_and_matches_interpreter` (px8f_buffer_native.rs:188) | durable-invariant | low | count(=3) and per-write offset/length triples derive from this test's own fixed LD_PRELOAD short-pwrite fault schedule (deterministic fixture, not incidental repo census); genuinely tests "checked writeAll recurses until exhausted, matches interpreter." R5-timing-env is a false positive — verified the full test body: no `Duration`/`Instant`/`sleep`/wall-clock coupling anywhere, only the deterministic fault-injection preload. Low conf only because literals couple to fixture internals. |
| `dynamic_zero_seed_takes_the_base_case` (px8l_recursive_decl_native.rs:191) | durable-invariant | high | declarations.len()==2 is fixture-local for this test's fixed PROGRAM (finite closure = main + admitted recursive decl), not a repo census. "zero" names the seed scenario, not a quantity — R6 false positive. |
| `ac1_well_typed_def_registers` (t2_acceptance.rs:40) | durable-invariant | high | is_ok()+non-empty-globals is the right level; no success variant to name. |
| `ac1_ill_typed_def_rejected_not_registered` (t2_acceptance.rs:54) | durable-invariant | high | Error variant genuinely doesn't matter to "rejected, not registered." |
| `ac4_malformed_input_no_panic` (t2_acceptance.rs:204) | durable-invariant | high | Contract is explicitly "must not panic," any Err satisfies it. |
| `ac4_unbound_name_no_panic` (t2_acceptance.rs:212) | durable-invariant | high | Same no-panic reasoning. |
| `ac4_type_error_no_panic` (t2_acceptance.rs:220) | durable-invariant | high | Same no-panic reasoning. |
| `ac5_session_state_persists` (t2_acceptance.rs:235) | durable-invariant | high | Forward-reference-in-session; is_ok() is the right oracle. |
| `shadow_outer_not_captured` (acceptance.rs:227) | durable-invariant | high | Real discriminating pair: correct path is_ok(), hand-injected capture-bug RDecl (bypasses parser) is rejected — over the actual de Bruijn capture guard. |
| `modifier_has_zero_trusted_base_delta` (case_eq_dependent_match_sugar.rs:114) | durable-invariant | high | Set-equality trusted_base() before==after — preferred relational style; "zero" names the asserted delta, R6 false positive. |
| `cat5_d1_source_span_package_elaborates_zero_delta` (cat5_parsing_package.rs:172) | durable-invariant | high | Set-equality trusted-base pin + real elaboration of the catalog file. |
| `cat5_d2_parser_result_surface_is_total_and_located` (cat5_parsing_package.rs:290) | UNCLASSIFIABLE | low | Asserts literal substrings of compacted .ken.md text, never elaborates — verified the full test body. Fragile/non-independent oracle. Whether this text-pin is intentional (stand-in for a not-yet-written structural check) is a call for the CAT-5/Parsing package owner. |
| `cat5_d3_bool_expression_surface_is_package_owned` (cat5_parsing_package.rs:338) | UNCLASSIFIABLE | low | Same source-text-substring pattern as cat5_d2 — independently verified, full body confirmed no elaboration call. |
| `cat5_d1_source_span_surface_is_byte_artifact_and_source_explicit` (cat5_parsing_package.rs:383) | UNCLASSIFIABLE | low | **Corrected via the Q2b negative-claim sweep.** Most assertions are the same text-substring pattern as cat5_d2/d3, but this row additionally calls `ken_elaborator::literate::extract_ken_md(PARSING_KEN_MD)` — a real fence-extraction mechanism, not pure static text search — though the forbidden-token checks that follow still scan the *extracted* source as text rather than elaborating/typechecking it. The prior note ("same pattern... never elaborates or exercises") inherited d2's finding by reference without independently reading this row's tail; still UNCLASSIFIABLE/low (extraction ≠ elaboration, so the oracle is still weak), but the blanket "never exercises anything" claim was inaccurate for this specific row. |
| `cat5_d1_valid_half_open_bounds_and_zero_width_offsets_check` (cat5_parsing_package.rs:465) | durable-invariant | high | Real elaboration of half-open/zero-width span proofs. "zero_width" names the scenario, R6 false positive. |
| `constrained_pair_instance_binds_two_dicts_and_resolves_at_use_site` (constrained_instance_elaboration.rs:11) | durable-invariant | high | Typed structural assertions (head_param_count, RVarTy shape matches, pi_count/lam_count) over the real dictionary-binding mechanism. |
| `constraint_names_are_deterministic_and_surface_misuse_rejects` (constrained_instance_elaboration.rs:66) | durable-invariant | high | Positive path variant-agnostic (correct); negative path names Err(ElabError::ParseError) exactly — real discriminating pair. |
| `compound_named_and_bare_d_boundaries_are_discriminating` (constrained_instance_elaboration.rs:130) | durable-invariant | high | is_ok() over real elaboration of compound/bare constraint forms; success-variant-agnostic is correct here. |
| `char_deceq_pin1_structural_encoding` (decimal_char_acceptance.rs:200) | durable-invariant | low | Source-text pin, but guards a property IsTrue's proof-irrelevance makes unobservable at the value level (companion char_deceq_pin1_value_consequence covers the runtime-observable half). Low conf: still fragile to a behavior-preserving reformat. |
| `arity_three_eq_and_j_sugar_still_intercept_normally` (fr2_absurd_collision_hygiene.rs:90) | durable-invariant | high | Real elaboration of arity-3 Eq sugar; "three" names the sugar's arity, R6 false positive. |
| `non_colliding_ctor_alongside_a_colliding_one_still_reports_only_the_real_collision` (fr2_absurd_collision_hygiene.rs:127) | durable-invariant | high | Names exact variant (ElabError::ParseError) + message content discriminating which ctor collided. |
| `only_exact_ken_fences_compile` (ken_md_literate.rs:50) | durable-invariant | high | compiled_ranges.len()==1 over a fixed inline markdown fixture — fixture literal, not repo census. |
| `ken_example_with_valid_body_check_passes_and_is_not_tangled` (ken_md_literate.rs:196) | durable-invariant | high | Same fixture-literal reasoning; also a real structural equality (tangled source unperturbed by removing example fence). |
| `checked_fence_ranges_are_utf8_safe_with_non_ascii_prose` (ken_md_literate.rs:286) | durable-invariant | high | Fixed inline non-ASCII fixture; UTF-8 char-boundary safety is the actual property under test. |

## Implementer share (23 tests, 10 files)

| test | class | confidence | note |
|---|---|---|---|
| `foreign_axiom_and_open_obligation_trust_entries_still_count` (km_literal_trust_accounting.rs:102) | durable-invariant | high | `axiom_new.len()==1` is a fixed 1:1 per-occurrence relationship (one syntactic `Axiom` ⇒ one hole), not a growing census; rest of test verifies foreign/alias/obligation visibility in trusted_base. |
| `wrong_endpoint_pair_lens_law_still_rejects` (km_sigma_eq_pair_refl.rs:36) | durable-invariant | high | Discriminating negative: mismatched-endpoint set-get law must be rejected by Refl. is_err() doesn't pin the variant — candidate for tightening in rework, but the reject-vs-accept property is durable. |
| `componentwise_proof_is_not_unrelated_full_pair_equality` (km_sigma_eq_pair_refl.rs:53) | durable-invariant | high | Discriminating negative: componentwise-equality proof must not be accepted as unrelated full-pair equality. Independently verified (full body, not just "same as :36") — confirmed same is_err()-breadth shape, own distinct bad-lemma fixture. |
| `parser_distinguishes_facade_and_in_scope_forms_with_renames` (l4_export_reexport.rs:72) | durable-invariant | high | Fixed literal source strings parsed into fixed shapes (facade vs in-scope, renames); item counts reflect each fixture's literal item list, not a growing census — a grammar-shape contract test. |
| `ac1_binding_separators_are_context_local` (let4_multi_binding.rs:44) | durable-invariant | high | Binding-group counts are fixed by each literal fixture string; verifies `;` groups the right binders per context (let-body/match-arm/nested-let) — a parser grouping-scope contract. |
| `shape_a_val2_12_nested_split_flat_sibling_recursion_accepts` (sct_completeness_repro.rs:82) | durable-invariant | high | "val2_12" cites the originating defect ID, not a frozen quantity. Positive half of a discriminator pair with the near-miss test below. |
| `shape_a_near_miss_recurses_on_unchanged_scrutinee_stays_rejected` (sct_completeness_repro.rs:170) | durable-invariant | high | Negative twin of :82 — same syntactic shape, genuinely non-terminating (recurses on unchanged scrutinee) — must stay rejected. |
| `shape_b_bad_ack_still_rejected` (sct_reconstruction_descent.rs:178) | durable-invariant | high | Negative control proving the fix's edge is DownEq not Down (size-preserving reconstruction must reject). is_err() only, no variant check. |
| `shape_b_bad_ack2_still_rejected` (sct_reconstruction_descent.rs:197) | durable-invariant | high | Negative control: size-increasing reconstruction must stay rejected (exact-depth-match requirement). |
| `control_shape_a_near_miss_still_rejected` (sct_reconstruction_descent.rs:234) | durable-invariant | high | Cross-WP regression control: shape (a)'s own near-miss (from sct_completeness_repro.rs) must remain rejected under this WP's changes — monotonicity guard. |
| `generated_manifest_is_closed_and_probe_comparison_discriminates` (ken-host/src/lib.rs:975) | UNCLASSIFIABLE | low | Mixes a genuine tamper-fails-closed discriminator (durable) with a literal fact_count==23 that duplicates the relational fact_count==facts.len() check two lines below. Per the sealed-catalog "closed≠frozen-cardinality" lesson this may be an unintended census freeze rather than a deliberate compat-vector — needs the owning ABI-manifest WP's intent (Foundation/host territory, not ours) to call it. |
| `producer_inventory_is_bidirectional_and_sync_drift_is_discriminating` (ken-host/src/lib.rs:1041) | durable-invariant | high | Tests the actual mechanism (a source-text-based closure verifier) via genuine injected-mutation discriminators in both directions. Asserting over Rust/C source text is intrinsic to what's under test here, not incidental. |
| `public_surface_contains_only_ken_owned_semantic_types` (ken-host/src/lib.rs:1145) | durable-invariant | high | Enforces a real abstraction-boundary contract (no backend types in public API) via a literal-name source scan; the property is durable even though the mechanism is text-based. |
| `rooted_operations_preserve_bytes_and_nofollow_policy` (ken-host/src/lib.rs:1164) | durable-invariant | high | Verified: `SystemTime` only makes the temp-dir path unique (nonce), never asserted — R5 flag is a false positive. Real assertions (byte-exact read-after-write/append, symlink-follow denied) are a durable I/O-boundary invariant. |
| `cwd_root_is_resolved_once_and_preserves_scope_and_symlink_denials` (ken-host/src/lib.rs:1208) | durable-invariant | high | Independently verified (full body, not inherited from :1164): same nonce-only `SystemTime` usage. Asserts specific error variants (ScopeEscape, SymlinkDenied) plus a resolved-once/no-TOCTOU-rebind property — genuine security-relevant invariant. |
| `scripted_home_lookup_binds_isolated_roots_once_and_preserves_denials` (ken-host/src/lib.rs:1293) | durable-invariant | high | Independently verified (full body): same nonce-only `SystemTime` usage. Verifies per-UID home isolation, exactly-once lookup caching (calls.get()==1), and denial preservation. |
| `eff1_perform_observe_resume_console` (ken-interp/src/lib.rs:1603) | durable-invariant | high | ops_performed.len()==1 reflects the fixed one-Vis tree built by construction, not a growing census; verifies perform/observe/resume against an explicit oracle trace. |
| `eff2_bind_graft_threads_response` (ken-interp/src/lib.rs:1780) | durable-invariant | high | ops.len()==2 fixed by the two-Vis tree constructed; real assertion is that op1's response threads into op2's tag — bind/graft correctness. |
| `eff5_x1_trace_equals_l5_itree_denotation` (ken-interp/src/lib.rs:2069) | durable-invariant | high | trace.len()==2 fixed by the two-op tree mirroring the L5 denotation; real property is a definitional-equality check (driven trace matches ITree denotation's Vis spine via code_id identity). |
| `cast_inductive_index_rewrite` (k2c_series2.rs:141) | durable-invariant | high | args.len()==4 is vcons's fixed inductive arity (A,n,a,xs), not a growing count. Real assertion is a structural discriminant (args[3] must be a sub-Cast, not bare xs) vs. the naive ill-typed reduct. |
| `quotient_respect_type_target` (k2c_series2.rs:451) | durable-invariant | high | Full discriminator pair: well-typed respect proof accepted, wrong-typed respect rejected — the Type-target quotient-elim respect-checking contract. |
| `quotient_omega_target_respect_free` (k2c_series2.rs:730) | durable-invariant | high | Pins the K2 regression fix: Ω-target quotient elim requires no respect proof (proof-irrelevance) — a genuine design invariant, not a snapshot. |
| `sigma_eq_component_proof_checks_under_first_proof_binder` (sigma_eq_pair_refl.rs:54, ken-kernel) | durable-invariant | high | Hand-built kernel term test: Sigma-Eq componentwise-Refl proof must check under correctly-weakened first-proof binder — positive half of a discriminator pair with the adjacent "unrelated pair" negative test. |

## QA share (24 tests, 4 files)

| test | class | confidence | note |
|---|---|---|---|
| `sct_reject_self_loop` (k2c_conversion.rs:501) | durable-invariant | high | is_err + matches KernelError::NotTerminating(_); paired discriminator with sct_accept_plus per AC4 comment. |
| `sct_reject_growing` (k2c_conversion.rs:520) | durable-invariant | high | only is_err(), no variant match — weaker assertion, but the self-argument-growth-must-reject property is durable. |
| `sct_reject_ctor_wrap_compose` (k2c_conversion.rs:545) | durable-invariant | high | Comment states test fails if compose(↓,?)=↓ (wrong rule) — genuine discriminator on the SCT compose lattice. |
| `unit_eta_two_vars_convert` (k2c_conversion.rs:602) | durable-invariant | high | "two" names the discriminator (Unit-η, `17 §2`), not a frozen count; sibling unit_eta_tt_and_var covers the other case. |
| `sct_reject_union_masking` (k2c_conversion.rs:658) | durable-invariant | high | Regression pin for a real Architect-caught soundness bug (union- vs set-based closure); is_err + variant match. |
| `declare_def_sct_rejects_self_loop` (k2c_conversion.rs:700) | durable-invariant | high | only is_err(), no variant match, but self-loop-must-reject is an SCT invariant. |
| `sct_reject_bare_self_reference` (k2c_conversion.rs:802) | durable-invariant | high | Soundness regression pin (bare Const self-ref bypassed SCT pre-fix); is_err + NotTerminating match. |
| `sct_reject_combinator_laundered` (k2c_conversion.rs:832) | durable-invariant | high | Soundness regression pin (self-ref laundered through a transparent passthrough); is_err + variant match. |
| `oriented_endpoint_corruption_and_affine_reuse_fail_closed` (cranelift_backend.rs:15303) | durable-invariant | high | Asserts specific error variant+reason substring, plus a separate affine-capability double-consume rejection. |
| `oriented_fresh_ih_semantics_retain_all_inherited_control_obligations` (cranelift_backend.rs:15334) | durable-invariant | high | Exact frame/invocation id pairs + a count(4)/len(5) derived from the fixture's own construction, not an open census. |
| `oriented_dynamic_edge_ledger_is_affine_and_sibling_isolated` (cranelift_backend.rs:15612) | durable-invariant | high | Exact error-reason substrings for double-consume/duplicate-handle; reachability-grounded (calls take_dynamic_splice_edges directly). |
| `oriented_edge_mutations_reject_in_all_three_direct_consumers` (cranelift_backend.rs:15772) | durable-invariant | high | "three" names the enumerated consumer set (PendingLetProducer/ProducerCall/OrdinaryCall) × 5 named mutations with exact reason-substring assertions — a discriminator matrix, not a count. |
| `px8n_bounded_nat_observes_exact_zero_successor_and_recursive_order` (cranelift_backend.rs:15929) | durable-invariant | high | Specific numeric outputs (10, 22, 0) tied to the fixture's own chosen inputs, each with a named semantic-meaning message; fixture-derived, not census. |
| `px8n_bounded_nat_rejects_zero_over_bound_misaligned_and_wrapping_progress` (cranelift_backend.rs:15972) | durable-invariant | high | Name encodes the boundary cases under test (0-count/overflow/wraparound), not a frozen count. |
| `px8j_all_three_producer_paths_reach_real_consumers` (cranelift_backend.rs:18615) | durable-invariant | high | "three" names the enumerated producer-path set (Composed/SourceMachine/DeferredConstructor); asserts real trace events reach production mechanisms (reachability-verified via PX8J_SOURCE_TRACE). |
| `px8j_one_two_three_scope_segments_reach_selection_hole_and_unwind` (cranelift_backend.rs:18718) | durable-invariant | high | Enumerates a depth-parametrized loop (1..=3), not a frozen count; assertions are relational (unique origins == depth, parent-chaining). |
| `live_effect_emitter_inventory_and_generated_layout_mutations_are_closed` (cranelift_backend.rs:19368) | durable-invariant | high | Asserts set equality CRANELIFT_HOST_EFFECT_CONSUMERS_V1 == ken_host::NATIVE_TESTED_TARGETS_V1 (relation, not a literal count) plus per-op wire-layout mutation rejection. |
| `px8j_all_three_direct_consumers_propagate_the_role_validator` (cranelift_backend.rs:20223) | durable-invariant | high | "three" names the enumerated consumer set (as above), not a count; exact error variant+reason across all three. |
| `program_runner_preflights_metadata_before_backend_lowering` (cranelift_backend.rs:20607) | durable-invariant | medium | reports.len()==5 is the cardinality of a fixed seed-program fixture (seed_program_with_lowerability), not an open census — but Runtime should confirm the fixture can't silently grow/shrink independent of the assertion. |
| `px8i_jit_and_object_construct_identical_local_helper_clif` (cranelift_backend.rs:21894) | UNCLASSIFIABLE | low | JIT==object CLIF identity is durable, but matches("-- helper --").count()==5 — whether 5 is a fixed contract (# of native-int local helpers) or could grow with new int ops needs Runtime subject knowledge. |
| `px8i_local_helpers_reject_invalid_zero_stale_and_wrong_arena_slots` (cranelift_backend.rs:21908) | durable-invariant | high | Adversarial synthetic CLIF built to probe zero/stale/wrong-arena-slot rejection directly at the codegen mechanism; name enumerates negative cases, not a count. |
| `dedup_across_two_distinct_constructions` (store.rs:844) | durable-invariant | high | "two" names the discriminator (two independently-allocated-but-equal-content values must dedup to one slot), not a frozen count. |
| `canonical_snapshot_projects_mode_but_never_owner_namespace` (filesystem.rs:363) | durable-invariant | low | Mode-preservation half is solid; verified the never-owner-namespace half greps producer+verifier source text for absence of `metadata.uid()`/`metadata.gid()` — legitimate today but fragile against a rename/reformat of the projection call (the R4 flag has a real point); Verify (owner) may want a behavioral guard alongside the source scan. |
| `twin_roots_preserve_raw_paths_files_directories_and_symlinks` (filesystem.rs:389) | durable-invariant | high | nodes.len()==3 is the cardinality of the 3-element SeedNode fixture literally constructed above it, not an open census. |

## Q2b negative-claim correction sweep (steward directive, `evt_a4a2jwec5fgx`)

Targeted re-check of every row whose note asserts what a test *does not* do,
or that inherits a note by explicit reference ("same as above" / "same
pattern"), per the Steward's fleet-wide correction. Checked: leader's
px8f/cat5-d2/cat5-d3/cat5-d1(:383) rows, implementer's
km_sigma_eq_pair_refl:53 and ken-host:1208/:1293 (each read independently
in full rather than trusted from a prior row's finding), and QA's
filesystem.rs:363. **One row corrected**: `cat5_d1_source_span_surface_is_byte_artifact_and_source_explicit`
(cat5_parsing_package.rs:383) — its inherited "never elaborates or
exercises" claim overstated the row; it does call `extract_ken_md` (a real
mechanism), though the classification (UNCLASSIFIABLE/low) is unchanged.
All other checked rows verified accurate on independent re-read; zero
further corrections.
