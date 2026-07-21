# Q2b triage result — Team Runtime

71 tests / 24 files, classified by the Runtime ring (leader + implementer +
QA) against `docs/program/qa-triage/Q2-runtime.md`. List only — **no test was
edited**, no build was run.

**Totals:** 64 `durable-invariant` · 3 `compat-vector` · 3 `transition-sentinel`
· 1 `UNCLASSIFIABLE`.

---

## Leader slice (24 tests / 9 files)

### `crates/ken-cli/tests/effect_composition_state_console_e2e.rs`
| test | class | confidence | note |
|---|---|---|---|
| state_console_pairing_runs_through_run_state_then_run_io | durable-invariant | high | asserts exact Console residual output through composed `run_state`/`run_io`; tmp-dir/`current_dir` usage flagged as env-coupling but no wall-clock assertion |

### `crates/ken-cli/tests/ken_check_mode.rs`
| test | class | confidence | note |
|---|---|---|---|
| check_pure_library_file_exits_zero_no_io | durable-invariant | high | "zero IO" is the literal contract of `ken check` on a pure file, not a frozen census |
| check_file_with_failing_fence_exits_one_same_error_path | durable-invariant | high | exit code 1 via the shared `elaborate_ken_md_file` Err path is the contract |
| check_io_shaped_file_also_exits_zero_without_running_io | durable-invariant | high | check must never drive IO regardless of shape |

### `crates/ken-cli/tests/px4b_native_production.rs`
| test | class | confidence | note |
|---|---|---|---|
| real_source_builds_one_identity_bound_linked_process_artifact | durable-invariant | high | hash-equality across package/runtime_program/artifact is the identity-preservation contract |
| authority_mismatch_fails_before_any_artifact_is_written | durable-invariant | high | rejects a specific `Admission` error variant + zero files written on ABI mismatch; the 0-file check is a fail-before-write guarantee, not a census |
| linked_console_broken_pipe_reaches_ken_instead_of_signal_termination | durable-invariant | high | primary assertion is behavioral (`BrokenPipe` → `IoErrorIdentityV1`, not signal death); trailing source-text grep for SIGPIPE/sigaction is fragile documentation, not load-bearing |
| fs_write_and_read_resume_through_the_native_capability | durable-invariant | high | real e2e FS capability round-trip through the native build |
| canonical_fs_identity_exactly_matches_across_real_producers_and_drift_fails | durable-invariant | high | cross-producer identity match is exactly the intended discriminator |
| naked_process_ir_helpers_are_not_public_production_api | durable-invariant | low | source-text grep is the only available mechanism for a negative "not public" claim short of a compile-fail/API-surface test; a benign rename of these helpers would trip it without violating the real invariant |

### `crates/ken-cli/tests/rosetta.rs`
| test | class | confidence | note |
|---|---|---|---|
| rosetta_examples_match_their_oracles | durable-invariant | high | the timeout/poll loop is a liveness guard against a hung child, not a timing assertion; the oracle-match property itself is untimed |

### `crates/ken-elaborator/src/checked_core.rs`
| test | class | confidence | note |
|---|---|---|---|
| body_view_recovers_package_bound_primitive_literal_and_application | durable-invariant | high | producer→view faithfulness for a primitive-literal application fixture |
| body_view_recovers_recursive_imported_and_dictionary_package_seams | durable-invariant | high | same producer→view faithfulness class, recursive/dictionary seam |
| body_view_recovers_package_bound_constructor_and_match_view | durable-invariant | high | `branches.len()==2` is scoped to this fixture's own two-constructor family, not a global census |
| body_view_recovers_package_bound_record_sigma_construction | durable-invariant | high | producer→view faithfulness for record/Sigma construction |
| emitter_adds_v0_schema_hashes_and_validates_representative_fixtures | UNCLASSIFIABLE | low | `assert_eq!(fixtures.len(), 1)` freezes the representative-fixture corpus at exactly one entry even though the loop body already handles an arbitrary count; need to know whether `representative_checked_core_fixtures()` is meant to grow — if so this line should drop and let the loop do the work |

### `crates/ken-elaborator/src/compiler_driver.rs`
| test | class | confidence | note |
|---|---|---|---|
| erased_whole_match_does_not_precede_the_runtime_match_census | durable-invariant | high | ordinal vector `[None, Some(0)]` is this fixture's own two-branch shape (one erased, one genuine), not a frozen global count |
| erased_constructor_parameter_and_live_ih_argument_emit_one_call_template | durable-invariant | high | "exactly one IH call template, not two" is the actual erasure contract for this fixture's erased+live constructor args |
| real_source_reaches_checked_core_and_selects_stable_target | durable-invariant | high | single-target selection contract for the named `StableSymbol` target |

### `crates/ken-elaborator/tests/cc5_pretty_doc_acceptance.rs`
| test | class | confidence | note |
|---|---|---|---|
| all_three_laws_are_checked_and_consumable_as_proofs | durable-invariant | high | the three named law probes staying transparent/consumable is CC5's own contract |
| cc5_has_zero_trust_delta_and_keeps_string_at_the_boundary | durable-invariant | high | `trusted_base()` before==after is a real relational check over a non-trivial env; source-substring checks pin the opaque-string boundary, which is the CC5 AC |

### `crates/ken-elaborator/tests/cc6b_path_posix_acceptance.rs`
| test | class | confidence | note |
|---|---|---|---|
| package_is_extracted_and_adds_zero_trust | durable-invariant | high | same zero-trust-delta-at-extraction pattern as CC5, legitimate |

### `crates/ken-elaborator/tests/class_field_purity.rs`
| test | class | confidence | note |
|---|---|---|---|
| surf2_d1_proc_traverse_parses_elaborates_and_registers_metadata | durable-invariant | high | field_names/types/purities are the declared class's own contract, not a frozen external census |
| surf2_d1_unmarked_classes_and_sort_discriminant_stay_status_quo | durable-invariant | high | asserts unmarked fields keep prior default-purity behavior |

---

## Implementer slice (19 tests / 7 files)

### `crates/ken-interp/tests/l1_acceptance.rs`
| test | class | confidence | note |
|---|---|---|---|
| ac1_int_exact_above_2_53 | durable-invariant | high | structural value equality proves exact Int arithmetic past f64 precision; name pins the boundary condition, not a frozen quantity — R6 false positive |
| ac3_overflow_obligation_emitted_int32 | durable-invariant | high | `obligations.len()==1` is scoped to one elaboration of one expression, not a global census; also checks `PartialPrim` kind + V2 provenance structurally |
| ac3_overflow_obligation_dischargeable_structure | durable-invariant | high | same per-expression scoping; checks Pi-abstracted goal shape + open-hole state |
| ac4_bare_overflow_never_wraps_silently | durable-invariant | high | core soundness AC — obligation must be emitted, hole stays open; avoids asserting the wrapped value to dodge a silent-wrap false green |
| ac4_explicit_wrapping_is_modular | durable-invariant | high | zero obligations for `+%` + exact modular value; the wrapping contract is definitional |
| ac5_no_implicit_cross_type_coercion | durable-invariant | medium | real property (no widening coercion), but oracle is broad `is_err()` not a named `ElabError` variant — legitimate R2, worth tightening in Q3+ |
| ac5_explicit_conversion_is_partial_option | transition-sentinel | high | `#[ignore]` named to a specific follow-on WP (L-classes/conversion) — correctly labelled |
| sec31_int_div_zero_emits_obligation | transition-sentinel | high | `#[ignore]` named to div-op registration, not yet in scope |
| sec24_char_excludes_surrogates | transition-sentinel | high | `#[ignore]`, empty body, named to the Char-literal surface-syntax WP; legitimate placeholder |
| sweep_int8_overflow_emits_obligation | durable-invariant | high | same per-expression obligation-count reasoning as the AC3 pair; cross-width sweep |
| sweep_int_total_no_obligation | durable-invariant | high | Int (arbitrary-precision) totality of `+` is load-bearing numeric-tower design — zero obligations ever |

### `crates/ken-elaborator/tests/i7_env_process_projectors_acceptance.rs`
| test | class | confidence | note |
|---|---|---|---|
| i7_adds_zero_trusted_base_and_no_forbidden_mechanism | durable-invariant | high | before/after `trusted_base()` SET equality (not a frozen literal) + forbidden-token source scan |

### `crates/ken-elaborator/tests/kenfmt_b1_lossless.rs`
| test | class | confidence | note |
|---|---|---|---|
| leading_trailing_and_interstitial_comments_have_stable_unique_homes | durable-invariant | high | `comment_attachments().len() == comment_count` is a bijection between two independently-derived quantities, not a frozen literal — R1 false positive |

### `crates/ken-elaborator/tests/kenfmt_let_layout.rs`
| test | class | confidence | note |
|---|---|---|---|
| ac3_ac4_nested_simple_bindings_fit_or_break_as_one_typed_chain | compat-vector | high | kenfmt's canonical pretty-printed text genuinely is the contract; also layers AST/resolved-shape round-trip + formatter fixed-point checks |
| ac6_ac7_worked_six_binding_proof_has_an_exact_readable_fixed_point | compat-vector | high | same reasoning, larger fixture pinning canonical layout of a nested let-chain inside a multi-binding proof |

### `crates/ken-elaborator/tests/n4_program_admits.rs`
| test | class | confidence | note |
|---|---|---|---|
| admitted_ambient_resolution_records_distinct_provider_provenance | durable-invariant | high | `resolutions.len()==2` is scoped to the 2-provider fixture built in-test, not a global census; checks exact provenance fields + distinct instance_ids |
| unadmitted_direct_dispatch_rejects_after_selection_and_one_line_admit_flips | durable-invariant | high | non-degenerate discriminator pair (unadmitted rejects, admitted succeeds on the same fixture); structured `UnadmittedInstance` fields, not `is_err()` |

### `crates/ken-elaborator/tests/state_effect_build_eff6_integration.rs`
| test | class | confidence | note |
|---|---|---|---|
| ac2_runstate_next_post_increment_through_real_interp | durable-invariant | high | builds the ITree term from real prelude combinators (not hand-fed), drives it through the real interpreter end-to-end; outputs are actual execution results, not a frozen census |

### `crates/ken-elaborator/tests/sub1_bytes_structural_view.rs`
| test | class | confidence | note |
|---|---|---|---|
| ac4_roundtrip_propositions_are_usable_but_not_refl_reductions | durable-invariant | medium | kernel-soundness-relevant: primitives are conversion-opaque so bare `Refl` must not close a postulated roundtrip law; trust-set equality is exemplary but the negative case uses `is_err()` not a named variant — legitimate R2, worth tightening |

---

## QA slice (28 tests / 8 files)

### `crates/ken-elaborator/tests/surface_transport_acceptance.rs`
| test | class | confidence | note |
|---|---|---|---|
| ill_typed_transport_wrong_equation_is_kernel_rejected | durable-invariant | high | AC1 soundness negative: any ill-typed J base must reject; the error variant is incidental, the rejection itself is the permanent contract |
| ill_typed_transport_wrong_witness_type_is_kernel_rejected | durable-invariant | high | same — J's `eq` arg must be Eq-typed; non-Eq witness must always reject |
| transport_package_adds_zero_trusted_base_delta | durable-invariant | high | "zero" is the intended permanent bound (combinators must always reduce through already-trusted J/Cast), not a milestone census — R6 misreads a trust-surface invariant as a count |

### `crates/ken-elaborator/tests/surface_unicode.rs`
| test | class | confidence | note |
|---|---|---|---|
| surf1_d3_formatter_emits_canonical_unicode | compat-vector | high | pins the exact canonical-spelling contract (ASCII→Unicode token mapping); changing an output glyph is a grammar-contract decision |
| surf1_d3_rejects_unbounded_unicode_identifiers | durable-invariant | high | identifier grammar bound; the lex-error variant doesn't matter, only that these scripts never lex as identifiers |
| surf1_d3_membership_glyph_is_not_let_delimiter | durable-invariant | high | non-degenerate pair (ASCII `in` accepted, `∈` rejected) on the same delimiter position |

### `crates/ken-elaborator/tests/val1_string_literals.rs`
| test | class | confidence | note |
|---|---|---|---|
| ackermann_sct_gap_closed | durable-invariant | high | documented closed-gap regression pin (GAP-ackermann-sct); SCT must keep accepting this genuinely-terminating lexicographic recursion — a capability that must not regress, not a snapshot |

### `crates/ken-kernel/tests/ax2_named_postulate_inertness.rs`
| test | class | confidence | note |
|---|---|---|---|
| changing_only_an_opaque_label_changes_audit_text_not_typing | durable-invariant | high | postulate identity/typing independent of the audit label string |

### `crates/ken-kernel/tests/conv_eq_congruence.rs`
| test | class | confidence | note |
|---|---|---|---|
| eq_congruence_reconstructed_lhs_converts | durable-invariant | high | Eq congruence componentwise recursion; documented discriminating flip (git-stash the arm → fails) |
| eq_congruence_both_sides_reconstructed_converts | durable-invariant | high | both non-endpoint components — rules out a single-hardcoded-slot special case |
| eq_congruence_distinct_endpoint_stays_rejected | durable-invariant | high | AC4 over-conversion gate, the soundness-critical negative pairing with the two above |

### `crates/ken-kernel/tests/k5_omega_fragment.rs`
All 9 durable-invariant / high — exclusively kernel soundness gates on `Absurd`/SCT, not census-shaped.

| test | class | confidence | note |
|---|---|---|---|
| absurd_type_motive_can_mention_context_variables | durable-invariant | high | — |
| refl_does_not_inhabit_constructor_disjoint_equality | durable-invariant | high | — |
| neutral_index_equality_cannot_feed_type_absurd | durable-invariant | high | — |
| absurd_proof_must_actually_be_bottom | durable-invariant | high | — |
| absurd_type_motive_proof_must_actually_be_bottom | durable-invariant | high | — |
| absurd_motive_must_still_classify_as_a_sort | durable-invariant | high | — |
| sct_rejects_self_reference_laundered_through_absurd | durable-invariant | high | Architect-flagged HARD gate, flip empirically verified |
| sct_rejects_self_reference_laundered_through_type_absurd_motive | durable-invariant | high | same mechanism, motive position |
| antisym_shaped_case_split_both_branches | durable-invariant | high | composite two-branch shape matching the real ES4-lawproofs blocker |

### `crates/ken-runtime/src/canonical.rs`
| test | class | confidence | note |
|---|---|---|---|
| float_minus_zero_distinct_from_plus_zero | durable-invariant | high | IEEE754 signed-zero distinction is a permanent property of canonical encoding (design doc §1.1), survives any future Value-variant additions |

### `crates/ken-runtime/src/native_execution_differential.rs`
All 7 durable-invariant / high — every R1 "count literal" flag here is a false positive: each `len()` assertion is derived from the specific fixture inputs constructed in-test (e.g. 2 supplied cases → 2 reports), not a frozen milestone census.

| test | class | confidence | note |
|---|---|---|---|
| suite_runner_preserves_per_case_lane_reports | durable-invariant | high | — |
| closeout_report_recommends_nc28_for_full_chain_starter_corpus | durable-invariant | high | — |
| closeout_frames_prerequisite_when_interpreter_lane_is_unavailable | durable-invariant | high | — |
| closeout_classifies_interpreter_mismatch_as_failed | durable-invariant | high | — |
| closeout_classifies_runtime_ir_mismatch_inventory_as_failed | durable-invariant | high | — |
| closeout_classifies_deferred_effect_policy_as_unavailable | durable-invariant | high | — |
| closeout_rejects_overclaimed_out_of_phase_proof_lane | durable-invariant | high | forcing a proof-lane to Proved without real evidence must not let closeout accept the overclaim — a no-overclaim soundness gate |
