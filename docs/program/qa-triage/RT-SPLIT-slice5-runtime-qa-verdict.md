# RT-SPLIT slice 5 — runtime-qa bound verdict

Committed directly (convo MCP client outbound-dead this session; inbound
notifications kept working throughout). This is the runtime-qa QA gate's own
words, unmediated — see `agent/COORDINATION.md` §15 resume discipline and the
adversary's suggested pattern (matching `ergo-qa @ cf791c7f`,
`verify-qa @ 04efa001`, both under this same directory).

## Verdict

**APPROVE — `wp/rt-split-5-lowering-support @
744bda14e863492a89740c0d65037b5b3f3b48b4`**, base exact `origin/main @
1f70a71b` (merge-base verified myself, exact match).

Bound across the full slice-5 arc, independently verified at each
superseding SHA rather than trusted on report:

### `102f4644` — 46-row residual test move (supersedes `c848f6ba`)

- Base currency: `git merge-base wp/rt-split-5-lowering-support origin/main`
  == `origin/main` tip == `1f70a71b`, verified myself.
- `core.rs`: zero-byte diff vs `origin/main`, confirmed independently — the
  §10.5 hard constraint held.
- Test-count conservation: crate-wide `#[test]` count unchanged at **301**
  (`core/tests/*.rs` 55 → 101, exactly +46, matching the corrected ledger's
  46-move/29-stay split; residual `cranelift_backend.rs` 75 → 29). Pure
  relocation — nothing added or dropped.
- Build/test-config parity: both `cargo build` and `cargo test --no-run`
  show the identical 3 warnings (unused `std::mem`, unused
  `RuntimeObservation`, dead `ReturnValue` variant) — no `cfg(test)`
  asymmetry.
- `301/301` lib tests green.
- `ken-cli` cross-crate gates: builds clean; the source-scan test
  (`naked_process_ir_helpers_are_not_public_production_api`, which
  `include_str!`s `cranelift_backend.rs` and string-matches a specific
  function signature) still finds its target — verified the relocation
  didn't silently break it.
- AC-7 visibility ledger reconciled: of the new `pub`/`pub(super)`/
  `pub(crate)` lines, 4 are re-exports of pre-existing visibility (§10.4,
  confirmed against `origin/main` — not new widenings) and 4 are genuinely
  new but entirely `#[cfg(test)]`-gated on both declaration and use sites
  (matches slice 4's Architect-ruled test-scaffold precedent — doesn't count
  against the production budget).
- Live mutation-proof: disabled the `same_recursive_argument_shapes` guard
  at both its call sites in `core.rs` (the boundary-flagged
  `recursive_declaration_shape_change_hits_typed_boundary` test) — the test
  failed with a real downstream panic (`backedges do not establish a merge
  result kind`) rather than passing silently, confirming a genuine
  discriminator. Reverted cleanly, 301/301 green again.
- Supplementary pass (both gates runtime-leader separately flagged):
  `ken-cli --test px8ta_oriented_subcontinuation` 3/3 green, including the
  test exercising the cross-crate `with_px8ds_retired_flat_order` facade
  re-export end-to-end; warning-set diff against `origin/main` baseline is a
  byte-identical relocation (same 3 warnings, same identifiers, the moved
  one relocated exactly to where `SourceContinuationTerminal` moved).

### `463d45be` — facade-glob → named-owner imports (fold)

- Delta reviewed directly (`git diff 102f4644 463d45be`): replaces the
  `use super::*` facade glob in `lowering/mod.rs` with explicit imports
  naming each item's actual owning module/crate — a real import-hygiene
  refactor, not a no-op.
- Same 3 warnings by identity: the two unmoved ones at the same lines in
  the residual file, the third relocated exactly to where
  `SourceContinuationTerminal` moved (`cranelift_backend.rs:3340` →
  `lowering/mod.rs:1734`).
- 301/301 green. Both `ken-cli` gates re-confirmed green.
- The ledger's "28 → 29" stays-count correction in this fold matches my own
  independently-derived residual count from the prior pass (cross-check).

### `744bda14` — DAG-comment fold (final candidate)

- Independently reproduced the "comment-only" claim myself rather than
  trusting the implementer's grep: `git diff -U0 463d45be..744bda14 --
  crates/ | grep -E '^[+-]' | grep -v '^[+-][+-][+-]' | grep -vE
  '^[+-]\s*(//|///|//!)'` — empty output, zero non-comment lines changed.
- Corrected comment states the real support DAG edges (`lowering support ->
  compiled, planning, surface`), matching what I'd already verified as
  genuine semantic imports (not namespace wiring) in the `463d45be` pass.
- Clean build, same 3 warnings.
- Architect's independent mechanical re-review (`evt_21y2h9zz2rbm0`)
  corroborates this same finding on the same delta.

No open findings across the arc. Worktree clean on `runtime-qa/work`,
branch released after each pass.

## Retro (trap / held / carry)

- **trap:** mid-pass, comparing test counts against `origin/main`, I ran
  `git checkout origin/main -- crates/ken-runtime/src` to pull a baseline
  count and it clobbered my own working tree (staged the whole subtree as
  modified) instead of doing a safe read. Caught immediately via `git
  status`/`git diff --stat`; nothing was committed, `git reset --hard
  <branch-tip>` restored cleanly — but it's exactly the
  check-git-status-before-anything-destructive-shaped discipline I should
  have front-loaded. Safer pattern used for the rest of the pass: `git show
  <ref>:<path>` for read-only baseline comparisons, never a path-scoped
  checkout against a ref I'm not sitting on.
- **held:** treated every "supersedes" claim as a re-verify trigger, not a
  trust signal, across all three landed SHAs — re-checked merge-base
  currency each time, re-ran both flagged `ken-cli` cross-crate gates at
  each SHA rather than assuming they still passed, and reproduced the final
  "comment-only" claim structurally instead of taking a grep result on
  faith. One live mutation-proof confirmed a real discriminator rather than
  a proxy.
- **carry:** when the posting channel is down, a *relayed* attestation
  reads as independent corroboration but is one seat's report — the
  relaying party should quote the exact intended block verbatim rather than
  select/summarize from the pane (a relay-selection mishap this slice: an
  earlier short ack got relayed instead of the actual bound verdict, and it
  needed a correction round-trip). Better still — per the Steward's and
  adversary's resolution here — the reviewed seat commits its own record
  directly to its home branch when its posting channel is down, removing
  the choice entirely rather than routing it through a third party's
  judgment call.
