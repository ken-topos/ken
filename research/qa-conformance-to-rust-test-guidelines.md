# QA guidelines for elaborating conformance into durable Rust tests

**Status:** Research advisory, draft for operator review. Not merge-ready.

**Grounding:** `origin/main` at
`78ef39eb75506fa2261a9a287eaac40ea2f18d45` on 2026-07-18. The catalog
incident is grounded additionally against the failed PX8-N candidate
`9813aa0b6a5a406482793ea2441a66f0b7f943ca` and the local repair object
`c49a04cd9a633116f21277ef82adea20cb56327c`.

## Executive finding

The catalog failure was not evidence that exact assertions are bad. It exposed
a category error: a test treated an **evolving, derived census** as though it
were a **durable semantic invariant**.

`HostOpV1::ALL.len() == 22` can legitimately pin the closed, versioned V1 ABI
catalog. `native_tested_lanes() == NATIVE_TESTED_TARGETS_V1` legitimately pins
a consumer to its authoritative producer. But
`deferred_named_lanes().len() == 13` merely restated
`22 - NATIVE_TESTED_TARGETS_V1.len()` in a second crate, and the test name froze
that transient state as “nine native and thirteen unavailable.” When four
operations were deliberately promoted, the behavioral contract remained sound
but the mirrored census rotted.

The QA rule should therefore be:

> A Rust test derived from conformance must identify the kind of promise it is
> making, reach the production mechanism that enforces that promise, use an
> oracle independent enough to detect a defect, and assert the strongest stable
> observable. An intended extension should fail a test only when the test is an
> explicitly labelled transition sentinel.

The Rust suite already contains excellent examples of this discipline. It also
contains recurring weaker forms: broad rejection assertions, disabled tests,
source-text greps, wall-clock gates, and tests that verify a current inventory
rather than the relation that makes the inventory correct. The goal is not a
ban on any syntax. It is a decision framework for when each syntax is honest.

## 1. Review scope and method

The review surveyed every Rust file under `crates/**/*.rs` with repository-wide
pattern scans, then deep-read representative tests from all eight crates and
traced the catalog failure across its producer and consumer.

At the grounding commit the tree contains:

| Crate | `#[test]` functions |
|---|---:|
| `ken-elaborator` | 1,037 |
| `ken-runtime` | 222 |
| `ken-kernel` | 202 |
| `ken-interp` | 155 |
| `ken-cli` | 93 |
| `ken-host` | 45 |
| `ken-verify` | 23 |
| `ken-foundation` | 22 |
| **Total** | **1,799 across 226 Rust files** |

Lexical indicators found 178 literal `.len()` assertions, 41 direct
`.is_err()` assertions, 33 direct `.is_ok()` assertions, 22 tests that embed a
Rust source file with `include_str!`, three wall-clock or sleep sites, and three
`#[ignore]` tests. These are **review queues, not defect counts**. Context
decides whether each assertion is appropriate.

This distinction matters. Exact arity assertions in parser and kernel tests are
often the property. Exact hashes are right when canonical bytes are the public
contract. A broad `.is_err()` followed immediately by matching the precise
variant is not broad in substance. Conversely, a thousand passing tests at the
wrong seam do not cover the missing seam.

The review uses the repository's existing principles:

- conformance cases pin a spec section as input to expected behavior
  ([conformance README](../conformance/README.md));
- QA must verify the property rather than a representative case
  ([coordination law §7](../agent/COORDINATION.md)); and
- QA must trace absent clauses, emitted output, production registration, and
  adversarial mechanism probes
  ([build-QA playbook](../agent/playbooks/build/qa.md)).

## 2. Diagnosis of the catalog failure

### 2.1 What changed

On `origin/main`, `HostOpV1::ALL` contains 22 operations and
`NATIVE_TESTED_TARGETS_V1` contains nine. The PX8-N candidate correctly added
`FsReadAt`, `FsWriteAt`, `BufferAllocate`, and `BufferFreeze` to the
native-tested set, taking it to 13 while leaving the V1 catalog at 22
([`effect_v1.rs`](../crates/ken-host/src/effect_v1.rs)).

The independent Verify consumer imported the authoritative native set but also
asserted a second fact:

```rust
fn imported_catalog_has_exact_nine_native_and_thirteen_named_unavailable() {
    assert_eq!(HostOpV1::ALL.len(), 22);
    assert_eq!(native_tested_lanes(), ken_host::NATIVE_TESTED_TARGETS_V1);
    assert_eq!(deferred_named_lanes().len(), 13);
    // ...
}
```

That test therefore contained three different promise classes:

1. **Compatibility vector:** V1 has exactly 22 operation identities.
2. **Cross-crate invariant:** Verify's native projection equals the host-owned
   source of truth.
3. **Transient census:** the complement currently has 13 members.

Only the third needed editing for a legitimate promotion, yet its name made the
whole test read as a permanent contract
([`catalog.rs:109-121`](../crates/ken-verify/src/catalog.rs)).

### 2.2 What the enduring test should say

The durable property is a partition:

- `native == NATIVE_TESTED_TARGETS_V1`;
- `native union unavailable == HostOpV1::ALL`;
- `native intersection unavailable == empty`; and
- every unavailable operation reports `RepresentedUnavailable` and is absent
  from the native set.

The in-flight repair at `c49a04cd` expresses this relation and renames the test
`imported_catalog_partition_is_exact_and_closed`. The count it retains is
derived from `NATIVE_TESTED_TARGETS_V1.len()`, not repeated as a literal.

This version survives an intended promotion while still catching every
semantically relevant defect: a missing operation, a duplicate classification,
a stale availability tag, or a Verify projection that disagrees with the host
source of truth.

### 2.3 What the failure teaches

The full CI gate worked: it found a same-change consumer outside the targeted
Runtime packages. But the red was maintenance noise, not evidence of a
behavioral regression. The desired test would have stayed green for the
intentional promotion and gone red only if the partition ceased to be exact.

The root mistake happened when an initial milestone statement became an
unlabelled permanent test. The PX6 frame correctly pinned nine deferred lanes
for that delivery. Later WPs were expressly allowed to promote them. An
acceptance criterion about **this transition** is not automatically an invariant
about **all future states**.

## 3. Three promise classes

Every conformance-derived Rust test should declare one of these classes during
review.

### 3.1 Durable invariant

An invariant should survive all intended extensions that preserve the
contract. Examples include:

- a catalog partition is disjoint and exhaustive;
- a pure term has no reachable effect node;
- a rejected program returns a specific diagnostic class;
- interpreter and native observations are equal; and
- an untrusted positive verdict has passed the kernel check.

Prefer relations, set equality, typed variants, and exhaustive matches.

### 3.2 Normative compatibility vector

A compatibility vector pins exact bytes or values because those exact values
are the contract. Examples include:

- ABI operation identities and schema field order;
- canonical serialization bytes and content hashes;
- FNV known-answer vectors; and
- a language grammar's exact constructor arity.

These tests are intentionally exact. Changing one requires an explicit version
or contract decision, not merely updating a snapshot.

### 3.3 Transition sentinel

A transition sentinel intentionally fails when a planned extension happens so
that someone must review the boundary. This can be useful, but it must be
labelled honestly:

- name it for the transition or frozen boundary, not the current count;
- state why ordinary extension must stop here;
- name the event that retires or updates it;
- keep it beside the authoritative owner where possible; and
- enumerate every known consumer in the change's blast-radius checklist.

`catalog_package_roots_use_only_the_controlled_sections` is close to this shape:
it validates the allowlist, then separately pins the currently populated
section set so the first package in a reserved section forces review
([`catalog_taxonomy.rs`](../crates/ken-elaborator/tests/catalog_taxonomy.rs)).
That is defensible only because the assertion explains the intentional stop. It
is not a model for ordinary evolving inventories.

If a reviewer cannot classify a test, the test is not ready.

## 4. What the current suite does well

### 4.1 Non-degenerate discriminator pairs

`b1_exact_denotation_alphabet.rs` compares a real reachable
`FsHandleMetadata` perform node with a program that declares identical effect
headroom but performs no metadata operation. The former exports the operation;
the latter rejects. This isolates the semantic axis that once failed: declared
headroom cannot manufacture denotation evidence
([exact-alphabet test][exact-alpha]).

Kernel positivity similarly includes the obvious negative occurrence and an
occurrence hidden inside an application, rather than treating one direct arrow
case as the whole property
([`inductive.rs`](../crates/ken-kernel/src/inductive.rs)).

### 4.2 Independent oracles

The bignum acceptance suite uses precomputed mathematical vectors and avoids
using the production arithmetic operation to calculate its own expected value
([`f1_bignum_acceptance.rs`](../crates/ken-interp/tests/f1_bignum_acceptance.rs)).
The Foundation FNV tests use published known-answer values rather than merely
asserting two calls agree
([`hash.rs`](../crates/ken-foundation/src/hash.rs)).

### 4.3 Cross-layer closure and mutation

The host manifest test compares producer, registry, observer, and consumer sets
for equality, then mutates each set independently and requires rejection. This
is stronger than four matching counts
([`effect_v1.rs:2490-2570`](../crates/ken-host/src/effect_v1.rs)).

The Cranelift test imports the authoritative native operation set and mutates
each layout field through the production verifier
([`cranelift_backend.rs:7126-7208`](../crates/ken-runtime/src/cranelift_backend.rs)).
The operation inventory is coupled by equality; layout discrimination is shown
by opposite outcomes.

### 4.4 Real production reachability

CLI tests build and execute admitted Ken source and assert process status,
bytes, and absence of partial artifacts. For example, the authority-mismatch
test asserts the exact structured admission error and that the output directory
remains empty, while the `Vis` test executes a linked artifact through the real
host dispatch
([`px4b_native_production.rs`](../crates/ken-cli/tests/px4b_native_production.rs)).

These are valuable because a hand-built runtime expression or direct helper
call cannot prove that checked source reaches the same mechanism.

## 5. Risk patterns found in the current suite

These are not all bugs. They are the places where QA judgment is required.

### 5.1 Derived counts and stateful names

There are 178 literal `.len()` assertions. Most pin local fixture shape,
constructor arity, or an exact schema and are appropriate. The catalog failure
was different because its count was derivable from an evolving authoritative
set in another crate.

Test names should state the invariant, not today's census. A search for names
embedding “nine” or “thirteen” found the failed catalog test as the only such
catalog-state name. That made the maintenance smell especially visible.

### 5.2 Broad outcome assertions

The scan found 41 direct `.is_err()` and 33 `.is_ok()` assertions. Many are
followed by exact variant checks or merely form the positive half of a pair.
Those are fine. A negative conformance case that stops at “some error” is not.

One concrete weak assertion is:

```rust
matches!(err, Err(ElabError::KernelRejected { .. }) | Err(_))
```

The `Err(_)` alternative subsumes the named alternative, so the test cannot
distinguish the promised kernel rejection from any unrelated error
([W-style test][wstyle]).

For a conformance negative, assert the exact enum variant and all fields that
identify the rule. Text is supplementary diagnostic quality, not the primary
oracle.

### 5.3 Disabled and placeholder tests

Three `#[ignore]` tests remain in `l1_acceptance.rs`. Two only assert future
elaboration succeeds; one has an empty body. Their comments honestly name the
deferral, but an ignored test provides no regression protection and its trigger
can quietly land without reification
([`l1_acceptance.rs:236-340`](../crates/ken-interp/tests/l1_acceptance.rs)).

The durable choices are:

- omit a not-yet-runnable Rust test and track the conformance case as debt; or
- keep a local placeholder marker with a concrete reification trigger and make
  QA block once that trigger has landed.

An ignored test must never be counted as coverage.

### 5.4 Source-text tests

Twenty-two sites embed Rust source files. Several protect real architectural
absence properties, such as “a test helper is not public production API” or
“backend types do not escape the host boundary.” Those properties matter, but
plain string matching is sensitive to formatting, comments, aliases, and
spelling.

`naked_process_ir_helpers_are_not_public_production_api` matches an exact
newline and function signature. Reformatting can fail it without changing
visibility; a semantically equivalent spelling can evade it
([native-production test](../crates/ken-cli/tests/px4b_native_production.rs)).

Use a compiler visibility check, type-level construction failure, AST/token
inspection, or a dedicated lint when possible. If raw source inspection is the
only available outer-ring net, scan the mechanism rather than a bare name,
explain the limitation, and add a mutation that demonstrates the scan bites.

### 5.5 Wall-clock and environment coupling

The interpreter performance regression test uses a two-second budget. It is
well motivated by a prior greater-than-60-second blowup and also checks the
result value, but wall-clock thresholds remain machine-load sensitive
([performance test][perf]).

Prefer structural work counters or complexity bounds when they can observe the
defect. Keep wall-clock tests in a performance lane, use wide separation between
good and bad behavior, and never let timing be the only correctness oracle.

Temporary directories and current-directory changes likewise need unique
per-test ownership and cleanup. Fixed shared temporary names can couple
parallel runs or preserve stale artifacts.

### 5.6 Test volume as a false confidence signal

Fifty-eight percent of the Rust tests are in `ken-elaborator`. That density is
reasonable for Ken's large surface-to-core boundary, but it means raw test count
is especially misleading. The catalog incident crossed from `ken-host` into
`ken-verify`; targeted Runtime and CLI tests could not see that consumer. The
required response is not a local workspace run. It is an explicit consumer
closure review, targeted affected-package tests, and the existing full CI net.

## 6. The conformance-to-test workflow

QA should require this workflow for every conformance case or WP acceptance
criterion.

### Step 1: restate the proposition

Write one sentence that can be true or false independently of the test code.
Do not start with a function name or expected fixture count.

Good:

> The native and unavailable classifications form a disjoint, exhaustive
> partition of the closed host operation catalog.

Weak:

> There are thirteen unavailable operations.

Record the exact spec or conformance case that owns the proposition. If the
source only describes the current milestone, classify it as a transition
sentinel rather than silently promoting it to an invariant.

### Step 2: identify the production seam

Name the producer, the consumer, and the call path the test will execute. Ask:

> If the new mechanism were deleted, would this exact test still pass?

If yes, the test is hand-feeding or testing a proxy. A helper-level unit test can
supplement the suite, but at least one case must derive its evidence through the
real registration, elaboration, lowering, dispatch, or verification path.

### Step 3: choose an independent oracle

Rank candidate oracles in this order:

1. a separately specified or separately implemented semantic oracle;
2. a typed structural value at the boundary under test;
3. a normative known-answer byte or hash vector;
4. a relation among independently produced values;
5. diagnostic text or source spelling only as a last resort.

Using a production constant as expected data is appropriate when the property
is **consumer equals authoritative producer**. It is circular when the same
constant supplies both the test input and the expected result without an
independent transformation or relation.

Round trips prove self-consistency, not truth. Pair them with an exact structural
wire assertion or independent vector.

### Step 4: design the discriminator before the positive case

For every gate or classification boundary, build a pair that differs on one
load-bearing axis:

- accepted versus rejected;
- present perform node versus declared headroom only;
- native versus unavailable;
- valid field versus one mutated field;
- kernel-checked positive constructor versus bypass attempt; or
- interpreter oracle versus deliberately changed native observation.

For a mechanism with several guards or fields, vary each independently. A
single large malformed fixture cannot show which checks are live.

### Step 5: choose the narrowest stable assertion

Use this hierarchy:

1. compile-time exhaustiveness over a sealed enum, where possible;
2. exact typed value or error variant with load-bearing fields;
3. set equality, disjointness, coverage, ordering, or another relation;
4. exact canonical bytes or hashes when those bytes are normative;
5. a predicate such as `.is_err()` only when the error class is irrelevant; and
6. diagnostic strings or raw source layout only when no structural observable
   exists.

Assert absence as well as presence when omission is the danger.

### Step 6: classify every literal

For each count, string, hash, timeout, or path in the assertion, ask:

- Is this value specified externally as part of the contract?
- Is it a property of this deliberately fixed fixture?
- Is it derived from another authoritative value?
- Is it merely today's repository state?

Contract and fixture literals are valid. Derived values should be calculated or
compared relationally. Repository-state literals belong only in labelled
transition sentinels.

### Step 7: enumerate consumer closure

When a closed set, enum, schema, status, or operation inventory changes, search
for consumers of the **element and its obligation**, not just its name. For each
element, identify:

- producer or registry;
- serializer and parser;
- interpreter and native consumers;
- verification or differential consumer;
- public docs or generated manifest; and
- conformance and Rust tests.

Prefer one authoritative enumeration imported by consumers. Where independence
requires a separate projection, compare complete sets and mutate omissions;
never mirror only their counts.

### Step 8: prove the test can fail for the intended reason

Before approval, use a scratch mutation, prior-commit run, or test-only selector
to break the production mechanism at the claimed seam. The unchanged test must
fail with the expected opposite observation.

Do not keep the mutation. Record what was changed and which assertion caught it.
For an old-to-new capability claim, run the literal assertion against the prior
commit when practical.

### Step 9: validate maintenance behavior

Ask two counterfactuals:

1. Which legitimate future change should leave this test green?
2. Which incompatible change must make it red?

If both answers are “any change,” the test is probably a snapshot or sentinel,
not an invariant. Label it accordingly.

### Step 10: run the right gates

Run the named test and affected packages through `scripts/ken-cargo`, verify the
test count is nonzero, and inspect the failure message. Never run the workspace
locally. Full-workspace consumer surprises remain CI's job, while QA's job is to
make those surprises semantic rather than incidental.

## 7. Rust assertion patterns

### 7.1 Evolving partitions

```rust
let all = HostOpV1::ALL.into_iter().collect::<BTreeSet<_>>();
let native = native_tested_lanes().into_iter().collect::<BTreeSet<_>>();
let unavailable = deferred_named_lanes().into_iter().collect::<BTreeSet<_>>();
let expected_native = NATIVE_TESTED_TARGETS_V1
    .into_iter()
    .collect::<BTreeSet<_>>();

assert_eq!(native, expected_native);
assert_eq!(native.union(&unavailable).copied().collect(), all);
assert!(native.is_disjoint(&unavailable));
```

Name this for the partition invariant, not for its current sizes.

### 7.2 Negative conformance

```rust
let error = run_case(input).expect_err("case must reject");
assert!(matches!(
    error,
    Error::EffectEscapes { ref effect, ref witness }
        if effect == "FS" && witness == expected_site
));
```

Do not append `| Err(_)` or reduce a promised diagnostic to `.is_err()`.

### 7.3 One-axis discriminator

```rust
let baseline = compile_and_observe(real_perform_source());
let control = compile_and_observe(headroom_only_source());

assert_eq!(baseline, Expected::Operation("FsHandleMetadata"));
assert_eq!(control, Expected::Rejected(MissingReachableOperation));
```

The fixtures should differ only in the perform node whose reachability is under
test.

### 7.4 Canonical bytes

```rust
assert_eq!(encode(value), independently_pinned_bytes);
assert_eq!(decode(independently_pinned_bytes), value);
```

The first checks truth of the encoding. The second checks decoding. A round trip
using only production encode and decode is insufficient by itself.

## 8. QA review record

For each conformance-derived test, QA should be able to fill this compact
record:

| Field | Required answer |
|---|---|
| Source | Exact spec section and conformance case |
| Promise class | Invariant, compatibility vector, or transition sentinel |
| Proposition | One implementation-independent sentence |
| Production seam | Producer → transformation → consumer |
| Oracle | Why it is independent enough to catch the defect |
| Discriminator | The one-axis opposite case or mutation |
| Observable | Exact typed value, relation, bytes, or error fields |
| Completeness | Guards, variants, and consumers enumerated |
| Maintenance | Intended green changes and required red changes |
| Execution | Named test, affected packages, nonzero count |

A missing answer is a Block, not a request to compensate with more happy-path
tests.

## 9. Proposed hard gates for the QA role

The existing QA playbook contains strong lessons, but they are distributed
across many incident-specific bullets. The following compact gates should sit
at the front of the conformance-to-test workflow:

1. **Traceability:** every test names its spec/conformance source and promise
   class.
2. **Reachability:** at least one test reaches the real production mechanism;
   hand-fed helper tests do not satisfy this gate.
3. **Discrimination:** every boundary has an opposite-observable pair; every
   load-bearing guard or field is independently exercised.
4. **Oracle independence:** expected values are not produced by the same logic
   under test, except when equality to an authoritative owner is the property.
5. **Assertion stability:** assertions use typed structure or relations; counts
   and strings are justified by contract class.
6. **Completeness:** sealed enumerations are exhaustive by construction, and
   cross-crate consumer closure is explicitly reviewed.
7. **No phantom coverage:** ignored, empty, zero-count, placeholder, and
   success-only tests are not counted as conformance.
8. **Causality:** QA demonstrates that breaking the claimed mechanism makes the
   unchanged test fail.
9. **Maintenance:** the test states which intended extensions remain green and
   which incompatible changes become red.
10. **Targeted execution:** affected tests and packages run locally; full
    workspace remains CI-owned.

These gates are deliberately about judgment, not syntax. They permit exact
counts, snapshots, source scans, time budgets, and shared constants when those
forms are the honest observable. They reject the same forms when they merely
encode incidental state.

## 10. Recommended follow-up audit queue

This report does not authorize repairs, but the following sites deserve bounded
follow-up:

1. Land the relational `ken-verify` catalog partition repair with the PX8-N
   change; do not replace `13` with `9`.
2. Strengthen the subsuming `Err(_)` assertion in
   `dependent_match_wstyle_acceptance.rs` to the promised structured error.
3. Reconcile or remove the three ignored L1 tests; confirm whether each named
   trigger has already landed.
4. Review the 22 Rust-source embedding sites. Replace exact whitespace and
   spelling checks with compiler, token, AST, or dedicated-lint checks where
   possible.
5. Label current-inventory tests such as the populated catalog section set as
   transition sentinels, including their retirement/update trigger.
6. Move wall-clock-only performance claims toward structural work counters when
   instrumentation can express the regression directly.

The point is not a one-time cleanup. It is to prevent the next conformance case
from becoming another fossilized census.

## Conclusion

Good QA does not translate a conformance bullet into the most literal possible
assertion. It identifies the enduring proposition behind the bullet, reaches
the production seam, chooses an independent oracle, constructs the case that
would falsify the proposition, and asserts at the narrowest stable boundary.

The catalog incident is a useful discriminator for the guidelines themselves:

- “there are thirteen unavailable operations” is a milestone census;
- “the two classifications are exact, disjoint, exhaustive, and agree with the
  authoritative native set” is the durable contract.

QA should encode the second unless it explicitly intends to stop the first
future promotion.

[exact-alpha]: ../crates/ken-elaborator/tests/b1_exact_denotation_alphabet.rs
[wstyle]: ../crates/ken-elaborator/tests/dependent_match_wstyle_acceptance.rs
[perf]: ../crates/ken-interp/tests/rtp1_elim_reduce_ih_perf_acceptance.rs
