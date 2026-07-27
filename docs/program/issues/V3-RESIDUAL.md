---
id: V3-RESIDUAL
title: "V3's suite has FOUR assertion-free placeholder tests carrying ordinary names — `disproved_carries_countermodel` asserts nothing, passes, and reads in cargo output exactly like a real pin"
status: merged
owner: verify
size: L
gate: G2-G3
depends_on: []
blocks: []
github: https://github.com/swe-toolkit/ken/pull/1103
origin: "Measured by the Steward 2026-07-27 at origin/main f1f626f7 while looking for Verify's next WP. V3 is BUILT (crates/ken-elaborator/src/prover.rs + tests/v3_acceptance.rs) — this node is its RESIDUAL, not the WP. Frame docs/program/wp/V3-prover.md (blob b7442ba2; its status header is STALE and re-pinned by this node). Unblocks SEC1-IFC-R3."
---

> ## ⛔ READ FIRST — V3 IS ALREADY BUILT. This node is the RESIDUAL.
>
> ⛔ **Do not implement the prover from scratch.** It exists:
> `crates/ken-elaborator/src/prover.rs` (blob **`f3df5c51`**, 392 lines) plus
> `crates/ken-elaborator/tests/v3_acceptance.rs` (blob **`7cd52414`**, 467 lines,
> **15 tests, green**).
>
> ⚠ **The frame's own status header is STALE and this node re-pins it.** It reads
> *"enclave WP after X1-effects-elab; spec-leader elaborates
> `spec/20-verification/23-prover.md` (DRAFT → implementation-ready), then Team
> Verify builds."* ⛔ **That elaboration is DONE** — `23-prover.md` (blob
> **`a64bd2a6`**, 407 lines) says *"Status: **V3 elaborated**
> (implementation-ready). Normative for the prover."* ⇒ No enclave step is owed.
>
> ⚠ ⛔ **Do NOT sequence this from `spec/SPEC-PROGRESS.md`.** Its row for
> `23-prover.md` says `DRAFT`, and that column is unmaintained — 47 of its 48
> rows say `DRAFT` and the legend's `REVISED` rung has never once been used.

## ⭐⭐ The objective — and the defect is not "the backend is missing"

`v3_acceptance.rs` contains **four `#[test]` functions whose entire body is
`let _ = "placeholder";`**. They contain **no assertion of any kind**.

⇒ **A Rust `#[test]` with no assertions PASSES.** So all four are **unconditional
green** and all four count toward the suite's reported 15.

| test | V3 acceptance item it is named for |
|---|---|
| `disproved_carries_countermodel` | ⭐⭐ **`AC2` "honest four-way"** — the `disproved` arm of the verdict trichotomy |
| `reflective_decision_computes_cert_d` | `AC4` reflective bridge, D-fragment (`23 §3.1`) |
| `kripke_embedding_cert_rechecks_fo` | `AC4` reflective bridge, FO/Kripke (`23 §4`) |
| `induction_descent_with_ih_and_localized_partiality` | HO sub-obligation descent (`23 §5`) |

⛔ **So `AC2` and `AC4` of V3's own acceptance list are not discharged**, while the
suite is green and the WP reads as delivered.

### ⭐⭐⭐ THE LOAD-BEARING PART — the four names are indistinguishable from real pins

`v2_acceptance.rs` has the same situation and handles it **honestly**: its four
placeholder tests are **named** `*_placeholder`
(`conditional_branch_boolean_equation_placeholder`,
`nonrecursive_degenerate_no_ih_placeholder`, …). ⇒ `cargo test` output *says* they
are placeholders.

⛔ **`v3_acceptance.rs`'s four carry ordinary, affirmative names.** Its
disclosure lives **only in the doc comment** (`[placeholder — reifies in V4]`) —
and ⛔ **`cargo test` never prints doc comments.**

⇒ ⭐ **`test disproved_carries_countermodel ... ok` is, at the output layer,
indistinguishable from a real passing pin of the countermodel contract.** The
suite's green line is what a reviewer, a gate, and a future Steward read. **V2
made this visible and V3 did not** — and V2's convention is the fix.

## The second measurement — the verdict is unreachable, not merely unpinned

`grep -rn 'Verdict::Disproved *{' crates/*/src/` returns **six** hits and **all
six are pattern-match arms** (the enum declaration, two `protocol.rs` mappings,
`diagnostics.rs` rendering, `export.rs`'s refuse-to-export guard, a `repl.rs`
display arm). ⛔ **Nothing in production constructs it.** Every route bottoms out
in `Proved` or `Unknown`:

```
attempt_obligation → classify → { Route::D, Route::FO, Route::HO }   -- exhaustive
  attempt_d   → attempt_ipc, else emit_unknown_hole
  attempt_fo  → attempt_ipc skeleton, else emit_unknown_hole
  attempt_ho  → attempt_ipc
  attempt_ipc → Verdict::Proved { cert }  (kernel-checked)  |  emit_unknown_hole
```

⛔ **Positive control** (a zero-hit grep is worthless alone): the identical
pattern **does** find a construction at
`crates/ken-elaborator/tests/sec1_acceptance.rs:70`. ⇒ The key is right; the
absence is real.

## ⭐ What IS delivered — do not rebuild it

Measured present and non-placeholder in `v3_acceptance.rs`:

- **`AC1` sound-by-re-check, with the verdict flip actually exercised** —
  `discharged_goal_cert_kernel_accepts` (correct cert → `proved`, goal **absent**
  from `trusted_base()`) against `corrupted_cert_kernel_rejects_unknown`
  (corrupted cert → `unknown`, goal **present** in `trusted_base()`), plus
  `classically_valid_topos_invalid_cert_rejected`. ⭐ That is a real flip on a
  real oracle, not a shape assertion.
- **`AC3` exhaustive classifier** — `classify_routes_each_shape_d_fo_ho` and
  `unrecognized_shape_to_ho_default_no_skip`.
- **`AC2`'s `unknown` half** —
  `unknown_hole_trusted_base_distinct_from_proved` (the `23 §1.3` honesty guard:
  `proved` ↔ absent from base, `unknown` ↔ present), and
  `verdict_keyed_by_id_no_side_channel`.
- **`AC4`'s negative direction** — `bare_unsat_no_cert_is_unknown_not_proved`.
- **`AC5` no-regression** — `pure_pipeline_no_obligations_unaffected`.
- IPC: `ipc_valid_propositional_proved`, `ipc_lem_invalid_not_refuted_unknown`.

⇒ ⭐ **The soundness spine is genuinely done.** What is missing is the
*completeness* side: a route that can reach a refutation, and the evidence it
must carry.

## Fixed inputs (measured at `origin/main = f1f626f7`; ⛔ re-derive at point of use)

| input | pin |
|---|---|
| production | `crates/ken-elaborator/src/prover.rs` blob **`f3df5c51`** — 7 × `[placeholder — reifies in V4]` |
| tests | `crates/ken-elaborator/tests/v3_acceptance.rs` blob **`7cd52414`** — 15 tests, **4 assertion-free** |
| sibling convention | `crates/ken-elaborator/tests/v2_acceptance.rs` — 4 placeholders, **named** `*_placeholder`. ⭐ Copy this |
| spec | `spec/20-verification/23-prover.md` blob **`a64bd2a6`** — V3 elaborated, normative |
| diagnostics spec | `spec/20-verification/24-diagnostics.md` — the countermodel schema |
| frame | `docs/program/wp/V3-prover.md` blob **`b7442ba2`** (⚠ stale header) |

## Acceptance criteria

| AC | claim | control |
|---|---|---|
| `AC-V3r1` | ⭐⭐ **Rename first, before any mechanism work.** The four assertion-free tests are renamed to the `*_placeholder` convention `v2_acceptance.rs` already uses, so `cargo test` output distinguishes them from real pins. | ⛔ **This is a separable first commit and it must land even if nothing else in this node does.** Control: the suite's own output. ⚠ It reduces no capability and fixes the part that misleads a reader today |
| `AC-V3r2` | A production path constructs `Verdict::Disproved` with a real `Countermodel`. | ⛔ Re-run the census: `Verdict::Disproved {` must appear as a **construction** in `crates/*/src/`. ⚠ Keep the `sec1_acceptance.rs:70` positive control so a broken grep key cannot pass this |
| `AC-V3r3` | `disproved_carries_countermodel` is real: a refuted goal yields a countermodel naming the failing input class, and where the backend yields `q : ¬φ` **that is kernel-checked** (`23 §1.2`). | ⛔ Must **redden** if the countermodel is dropped or the `¬φ` cert is not re-checked. A `matches!(v, Disproved)` shape assertion does **not** discharge this |
| `AC-V3r4` | ⛔ **The cardinal rule is untouched:** `proved` is returned **only** when `check(env, Γ, cert, φ)` accepts (`23 §1.5`). | ⭐⭐ **A refutation backend must NOT become a second trust root.** Control: the existing corrupted-cert verdict-flip stays green, and `trusted_base()` deltas are unchanged for `proved`. ⛔ If a `disproved` can be believed without a kernel-checked `¬φ`, **stop and re-raise** |
| `AC-V3r5` | `reflective_decision_computes_cert_d` and `kripke_embedding_cert_rechecks_fo` are real, or **explicitly re-deferred under the renamed convention**. | ⛔ Either is acceptable; ⛔ silence is not. A deferral must be visible in the **test name** |
| `AC-V3r6` | No regression: V1/V2 obligations and T1 status consumed unchanged. | the landed suites stay green (`v1` 20, `v2` 20, `v3`, `v4` 6, `t1` 7); "no-regression" means **green in CI**, never a local `--workspace` run |

## Slicing — ⭐ `AC-V3r1` is a small, immediate, independently valuable commit

⭐ **Land `AC-V3r1` alone first.** It is a rename, it costs almost nothing, and it
removes the misleading signal **today** — independently of whether the backend
ever lands in this WP. ⛔ Do not bundle it behind the backend work.

`AC-V3r2`–`AC-V3r4` are the substantial half (**L**). ⚠ If the backend turns out
to require infrastructure not present, **land `AC-V3r1` and re-raise the rest to
the Steward with what you measured** — ⛔ do not add further placeholders.

## Scope

**IN:** `crates/ken-elaborator/src/prover.rs`, `tests/v3_acceptance.rs`, and the
countermodel type/schema wiring.

⛔ **OUT:**
- ⛔ **Re-implementing the soundness spine** — the cert re-check bridge, the
  verdict flip, the exhaustive classifier, the `trusted_base()` honesty guard.
  ⚠ They are **delivered and real**.
- ⛔ `v2_acceptance.rs`'s four placeholders — ⚠ they belong to **V2** and are
  already honestly named. Report them, ⛔ do not fix them here.
- ⛔ **No kernel enlargement.** The kernel re-check must remain the sole reason
  `proved` is believed.
- ⛔ Sec1's `[Sec1-reduce]` / `product(c, ζ)` — that is `SEC1-IFC-R3`, which this
  node **unblocks** but does not contain.

## Validation — ⛔ TARGETED ONLY

⛔ **NEVER `--workspace`** (operator, `agent/COORDINATION.md §12`). `-p
ken-elaborator`, `--test v3_acceptance` (plus `v1`/`v2`/`v4`/`t1` for
`AC-V3r6`). Workspace, `--locked`, and conformance run **in CI**.

⚠ `ken-cargo` is a single machine-wide `flock`, slots == 1 — coordinate a
seat-to-seat yield **in-thread**, ⛔ never by sampling `ps`.

## Clean-room

⛔ Permissive SMT-integration patterns are **Spec-enclave-only — never vendored,
never consulted by the implementer** (`CLEAN-ROOM.md`).

## Reporting

Return exact SHA/tree/base and specifically: **the renamed test list for
`AC-V3r1`**, **the re-run construction census for `AC-V3r2` with its positive
control**, and **the redden result for `AC-V3r3`** when the countermodel is
dropped.
