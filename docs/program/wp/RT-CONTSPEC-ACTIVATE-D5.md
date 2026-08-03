# RT-CONTSPEC-ACTIVATE D5 — final population disposition

The before/after disposition for `RT-CONTSPEC-ACTIVATE`, recorded against exact
objects. Authority: Architect `evt_6bf2mmehjzy3k`, Steward recut
`evt_7e03zc2rtbm2q`, leader release `evt_6bt0xrbj2raac`.

## Exact objects

| role | SHA |
|---|---|
| base (merge-base with `main`) | `20162242c1830743df4270487caf2f75b54bcaa3` |
| candidate | `0c3c0476497a1da8bb4be744aab4dc08dfd20c9e` |

The base is the merge-base, not merely the branch point: `git merge-base
HEAD origin/main` and `git rev-parse origin/main` both return `20162242`, so
nothing landed on `main` between the cut and this measurement that the
comparison below would miss.

## The two populations are different fixtures and are never interchanged

The earlier frame wrote `AC-1`'s `2/2/2/2` as though it also described the seam
the emission gate runs on. It does not. Both numbers below are measured; neither
is derived from the other, and neither may be reported as the other.

### D5a / AC-1a — nested planner census

Planned and emittable continuation population on the nested planner fixture.

| | base | candidate |
|---|---|---|
| planned specializations | 2 | 2 |
| planned causal calls | 2 | 2 |
| emittable | 2/2 | 2/2 |

Asserted by `contspec_planner_closes_ordered_keys_units_and_causal_edges_dormantly`
(in `cranelift_backend::planning::static_transition::tests`), which is present
and passing in **both** pass sets below. This is the
no-plan-change control: the numbers are identical because the node adds no
planner population.

### D5b / AC-1b — executable lowering witness

The population that actually reaches `claim_and_call_continuation`. This is a
**different program** from the census fixture above.

| | base | candidate |
|---|---|---|
| planned | (gate absent) | 1 |
| resolved | (gate absent) | 1 |
| declared | (gate absent) | 1 |
| emitted | (gate absent) | 1 |
| causal count per identity | (gate absent) | exactly 1 |

Measured directly, not inferred: `resolved entries=1 distinct FuncIds=1
distinct specializations=1 planned=1 units=1`.

The four sets are compared **as sets, not as lengths**, in
`ContinuationClaimLedger::close`. Two sets of equal size can differ, so a length
comparison would pass for a population that swapped one token for another. The
base column reads "gate absent" rather than a number because the equality
mechanism is what this node adds; there is no earlier reading to compare it to,
and stating one would be a fabricated baseline.

A one-token emission population is sufficient and non-vacuous for a universal
per-identity routing property. It is not inflated to two by borrowing the
census's counts.

## AC-5 — aggregate pass/fail-set comparison

Compared as **name sets**, not as counts, so a test that disappeared and a test
that was added cannot cancel out in a total.

| | base | candidate |
|---|---|---|
| passing test names | 651 | 657 |
| failed | 0 | 0 |
| ignored | 1 | 1 |

Per-target: base `611 + 26 + 14`, candidate `617 + 26 + 14`.

**Lost (passing on base, not on candidate): none.** The set difference is empty,
so no test regressed, was renamed away, or was silently dropped.

**Gained (6), each named:**

- `ac1b_the_executable_lowering_witness_closes_its_one_token_population`
- `d4_substituting_the_emitted_funcref_reds_the_emission_equality`
- `d4_an_unrecorded_continuation_emission_reds_the_clif_sweep`
- `d4_failing_to_accumulate_emissions_reds_the_closeout_set_equality`
- `d4_claiming_the_same_causal_token_twice_reds_the_ledger`
- `d4_claiming_under_a_unit_that_does_not_own_the_token_reds`

All under `cranelift_backend::lowering::core::tests::control`.

## Frozen-surface evidence

Blob identity, base against candidate — equal object IDs, not a diff summary.

| surface | OID (both) |
|---|---|
| `planning/static_transition/abi.rs` | `23b9f5d778bf98fbb2907cf087bf06da30d82e7d` |
| `planning/static_transition/semantic_ir.rs` | `c5e0c9318c93a00c2320ac4dd27ba157f5c1a59a` |
| `boundary_value.rs` | `3dd70791d3d5631aa4bdd1ca0fe15b9f032ee9ae` |
| `boundary_value_clif.rs` | `ae774377b926102311f157d98a382833285675b4` |

## Scope audit

- **No planner/ABI population delta.** Grounded twice: AC-1a's census asserts
  identical counts on base and candidate, and every touched reference to
  `continuation_specializations` / `continuation_specialization_calls` in
  `static_transition.rs` is a **read** (`.iter().map(..)`) — the node adds
  read-only projection views and their accessors, never a construction site.
- **No fixture search.** `contspec_emission_witness()` is the identical
  expression that was already inline in the `AC-4` route census, moved so it can
  be named by both consumers. The census passes unchanged, by name. No case-body
  or source-spelling variants; hard stop 8 is spent and was not re-filed.
- **No boundary-transfer change.** `boundary_transfer_admissibility` is
  untouched; the six-shape negative result at `a84dbfba` stands.
- **No cross-function `FuncRef`.** Every ref is minted into the function that
  calls it. The `D4` substitution draws its distinct callable from that same
  function's own declarations and rejects loudly if there is none.
- **No durable or borrowed closure lane, no 0/0 witness.**

## What this node does NOT discharge

- **The behavioural target-dependence obligation.** That selecting the wrong
  same-shaped target changes or fails an observable result is
  `RT-CONTSPEC-WITNESS` `D7`/`AC-9`. Deferred, explicitly not discharged here,
  and merging `ACTIVATE` does not close it. Seam 4 is the first that runs an
  integrated native assembly, so it is the first that can execute it lawfully.
- **The same-shaped two-target redirect.** Its precondition — at least two
  distinct same-shaped targets in one lawful callable population — does not hold
  in this population, which is why `ACTIVATE`'s version of it was replaced
  rather than blocked. It lives wholly in `RT-CONTSPEC-WITNESS`.
- **Declaration-table construction.** The emission gate proves routing from the
  planner-issued identity to the emitted callee is exact. `bundle` remains the
  naming authority for which `FuncId` a specialization denotes, so a wrong
  forward declaration would move both sides together.
- **Which of the three closures crosses the boundary**, and whether the refusal
  precedes or follows the ruled direct continuation call.
  `boundary_transfer_admissibility` reports no origin, defining unit, child path
  or claim state. The Architect explicitly did not rule that no lawful observer
  can exist, and authorized no carrier.

## Two controls that were accepted while unable to fire

Recorded because it is a class, not two incidents, and because the next reader
should not have to rediscover it.

Both were committed `cfg(test)` mutation controls on this seam, both green, and
neither could perturb anything:

- the same-shaped redirect searched `continuation_calls` for a distinct
  same-shaped entry; this function holds exactly one, so it refused **before**
  the call seam — a red for the wrong reason, which is not a control result;
- the wrong-owner control read `find(|owner| owner != defining)
  .unwrap_or(defining)`; the single causal token's owner **is** the defining
  unit, so `find` returned `None` and the mutation silently became the
  **identity**. Green by vacuity since `457b9fc6`.

Both were found by running them, not by reading them. A mutation control that
cannot construct its perturbation must **fail loudly**; a fallback to the exact
value is a defect wearing the shape of defensive coding. Three outcomes must be
distinguished, never two: red at the seam, red before the seam, and green — and
green is a reach question before it is a defect.
