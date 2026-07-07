# NC14 prerequisite - Checked-Core Data and Match View

**Owner:** Language-led, with Runtime review and Kernel boundary review.
**Branch:** `wp/NC14-checked-core-data-match-view`. **Size:** M.
**Risk:** high.

## Objective

Expose the minimal package-authoritative checked-core body/data view that
Runtime needs before opening NC14 data-constructor and pattern-match lowering.

NC14 must lower constructor values and supported match/eliminator structure from
checked-core package facts. On landed `origin/main @ b0738a72`, the NC13
checked-core body-view seam is intentionally narrower: it covers variables,
direct declaration calls, lambdas, applications, and non-recursive lets, and it
rejects constructor and eliminator tags as `unsupported_checked_body_shape`.
Runtime must not reopen raw source authority or parse unowned canonical bytes to
fill that gap.

This prerequisite extends the Language-owned checked-core view surface just
enough for Runtime to consume constructor references and supported match
structure from `CheckedCorePackage v0`.

## Scope

In scope:

- package-derived structured views for constructor references in checked-core
  bodies;
- package-derived structured views for the supported match/eliminator shapes
  needed by NC14;
- constructor identity facts tied to the package's stable constructor symbols,
  parent family, arity, target-index count, and lowerability metadata;
- branch data sufficient for Runtime to bind supported constructor payloads and
  select supported branches later;
- stable fail-closed lanes for stale constructor identity, missing required
  branch data, unsupported dependent motives, unsupported proof-only matches,
  unsupported eliminator shapes, and impossible-branch misuse when package
  facts do not justify the shape;
- focused Language tests that derive the new view from emitted
  `CheckedCorePackage v0` contents, not raw source;
- a Runtime-facing handoff note naming the exact API/types and lane names NC14
  may consume.

Out of scope:

- runtime-IR lowering of constructors or matches;
- runtime-IR evaluator changes;
- Cranelift, native, object, linker, or backend broadening;
- kernel admission, positivity, or coverage rule changes;
- compiler-driver target-selection widening;
- raw-source inspection after checked-core emission;
- broad dependent-motive compilation or proof-only match execution;
- NC8/NC9 certificate coverage claims for context-sensitive match forms.

## Deliverables

- A Language-owned checked-core data/match view surface callable by later
  compiler stages.
- Positive fixtures for package-derived constructor references and at least one
  supported match/eliminator body shape.
- Negative fixtures for stale constructor identity, missing branch data,
  unsupported dependent motive, unsupported proof-only match, unsupported
  eliminator shape, and impossible-branch misuse without package evidence.
- A short Runtime handoff section in this WP naming the exact view functions,
  data types, and stable error lanes.
- If the package contract needs a normative pointer for this new view, a narrow
  update to `spec/40-runtime/46-checked-core-package.md`.

## Acceptance

- The view is derived from validated `CheckedCorePackage v0` contents and the
  selected target closure, not raw source or a hand-maintained side table.
- Constructor identity is package-bound: a consumer cannot confuse
  constructors across packages, families, or stale symbols.
- Supported constructor and match/eliminator views expose enough structured
  information for NC14 Runtime lowering to bind payloads and select branches in
  a later WP.
- Unsupported dependent motives and proof-only match shapes reject before
  Runtime/backend work with stable lanes.
- Impossible-branch information is exposed only when package-side checked-core
  facts justify it. This prerequisite must not invent a contradiction,
  arbitrary trap, or host fallback.
- Existing NC13 body-view behavior remains intact for variables, direct calls,
  lambdas, applications, and lets.
- The diff does not change kernel admission, trusted-base accounting, runtime
  IR schema, evaluator behavior, backend/native code, or compiler-driver target
  selection.

## Guardrails

- Do not decode raw surface source or use source text as semantic evidence after
  checked-core emission.
- Do not make Runtime reconstruct unchecked terms from arbitrary bytes unless
  the reconstruction is tied back to canonical package semantic content and
  exact package identity.
- Do not weaken kernel coverage, positivity, or impossible-branch authority.
- Do not claim runtime lowering, evaluator agreement, Cranelift support, native
  support, NC8 certificate coverage, or NC9 proof-erasure validation from this
  prerequisite.
- Keep WP/campaign identifiers out of durable non-test implementation names and
  public report strings.

## D0 handoff

Runtime D0 for NC14 (`evt_2dsnrs3cwcwa3`) stopped before D1 because the
Runtime IR/evaluator shapes already exist, but no landed package-side
constructor/match body-view surface exists.

Language D0 (`evt_6c783vvdwe0r3`) agreed: there is no equivalent already-landed
surface, and the missing seam is Language-owned.

Kernel D0 (`evt_b3t0ezq5j4hk`) agreed: no kernel prerequisite implementation is
visible, and Kernel review for NC14 remains a semantic-cut review. Runtime may
trap or reject an impossible branch only when exact package-side checked-core
facts make that branch impossible on the exact artifact. NC14 must not weaken
kernel coverage/exhaustiveness or turn context-sensitive match shapes into
arbitrary host behavior.

Language should confirm the exact scope, decide whether the spec contract needs
a narrow pointer update, and then implement the minimal checked-core package
view satisfying the acceptance criteria. Runtime remains parked on NC14 proper
until this prerequisite lands or Language identifies an equivalent landed
surface. Kernel boundary review remains required before NC14 proper uses
impossible-branch facts for trap lowering.

## Runtime handoff

The prerequisite extends the existing `ken_elaborator::checked_core` body-view
API. Runtime should consume the same package-authoritative entry points it
already uses for expression bodies:

- `checked_core_body_view_for_selection(package, selection)`
- `checked_core_declaration_body_view(package, selection, symbol)`
- `CheckedCoreBodyTerm::ConstructorReference(CheckedCoreConstructorView)`
- `CheckedCoreBodyTerm::Match(CheckedCoreMatchView)`
- `CheckedCoreMatchBranchView`

`CheckedCoreConstructorView` carries the package-bound constructor symbol,
parent family symbol, level arguments, family parameter/index counts,
constructor arity, target-index count, recursive positions, constructor
lowerability, and family lowerability. Constructor applications are still
represented by ordinary `Application` nodes whose callee is a constructor
reference, so payload order remains the canonical checked-core application
order.

`CheckedCoreMatchView` carries the family symbol, level arguments, canonical
parameter bytes, canonical motive bytes, canonical index bytes, decoded
scrutinee, and constructor-ordered branch views. Each branch pairs the
constructor metadata above with the decoded method body.

Stable body-view error lanes Runtime may preserve in reports:

- `stale_constructor_identity`
- `missing_match_branch_data`
- `unsupported_dependent_motive`
- `unsupported_proof_only_match`
- `unsupported_eliminator_shape`
- `unjustified_impossible_branch`

The view remains a pre-lowering package surface. Current expression lowering
still rejects constructor and match body terms with
`constructor_lowering_unsupported` and `match_lowering_unsupported` until the
Runtime-owned NC14 lowering WP consumes this seam deliberately.
