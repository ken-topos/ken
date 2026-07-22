# RT-SPLIT slice 7 (final) — runtime-qa bound verdict

Committed directly (convo MCP client outbound still dead this session;
inbound notifications kept working throughout — same pattern as
`RT-SPLIT-slice5-runtime-qa-verdict.md @ 53501ffe` and
`RT-SPLIT-slice6-runtime-qa-verdict.md @ a4473ab0`).

## Verdict

**APPROVE — `wp/rt-split-7-artifact-internals @ 1b2d3e96b28d82e090787bc22a6595b3cad2090b`**,
base exact `origin/main @ ab7ad89fd998eb1c6e5c353f9d294349d24d5d8b` (merge-base
verified myself, exact match; the one commit landed on `origin/main` since —
`9d2b4feb`, DOC-W1-2 — touches nothing under `crates/ken-runtime/src`).

This is the seventh and final RT-SPLIT slice: `cranelift_backend.rs` goes from
1,445 to **492 lines** (independently counted).

### Build/test

- `cargo build -p ken-runtime`: 1 warning (`dead_code` on
  `SourceContinuationTerminal::ReturnValue`), matching prior slices exactly.
- `cargo test -p ken-runtime --lib --no-run`: 0 warnings — independently
  confirmed the asymmetry is real, not a stale-cache artifact: `ReturnValue`
  is constructed in production (`lowering/mod.rs:3623,3987`) and matched in
  test-only code (`lowering/core/tests/control.rs`, `lowering/core.rs:2079`),
  so the "never constructed" lint only fires in the lib-only build.
- `301/301` lib tests green, both before and after my mutation-proof (see
  below).
- `ken-cli --test px4b_native_production`: **13/13 green**, independently
  re-run myself (80s), including
  `naked_process_ir_helpers_are_not_public_production_api` — this is the
  cross-crate declaration-text oracle no `-p ken-runtime` config can observe.

### No-re-touch (SIXTH consecutive slice) — independently reproduced

- `git diff ab7ad89f..1b2d3e96 -- .../lowering/core.rs` → **0 bytes**.
- `git diff ab7ad89f..1b2d3e96 -- .../lowering/mod.rs` → **0 bytes**.
- `git diff 7c6e03c8..1b2d3e96 -- .../artifact/api.rs` → **0 bytes** (byte-
  identical against the slice-6 tip, as claimed).

### AC-2 — exported symbol identity

Rebuilt rustdoc independently on both the candidate and a detached
`origin/main @ ab7ad89f` checkout (removed after); extracted
`(struct|enum|fn|trait|type|constant|union|macro).NAME.html` from
`target/doc/ken_runtime/all.html` on each — **byte-identical, 338 symbols both
sides, empty diff.** Trait-impl count independently re-confirmed at 8, all
still confined to untouched `surface.rs` (verified by direct line count, not
by trusting the prior slice's figure).

### The final explicit-facade cut (§10.4) — independently re-derived

`pub use surface::*;` → explicit lists. I did **not** trust the commit's own
"set-equality assertion" claim — I independently grepped every module-level
`^pub \|^pub(` declaration in `surface.rs` (19 `pub` items + 1 `pub(crate)`
item, `backend_module`) and diffed that against the literal names in the new
`pub use surface::{...}` / `pub(crate) use surface::backend_module;` block:
**exact match, both directions, 19/19 plus the one `pub(crate)` name.** The
two `pub(super)` items (`unsupported`, `backend`) are correctly excluded —
neither has a facade-crossing consumer (every in-subtree user names
`super::surface::` directly, which I did not re-derive myself but is
consistent with the zero-widening budget holding).

### AC-7 — production visibility ledger

Extracted every newly-`pub`-visible line across the slice's moved/added files
(`artifact/mod.rs`, the `lowering/core/tests/*` tree, `test_support.rs`,
`artifact/tests.rs`) and checked each against its bare-private declaration on
`origin/main @ ab7ad89f`:

- The 5 `_for_lowering_tests`/adapter functions (`new_jit_module_for_...`,
  `new_object_module_for_...`, `compile_expr_for_...`, `native_isa_for_...`,
  `native_platform_target_name_for_...`) are each a `#[cfg(test)]`-gated
  **one-line delegation** beside a bare-private original — confirmed by
  reading the actual code, not inferred from the commit message.
- `total_primitive` (bare-private on `main` → `pub(super)` in
  `test_support.rs`): the whole module is still declared
  `#[cfg(test)] mod test_support;` at `cranelift_backend.rs:26` (I checked the
  line directly — the attribute sits one line above the `mod` keyword) —
  doesn't exist in non-test builds, not a production widening.
- No other symbol changed visibility. **Zero production widenings this
  slice**, consistent with the frame's own claim that the 22/24 budget "stays
  22/24" through slice 7 (`docs/program/wp/rt-split-cranelift-backend.md:1557-1558`,
  read directly).

### AC-8 — no-production-consumer / cross-tree import discipline

The new `pub(in crate::cranelift_backend) use ...` blocks in
`lowering/core/tests/mod.rs` are a **ruled test module** (AC-8 class 2 —
imports permitted). I verified the aliasing claims line-by-line against
actual usage rather than trusting the commit prose:

- `compile_expr`/`new_jit_module`/`new_object_module` aliased in
  `tests/mod.rs` from the 3 corresponding `artifact::*_for_lowering_tests`
  adapters — confirmed.
- `native_isa_for_lowering_tests` is aliased **directly** in `effects.rs`
  (not through `tests/mod.rs`) — confirmed via grep, matching the commit's
  own note that its crossing consumer moved to `effects.rs` during the
  placement fold.
- `native_platform_target_name_for_lowering_tests` is aliased **directly** in
  `constructors.rs` — confirmed, same pattern.
- `artifact/tests.rs` aliases `require_i64_for_artifact_tests` and
  `verify_cranelift_function_for_artifact_tests` from `lowering::mod` — both
  pre-exist there (part of the 0-byte-diff file, i.e., landed in an earlier
  slice), confirmed present.

### Rule-8 / AC-9 residual placement — structural presence check

Rather than re-deriving the full per-item final-user-LCA table (that
judgment belongs to whoever holds the placement-ruling context, per my own
slice-6 retro), I ran a structural conservation check: extracted every
top-level `fn` name from the base `cranelift_backend.rs` (`ab7ad89f`, 1,445
lines) and confirmed each standalone item (`compile_expr`,
`total_primitive`, `native_isa`, `recursive_computational_result_depth`, the
full `px8*`/`emit_*` set, etc.) appears **exactly once** somewhere in the
current `cranelift_backend/` subtree — no loss, no duplication. (Names that
recur, `child`/`drop`/`from_program`, are common impl-method names on
distinct types, not the residual items in question — expected noise, not a
defect.)

### Frame reconciliation commit (`1b2d3e96`) — verified scope

The commit claims to carry Steward's frame-text corrections on the branch
"so main does not move under a held WP merely to fix prose." I checked the
full file list: it touches `artifact/mod.rs` (26 lines),
`RT-SPLIT-slice7-residual-placement-ledger.md`, and
`rt-split-cranelift-backend.md`. Read the `artifact/mod.rs` hunk directly —
it is a **comment-only edit** (documents the adapter census being re-derived
after, not before, the placement fold); no code line changed. Consistent
with the claim.

### Live mutation-proof

Targeted `recursive_computational_result_depth`
(`lowering/core/tests/mod.rs`), flagged in the implementer's own commit as a
placement self-correction (their first pass missed that `tests/mod.rs`'s own
code calls it). Mutated it to ignore its `depth` parameter
(`let depth = 0 * depth;`) and ran the real consumer,
`px8j_all_three_producer_paths_reach_real_consumers` (`control.rs`) — **failed
with `SourceMachine must mint a recursive IH`**, the expected consequence of
collapsing the recursive tree fixture to a single leaf. Reverted cleanly
(`git diff --stat` empty), `301/301` green again.

No other open findings. Worktree clean, branch released back to
`runtime-qa/work` immediately after this pass.

## Retro (trap / held / carry)

- **trap:** none this slice. The two near-misses flagged in the slice-6 retro
  (destructive checkout, comment-substring grep) did not recur — I used
  `git show <ref>:<path>` and a detached temp worktree for all read-only
  base comparisons, and relied on structural presence checks (name-appears-
  exactly-once) rather than a raw `#[test]`/`fn` count for conservation,
  which sidesteps the comment-prose trap entirely.
- **held:** independently re-derived every claim rather than sampling —
  AC-2 symbol diff, the 19-item facade set-equality, all 5 adapter gating
  sites, both non-`tests/mod.rs` adapter aliasing sites, and the frame-
  reconciliation commit's claimed scope (I read the actual hunk rather than
  trusting "comment-only" from the message, same discipline as slice 5's
  `744bda14`). One live mutation-proof on an item the implementer's own
  commit flagged as previously mis-derived, which is the sharper target
  than picking an arbitrary moved function.
- **carry:** for a final/closing slice with a long self-correcting commit
  chain (three "the first pass was wrong" admissions across
  `25466822`/`94827902`/`a6c9462d`), the earlier corrections are informative
  about *where* to spend independent-verification effort — each admitted
  miss names a category of defect (restricted-visibility items dropped by a
  glob enumeration; a consumer outside the subtree; a parent unable to see a
  child's own private caller) worth specifically re-checking on the final
  candidate rather than assuming the last commit's self-correction closed
  the whole category. All three were independently re-verified above and
  held.
