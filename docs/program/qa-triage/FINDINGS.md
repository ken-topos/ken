# Q2 findings — the rework tracks are ~10 tests, not ~110

**428 of 1905 tests triaged, 100% classified, six rings in parallel.**
Q2a scan: `scripts/qa-risk-scan.py`. Q2b triage: per-team results in this
directory.

## The distribution

| class | count | share |
|---|---:|---:|
| durable-invariant | 392 | **91.6%** |
| compat-vector | 19 | 4.4% |
| transition-sentinel | 7 | 1.6% |
| UNCLASSIFIABLE | 10 | 2.3% |

Confidence: 399 high / 26 low.

**The suite is in materially better shape than the queue implied.** That is
the advisory's own prediction holding: its scans are review queues, not
defect counts. A 428-row queue resolved to roughly **ten** tests worth
touching.

## What this does to Q3–Q7

| track | scoped against | survived triage |
|---|---|---|
| **Q3** derived counts / stateful names | 155 R1 + 108 R6 hits | **~3** |
| **Q4** broad outcome assertions | 147 R2 hits | **0 — the track is EMPTY** |
| **Q5** source-text tests | 25 R4 hits | **~6** |
| **Q6** ignored / placeholder | 3 R3 hits | **1** (one unlabelled sentinel) |
| **Q7** timing / fixtures | 27 R5 hits | **0 — the track is EMPTY** |

> ### ★ Q4 and Q7 are empty, and that is the most decision-relevant result
>
> **147 tests were flagged for asserting an outcome without naming the
> variant. Every one classified as a sound durable invariant.** Same for all
> 27 wall-clock/environment flags — one was explicitly identified as a
> scanner false positive (a large-stack thread spawn is not wall-clock
> coupling).
>
> Two of the five rework tracks were scoped from **hit counts**, and hit
> counts turned out to carry almost no signal about defects. **Authorizing
> Q3–Q7 from the scan totals would have committed the fleet to reworking
> ~300 tests that triage says are correct.**

## The residue, in full

**Actionable (≈10):**

- **1 unlabelled sentinel** — `ken-runtime/src/ir.rs:654
  seed_examples_are_observation_limited`. `examples.len()==5` over a
  *growable* seed list. Advisory §3.3: a sentinel is legal **only if
  labelled honestly**. This one is not labelled. Cheapest real fix in the
  set.
- **~6 source-text proxies (Q5)** — tests asserting over `include_str!`-ed
  Rust source as a stand-in for a mechanism: `eval.rs`
  resource-table lifetime, `effect_v1.rs` owner/close confinement, three
  `cat5_parsing_package.rs` rows, one `.ken.md` literate-source row.
- **~3 derived counts (Q3)** — `fact_count==23` duplicating a generated
  manifest; `fixtures.len()==1` freezing a representative corpus at one
  entry; `matches("-- helper --").count()==5` of unverified provenance.

**Not actionable, and confirmed so:**

- **All 19 compat-vectors are legitimate**, every one at high confidence:
  ABI byte layout, canonical export hashes, §3 grammar arity, kenfmt
  layout contract, wire projections. These *are* the contract.
- **6 of 7 sentinels are correctly labelled**, naming their retiring event
  (`[placeholder — reifies in V3]`, `#[ignore]` pointing at a named WP).
- **1 UNCLASSIFIABLE was a scanner artifact** — `output_dir`, see below.

## ⚠ Two defects in my own instruments, both found by others

**1. The scanner fabricated a test.** An unanchored `#[test]` regex matched
the attribute *mentioned in prose* — `rt_parity_native.rs:3` is a doc
comment reading ``//! Each case is its own `#[test]` ``. The phantom's body
ran to the next real test 480 lines later, swallowing every helper and
attributing their matches to `output_dir`, a tmp-dir function. **Found by
Team Foundation reading the source.** Not found by `--self-test`, which
passed throughout because it only checked files that *have* the patterns,
never one that would *invent* a row. A negative arm now exists.

**2. "Two counts agreed" was worthless corroboration.** The suite total was
documented as 1909, and I cited the scanner independently reproducing 1909
as evidence it was not dropping tests. **Both used the same naive `#[test]`
match, so both counted prose mentions and both were wrong.** A differential
oracle is blind to a premise its two sides share; agreement between two
instances of one method is an echo. True total: **1905**.

**3. The aggregator silently read the wrong file.** Ergo parsed at 23 rows
against a reported 71 — a glob matched both `ergo-leader`'s assembled result
and `ergo-qa`'s 24-row share, and took whichever came first. Caught only
because the leader had reported its own count, giving an independent oracle
to disagree with. Now anchored to the assembling worktree.

## Process notes worth keeping

- **Kernel stalled at 50/72 and reported nothing.** Its leader delegated the
  implementer's share, the message never reached that seat's turn, and the
  leader went idle believing it had handed off — the same transport failure
  that ate five of six Steward kickoffs, reproducing one level down inside a
  ring. **Silence and done are indistinguishable from outside.**
- **The correction sweep worked.** A wrong note behind a correct
  classification (two rows claiming a test "does not call `trusted_base()`"
  when both do) was found by sampling, broadcast, and swept by all six rings
  — including three that reported **zero** corrections, which is the answer
  that makes the other counts trustworthy.
