# `RT-CONTSRC-PRODUCER-LOCAL` `D2` — both binding kinds, populated

**What landed:** the `ProducerLocal` arm `D1` represented is now **constructed**
by the planner, for both binding kinds, with a derived contract. ⛔ Nothing is
admitted: every candidate whose required environment names one still declines,
so no edge's verdict moves.

## The two kinds, and where each is minted

Both are minted at the site that **introduces an environment entry**, which is
also the only site that can supply a locator.

| kind | minted at | `binding_origin` | `binding_ordinal` | `locator.environment_origin` | `locator.environment_index` |
|---|---|---|---|---|---|
| host-effect result | the `Let` whose bound value is an `Effect` | the `Effect` occurrence | `0` | the `Let`'s **body** | `0` |
| `Match` case binder | the `Match` / `ComputationalMatch` case descent | the **case body** occurrence | the binder's index | the same case body | the binder's index |

**The two origins differ for the effect result and coincide for the case
binder, and that is the separation `D1` exists to express.** An `Effect`
*creates* the value; the `Let` body is the scope that *holds* it. A case binder
is introduced by, and held in, one scope.

⭐ **Why the effect result is minted at the `Let` and not in the `Effect` arm.**
A locator names a position in an environment. An effect result consumed without
a binder never enters an environment, so there is no position to name — and it
correctly stays `Open`. Minting in the `Effect` arm would have to invent a
locator for a value that has no home.

⛔ **The case binder's identity is the case body plus an ordinal, never the
match occurrence plus an encoded pair.** Packing a case index and a binder index
into one `u32` is the aliasing this plane refuses everywhere else, and it is
unnecessary: the case body is already the exact static identity of that binder's
scope, so two binders in different cases of one match differ by origin and two
in the same case differ by ordinal.

## The contract, fact by fact

One function derives both kinds. Only the carrier and the referent lifetime
differ per kind; ownership and storage owner are read off `AbiCarrier`'s
existing methods — the same two the entry plane's `abi::slot` reads — so this
record **cannot disagree with the entry plane** about what a carrier implies.

| fact | host-effect result | `Match` case binder |
|---|---|---|
| carrier | `abi::result_carrier` on the `Effect` shape | ⚠ `ValueWord`, derived here |
| ownership | `AbiCarrier::ownership` | `AbiCarrier::ownership` |
| storage owner | `AbiCarrier::storage_owner` | `AbiCarrier::storage_owner` |
| referent affinity | the `Effect` occurrence's own lifetime authority | the scrutinee child's lifetime authority |

⚠ **The one fact no prior authority stated, flagged rather than buried.** A case
binder is not an occurrence's *result*, so `result_carrier` — whose contract is
"the carrier an occurrence's result travels in" — does not answer for it, and
nothing else did either. `ValueWord` is this plane's carrier for an ordinary
in-body Ken value: it is what `result_carrier` assigns every expression shape
that is not `Trap` or `ImportedDeclarationRef`, and a binder is exactly that.
**This is a `D2` derivation, not a pre-existing reading**, and it is the single
item in this deliverable that wants the Architect's eye.

⭐ **The binder's referent lifetime is not conservatively floored, because it
does not have to be.** `PlannedReferentLifetime::Persistent` is issued only when
the complete source result is closed over persistent children, so a field of a
persistent scrutinee is persistent *by that type's own definition*. Reading the
scrutinee's lifetime is a read, not a promotion — and the type documents that it
has no promotion operation.

The derivation is measured to be per-binding rather than stamped: on the control
fixture the case binder's scrutinee is persistent and the effect result is
activation-owned, so the two affinities **differ**. A single hardcoded affinity
— the easiest wrong implementation — reds that assertion.

## Represented, not admitted

`exact_continuation_source_environment` declines any candidate whose exact
inputs name a producer-local coordinate.

**This preserves the pre-`D2` population exactly.** Every position that is a
producer-local binding today was `Open` before `D2`, and `Open` already declined
at the take-loop immediately above. The decline moved from *opaque* to a *named
domain*; the outcome did not move at all. That is why the `D0` per-row parity
record and both lib baselines are unchanged.

⛔ **It is not an edge selector.** It tests the coordinate domain, is uniform
over every edge, and consults no corpus, closure identity, first-`Open` reason
or planned-member status — the four forbidden substitutes.

⛔ **It must sit before the validator, not after.** The refusal `D1` installed in
`validate_continuation_source_slot` is a planner **`Err`**, which rejects the
enclosing source program. Declining the *candidate* is the correct boundary, and
it is the one the take-loop already uses for `Open`.

**Promise class: transition sentinel.** `D4` releases admission and deletes this
block; that is the event that retires it.

## The gate is measured, and the first version of this evidence was not

`contsrc_d2_a_producer_local_environment_declines_the_candidate_not_the_program`
is proved to discriminate: with the gate replaced by `if false`, it fails, and
the failure is the `D1` validator refusal — which is the exact evidence that the
gate sits before the validator.

⚠ **Recorded because it nearly shipped as a vacuous row.** The first version of
this fixture put `unit()` in the case body. That case body then required *no*
surrounding environment, so `required_input_count` was zero, `exact_inputs` was
empty, and **no producer-local coordinate ever reached the gate**. Both rows were
green, and stayed green with the gate disabled. The fixture now uses `Var(3)` —
past the two computational binders, onto the enclosing `Match` binder — which is
what makes the required environment non-empty. The `Var` index is load-bearing
and is commented as such at the fixture.

## Baselines

Measured on this commit against the `D0` record at `179af863`, per row:

- `ken-cli` `rt_parity_native`: unchanged — `buffer_freeze` green, six red with
  byte-identical texts, including the five-`ComputationalMatch` / one-`Match`
  split.
- `ken-runtime` lib: the `D1` checkpoint's 721 passed / 2 failed, plus the two
  `D2` controls, with the same two standing reds.
- `ken-elaborator` lib: 108 passed / 0 failed.
- The walk change alone, measured before the two controls were written, was
  already 721 / 2 — so the behaviour-preservation claim is independent of the
  tests that assert it.
