# RT-SPLIT slice 7 — final-user-LCA ledger for the 37-item residual

**Rule 8 / AC-9 deliverable** (Architect `evt_h69xwchqqxmj`). Derived at
`origin/main @ ab7ad89f` from the branch tip; **the ledger is the deliverable,
not the count** — every row states the item's final direct users so a reviewer
can re-derive the placement rather than trust a total.

## How the user set was bounded, and why it is complete

**Visibility partitions the problem structurally.** 34 of the 37 items are
private `#[cfg(test)]` declarations at facade file scope — they are
*unreachable* outside `cranelift_backend`, so an in-subtree scan is complete
**by construction**, not by exhaustive grep. Only the **3 `pub(crate)`** items
can have a consumer above the facade, and all three do.

⚠ My first pass got this wrong in **both** directions and neither error was
self-announcing: it missed `object_linker_packaging.rs` (a crate-root sibling,
outside the subtree entirely) and it produced a **false positive on `big`**,
whose bare-name matches in four other crates are unrelated local `fn big`/`let
big` bindings. The visibility partition is what makes the bound provable
instead of grepped.

## Placement

"via X" = the item's only reach is through X, so it inherits X's destination —
per AC-9's quantifier, an item reachable **only** through a lower-LCA owner
**does not have a facade LCA in the first place**.

| # | residual facade test item | final-user LCA → destination | direct final users |
|---:|---|---|---|
| 1 | `oriented_test_frame` | `lowering/core/tests/control.rs` | control×3 |
| 2 | `oriented_test_interface` | `lowering/core/tests/control.rs` | control×13, via oriented_test_frame |
| 3 | `recursive_computational_result_depth` | `lowering/core/tests/mod.rs` | control×1 (`control.rs:1185`) + `tests/mod.rs`×1 (`mod.rs:140`, `recursive_computational_result`) — LCA of a `control` user and a `tests/mod.rs` user is `tests/mod.rs` |
| 4 | `self_consistent_join_site` | `lowering/core/tests/control.rs` | control×2 |
| 5 | `self_consistent_root_join_site` | `lowering/core/tests/control.rs` | control×4 |
| 6 | `emit_process_entrypoint_object_with_symbols` | `lowering/core/tests/constructors.rs` | constructors×1 |
| 7 | `BorrowedFixtureValue` | `lowering/core/tests/effects.rs` | effects×9, via NativeInvocationFixture, via run_px8n_arm_fixture |
| 8 | `NativeInvocationFixture` | `lowering/core/tests/effects.rs` | effects×2, via run_px8n_arm_fixture, via px8n_scripted_host_dispatch |
| 9 | `PX8I_BIG_READ_START` | `lowering/core/tests/effects.rs` | effects×1, via px8n_scripted_host_dispatch |
| 10 | `PX8I_BIG_U64` | `lowering/core/tests/effects.rs` | effects×3, via px8n_scripted_host_dispatch |
| 11 | `PX8I_METADATA_BIG` | `lowering/core/tests/effects.rs` | effects×1, via px8n_scripted_host_dispatch |
| 12 | `PX8I_WRAPPING_WRITE_START` | `lowering/core/tests/effects.rs` | effects×1, via px8n_scripted_host_dispatch |
| 13 | `PX8N_OVER_BOUND_READ` | `lowering/core/tests/effects.rs` | effects×1, via px8n_scripted_host_dispatch |
| 14 | `PX8N_OVER_BOUND_WRITE` | `lowering/core/tests/effects.rs` | effects×1, via px8n_scripted_host_dispatch |
| 15 | `PX8N_READ_EOF` | `lowering/core/tests/effects.rs` | effects×1, via px8n_scripted_host_dispatch |
| 16 | `PX8N_SHORT_READ` | `lowering/core/tests/effects.rs` | effects×1, via px8n_scripted_host_dispatch |
| 17 | `PX8N_SHORT_WROTE` | `lowering/core/tests/effects.rs` | effects×3, via px8n_scripted_host_dispatch |
| 18 | `PX8N_ZERO_WRITE` | `lowering/core/tests/effects.rs` | effects×1, via px8n_scripted_host_dispatch |
| 19 | `Px8nHostReplyFixture` | `lowering/core/tests/effects.rs` | effects×1, via run_px8n_write_arm_fixture, via run_px8n_arm_fixture, via px8n_scripted_host_dispatch |
| 20 | `px8n_exact_nat` | `lowering/core/tests/effects.rs` | effects×1, via px8n_write_arm_fixture_with_start |
| 21 | `px8n_failure` | `lowering/core/tests/effects.rs` | effects×10, via px8n_write_arm_fixture_with_start |
| 22 | `px8n_scripted_host_dispatch` | `lowering/core/tests/effects.rs` | via run_px8n_arm_fixture |
| 23 | `px8n_write_arm_fixture` | `lowering/core/tests/effects.rs` | via run_px8n_write_arm_fixture |
| 24 | `px8n_write_arm_fixture_with_start` | `lowering/core/tests/effects.rs` | effects×1, via px8n_write_arm_fixture |
| 25 | `run_px8n_arm_fixture` | `lowering/core/tests/effects.rs` | effects×6, via run_px8n_write_arm_fixture |
| 26 | `run_px8n_write_arm_fixture` | `lowering/core/tests/effects.rs` | effects×3 |
| 27 | `big` | `lowering/core/tests/mod.rs` | effects×5, values×14 |
| 28 | `constructor_field_aggregate` | `lowering/core/tests/mod.rs` | constructors×3, control×1 |
| 29 | `host_result_closure_match` | `lowering/core/tests/mod.rs` | constructors×1, control×7, effects×4 |
| 30 | `host_result_computational_fixture` | `lowering/core/tests/mod.rs` | constructors×3, control×1, effects×1 |
| 31 | `ordinary_match_closure` | `lowering/core/tests/mod.rs` | constructors×5, via host_result_closure_match |
| 32 | `Px8trNestedRouteObject` | **facade (retained)** | via emit_px8tr_nested_post_effect_object ⬅ **above-facade user:** return type of the above, pub(crate) reach |
| 33 | `emit_process_entrypoint_object_with_cranelift` | **facade (retained)** | constructors×25, control×1, effects×5 ⬅ **above-facade user:** object_linker_packaging.rs:659 (crate::…) + ken-cli px4b text oracle |
| 34 | `emit_px8tr_nested_post_effect_object` | **facade (retained)** |  ⬅ **above-facade user:** object_linker_packaging.rs:681 (crate::cranelift_backend::…) |
| 35 | `px8tr_nested_post_effect_fixture` | **facade (retained)** | via emit_px8tr_nested_post_effect_object |
| 36 | `px8tr_test_interface` | **facade (retained)** | via px8tr_nested_post_effect_fixture |
| 37 | `total_primitive` | **`cranelift_backend/test_support.rs`** | api_tests×5, effects×2, values×10 |

## Totals

| destination | items |
|---|---:|
| `lowering/core/tests/control.rs` | 4 |
| `lowering/core/tests/constructors.rs` | 1 |
| `lowering/core/tests/effects.rs` | 20 |
| `lowering/core/tests/mod.rs` | 6 |
| **facade (retained)** | 5 |
| `cranelift_backend/test_support.rs` | 1 |
| **TOTAL** | **37** |

`values.rs` receives nothing: every item with a `values` user also has a user
in another subject module, so its LCA lifts to `tests/mod.rs`.

## Corrections found by executing the fold

> ⛔ **Row 3 was left stale by this very section, and that is the finding.**
> The correction below, and the totals, were both updated to agree with the
> code — **the row they correct was not.** A reader checking the table got one
> answer and a reader reading the corrections got another, from one document
> that declares every row re-derivable and binding.
>
> This is the same defect the corrections describe, one level up: **a claim and
> its correction living in two places with no mechanism keeping them in step.**
> Appending a correction is not making a correction. The row is the
> authoritative statement; the appendix explains it, and must never be the only
> place the truth appears.
>
> Caught by @architect on `dec_42h3pjehkpwfg`. On the fold I re-derived **all
> 37 rows** against the tree rather than spot-fixing the one reported —
> 36 were already consistent, row 3 was the only divergence.


Two rows were wrong and **compiling is what found them** — both the same
window defect, a user site outside the scan:

1. **`recursive_computational_result_depth`** — `tests/mod.rs`'s **own code**
   calls it, and the first scan treated `tests/mod.rs` only as a destination,
   never as a *user site*. Placing it in `control.rs` broke that caller: a
   parent cannot see a child's privates. → `tests/mod.rs`.
2. **`total_primitive`** — placement overturned by Architect
   `evt_1s7nxrjje35tk`: a genuine facade-LCA *fixture helper* has a lawful
   lower namespace home in `test_support.rs` under §10.2a rule 2, so rule 8
   point 3 cannot leave it at facade file scope. → `test_support.rs`.
