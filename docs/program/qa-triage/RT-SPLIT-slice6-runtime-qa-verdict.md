# RT-SPLIT slice 6 — runtime-qa bound verdict

Committed directly (convo MCP client outbound still dead this session;
inbound notifications kept working throughout — same pattern as
`RT-SPLIT-slice5-runtime-qa-verdict.md @ 53501ffe`).

## Verdict

**APPROVE — `wp/rt-split-6-artifact @ 96410860e1aea21176229bd714f49be3a2d24b4e`**,
base exact `origin/main @ 706575a0` (merge-base verified myself, exact
match).

### Build/test

- Both `cargo build` and `cargo test --no-run` show the identical 2
  warnings (`unused std::mem`, dead `ReturnValue` variant) — no `cfg(test)`
  asymmetry. Down from slice 5's 3 warnings (the `RuntimeObservation`
  unused import is gone, consistent with the residual file shrinking).
- `301/301` lib tests green.
- `ken-cli` cross-crate gates re-run: `px8ta_oriented_subcontinuation` 3/3
  green; `px4b_native_production`'s source-scan test
  (`naked_process_ir_helpers_are_not_public_production_api`) still finds
  its target and passes.

### AC-2 — exported symbol identity

Rebuilt rustdoc independently on both the candidate and an `origin/main`
detached checkout; extracted `(struct|enum|fn|trait|type|constant|union|
macro).NAME.html` from `target/doc/ken_runtime/all.html` on each — **byte-
identical, 338 symbols both sides, empty diff.** Trait-impl count
independently re-confirmed at 8 (all in untouched `surface.rs`), matching
the baseline established across prior slices.

### Test-count conservation

- The 29 residual tests (2 stay-for-`artifact`, 27 move to
  `artifact/api/tests.rs`) match `origin/main`'s pre-slice-6 residual count
  of 29 exactly by name (diffed the full function-name sets — empty diff,
  pure relocation).
- **Near-miss worth recording**: my first crate-wide `#[test]` grep read
  **302**, one over the expected 301. Traced it to a grep false-positive —
  `test_support.rs`'s own doc comment states *"Contains no `` #[test] ``
  cases..."* and the literal substring `#[test]` inside that prose matched
  my pattern. Re-counted with a comment-excluding pattern
  (`grep -v '^\S*:\s*//'`): genuinely **301**, matching baseline exactly.
  Caught before it went into this verdict — flagging the near-miss anyway
  since it's exactly the kind of self-referential grep trap worth naming.

### AC-7 — production visibility ledger

Checked every new `pub`/`pub(super)`/`pub(crate)` line in the touched
files against `origin/main`'s original visibility for the same symbol. All
relocated functions (`run_nc6_seed_examples`, `run_nc8_validated_seed_examples`,
`emit_bound_process_program_object_with_cranelift`,
`run_process_expr_with_cranelift`, and others) carry their **pre-existing**
visibility unchanged (`pub`→`pub`, `pub(crate)`→`pub(crate)`) — moved
verbatim, not widened.

The one exception, `test_only_distinguished_root_join_plan`
(bare-private on `main` → `pub(super)` in the new `test_support.rs`): the
whole module is declared `#[cfg(test)] mod test_support;` at
`cranelift_backend.rs:19-20` — doesn't exist in non-test builds, matching
the established test-scaffold precedent from slices 4/5. Not a production
widening.

### AC-8 — no-production-consumer ledger

Independently verified all 4 cited direct-rooted-use line citations from
the commit message (`cranelift_backend.rs:168,437,482,947`) — each is
exactly `crate::cranelift_backend::test_support::
test_only_distinguished_root_join_plan()`, matching the ledger's claim.
Zero production consumers confirmed.

### Live mutation-proof

Disabled the `RuntimeIrRunReport` identity-match guard
(`artifact/api.rs`, the `package_identity`/`core_semantic_hash`/
`runtime_artifact_hash` triple-check) — the targeted test
(`nc22_runtime_ir_report_identity_mismatch_rejects_before_native_lowering`)
failed with the exact predicted assertion failure rather than passing
silently, confirming a genuine discriminator. Reverted cleanly, 301/301
green again.

### One non-blocking documentation note

`test_support.rs`'s module doc comment states `test_only_distinguished_
root_join_plan`'s "users span `lowering` and `artifact::api`." I checked
this directly: right now, all 4 real consumers are `#[cfg(test)]` helpers
in the still-residual `cranelift_backend.rs` (feeding `lowering`-side
tests); zero consumers exist in `artifact/api/tests.rs` or the two
remaining artifact-destined (non-api) tests. This may be legitimate
forward-provisioning under rule 6a (LCA computed from *final* ruled
destinations, not current callers) — I don't have visibility into the full
Architect ruling context that grounds the facade-scope placement decision.
Not blocking approval (the AC-8/AC-7 gates that actually matter for this
slice are independently verified above), but worth the implementer/
architect confirming the wording is accurate to the final-state plan
rather than the present tense it's stated in.

No other open findings. Worktree clean on `runtime-qa/work`, branch
released.

## Retro (trap / held / carry)

- **trap:** none new this slice beyond the near-miss above (the
  `#[test]`-substring-in-comment grep false-positive) — caught before it
  reached the verdict, so no real cost, but worth the note since it's the
  same failure class as `an-oracle-that-greps-a-name-fires-on-prose-that-
  denies-it` (fleet memory) applied to my own counting method, not just a
  reviewed artifact.
- **held:** re-derived AC-2 (rustdoc symbol diff) fresh on both sides
  rather than trusting the evidence package (which I couldn't fully read
  this slice due to the convo outage); independently verified every AC-8
  line citation rather than sampling; one live mutation-proof on a genuine
  moved test confirmed real wiring.
- **carry:** when a doc comment makes a claim about a symbol's eventual/
  final consumer span that isn't yet true of the code in front of you,
  flag it precisely (which consumers exist now vs. claimed) rather than
  either rubber-stamping the prose or blocking on it outright — the
  distinction between "wrong" and "not yet materialized, forward-stated"
  usually isn't resolvable from the diff alone and belongs to whoever holds
  the full placement-ruling context.
