# `RT-CONTSRC-PRODUCER-LOCAL` `D4b` — admission closeout and activation

Deliverable record. Code on `wp/RT-DECL-CLOSURE-PORT-typed-units` over
checkpoint-approved `D3b` at `012a2c88`. Released by `evt_ssj6z1yd2r2t`.

## A retraction, first, because a later reader would otherwise build on it

`D3b`'s record carried **"0 of 60 consumer observations held a generated
emission owner"** and concluded that the generated-frame route could be proved
only by construction, not behaviourally. The Architect's ruling cited that figure
when it wrote that the boundary "explains the present evidence boundary".

**That figure is wrong and is withdrawn.** It was measured while the
capture-view defect was still live — every generated-context capture was
refusing before it reached the consumer — so the probe recorded **the breakage,
not the design**. I then carried the number forward as a standing fact into the
record and the handoff.

Re-measured on the repaired tree, `verify_entry_frame` takes the generated-frame
arm **30 times** against **58** predeclared, across ordinary lowering tests —
including
`nested_post_effect_checked_recursor_reaches_success_and_retains_exact_trap_provenance`,
which emits and **executes** a real object. The route was already live; only the
evidence was stale.

⛔ The lesson is the one that made it dangerous: a reachability count taken while
the path under test is failing measures the failure. It reads as a fact about the
design, it is self-consistent, and nothing about it looks provisional.

## The partition: `interned = V`, `declined = R`

`D4a` admitted the producer-local domain by **deleting** a filter. The claim it
left behind is that `R` is refused **upstream**, by the take-loop's own two
clauses and nothing else.

`d4b_admission_is_exactly_the_closed_required_vector` measures that over real
candidate edges as an **equivalence**, asserted in both directions per record: a
candidate is admitted **iff** every required position is closed and unambiguous.
An extra route modality, a special case, a corpus lookup, a closure-identity test
or a first-`Open` classification would each appear as a record where the two
sides disagree.

### The control caught a defect in itself before it caught anything else

The ledger's first form computed `admitted` from a predicate written **beside**
the take-loop. A mutation that installed an extra route modality in the *real*
loop then **survived** — the control was comparing the instrument with itself.

Restructured: the record is pushed as *not* admitted and flipped only where the
take-loop actually falls through. **Reaching that line is what "admitted"
means.** Against the corrected instrument both reachable mutations red at the
right record with the right diagnosis:

| mutation | record it reds at | left / right |
|---|---|---|
| first-`Open` classification | `[Closed, Closed, Open]` | `true` / `false` |
| ambiguity collapsed to first source | `[Closed, Closed, Ambiguous(2)]` | `true` / `false` |

⚠ The first mutation I tried — "an all-`Open` vector also admits" — **survived**,
for a harmless reason: no candidate in the corpus carries an all-`Open` vector,
so it is unreachable. A surviving mutation is not automatically a weak control,
but it is always a question, and here the question is what found the real defect
above.

⛔ **Both decline clauses are separately witnessed, and the declining shapes are
named rather than hoped for.** A row that witnessed only `Open` would leave the
ambiguity clause unmeasured, and both sides are asserted non-empty or the
equivalence is half-vacuous.

## The census, re-run with program fingerprint identity

Instrument: a temporary probe in `exact_continuation_source_environment`,
reverted before commit. `ken-runtime` lib, single-threaded, **820 raw records**.
The suite was **728 passed / 7 failed with the probe installed**, unchanged — so
the instrument does not perturb what it measures.

Identity is `program fingerprint + consumer owner + continuation origin +
producer construct origin`, where the fingerprint is
`source occurrences / function units`.

| | value |
|---|---|
| raw records | 820 |
| distinct edge identities | 58 |
| `(identity, vector)` instances | 61 |
| identities carrying two vectors | 3 |
| **admitted (`V`)** | **58** |
| **declined (`R`)** | **3** |

⚠ This identity is **coarser** than the prior census's, which also keyed on
recursive position and closure origin — neither is in scope at the probe point.
A coarser key merges rather than splits, so it **under**-counts identities: 58
here against 60 before, 61 instances against 66. That is the expected direction
and it does not affect the partition, which is computed per record.

### `R` is invariant, and this is the load-bearing half

`C`/`V` may move with fixtures. The three declined edges did not:

| fingerprint | consumer | continuation | construct | cause |
|---|---|---|---|---|
| `10/2` | `fn0` | `origin10` | `origin19` | `OPEN[ih-binder]` |
| `12/2` | `fn0` | `origin5` | `origin14` | `OPEN[let-value:Construct]` |
| `15/2` | `fn0` | `origin5` | `origin14` | `AMBIG2[let-value:If]` |

The first row's fingerprint, consumer, continuation and construct match the
prior census's IH edge **exactly**. Decline causes across the whole corpus are
**2 `OPEN` + 1 `AMBIG2`** — the three named causes, unchanged, with no fourth
cause and no edge declined for any other reason.

## Behavioural activation of the generated-frame route

Incidental traffic is not a control, so `d4b_the_generated_frame_consumer_runs_on_a_real_compile`
arms a counter over a real object compile, asserts the arm is **actually taken**,
and then displaces the claimed `ContinuationContextId` — which must red with the
identity-agreement refusal. That puts `D3b`'s three-sided revalidation on the
behavioural path rather than only where a planner control drives it.

⛔ The mutated run asserts the arm was reached **at least once**, not the same
count as the exact run. The refusal short-circuits the compile, so an equality
reds on the short-circuit rather than on the guard — the first draft asserted
equality and failed `1` vs `2`, which is the measurement saying so.

⛔ `d4b_displaced` can only displace an id that already exists. No test can mint
a `ContinuationContextId` from an integer and have it read as planner-issued.

## What is preserved

`D3b`'s two separately validated views, the nearest-exact-alias law, the
publication gate and every fail-closed boundary are untouched. No unit frame was
edited, nothing was padded or shifted, no caller tail was copied and no capture
was fabricated.

## Suite

`ken-runtime` lib: **730 passed / 7 failed / 1 ignored**. The 7 are the baseline
reds — the two standing `D0` reds plus the five `D4a` reds at their downstream
`Var: no runtime binding` boundary, which `RT-UNIT-CLOSURE-CONVERT` owns. Both
`check -p ken-runtime` and `check --profile test` are clean. The workspace build,
the `--locked` gate and conformance are CI's.

⭐ **The five `Var` reds did not block any `D4b` evidence.** The generated-frame
route is reached and exercised on fixtures that compile and run today, so no
hard stop is owed on the activation requirement.
