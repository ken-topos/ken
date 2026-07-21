# Q2b triage — Verify QA share

Scope: `ken-host/src/abi_v1.rs` (13), `ken-interp/tests/px8x_single_schema_observation.rs` (1),
`ken-kernel/tests/k1p5_wstyle.rs` (8), `ken-kernel/tests/obs_eq_termination_congruence.rs` (2).
24 tests. List only — no test changed.

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

## Counts

- durable-invariant: 21
- compat-vector: 3
- transition-sentinel: 0
- UNCLASSIFIABLE: 0
