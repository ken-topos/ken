# `CI-IGNORED-SWEEP` — frame

Owner: **verify**. Size: **S**. Gate: none. Depends on: nothing.
Origin record: [`CI-IGNORED-SWEEP`](../issues/CI-IGNORED-SWEEP.md)

Ground: `origin/main` **`368ff87e`**. Every line number and tool fact below was
read at that ref.

## 0. Posture

Nothing in the repo ever re-runs an ignored row. The suppressed population is
write-only: a row goes in, and no mechanism asks whether it still belongs.
When an owner node lands its repair, the `#[ignore]` persists — so the repair
ships with its own regression cover switched off.

**Treat every anchor as perishable. If a fixed input turns out false against the
landed code or the landed tooling, say so and escalate — do not quietly build
around it.**

**This node builds an instrument. The failure mode of an instrument is
reporting when it measured nothing**, which is the exact defect the two `px8f`
jobs already had. Section 3 is written against that, not against ordinary
correctness.

## 1. Fixed inputs

### 1a. The population is 50, and it is selected by attribute, not provenance

Anchored count at `b0a0a20c` — use `^[[:space:]]*#\[ignore`, since the
unanchored form also matches doc-comment lines that merely mention the
attribute and inflates every file:

```
total 50 — ken-cli 34, ken-verify 10, ken-runtime 3, ken-interp 3
```

The node's original `46` was a scope error, not a miscount: it summed the
42-row authorized set and the four pre-existing `px4b` ignores, counting the
population this program **authored** rather than the population the mechanism
will **select**. A sweep selects on the attribute.

### 1b. Only 44 of the 50 have ever been measured

The one hand-run, at `7d204438`:

```
ken-cli    --no-fail-fast -- --ignored   ->  0 passed / 34 failed
ken-verify --no-fail-fast -- --ignored   ->  0 passed / 10 failed
```

That covered `ken-cli` and `ken-verify` only. **The three `ken-runtime` and
three `ken-interp` ignores were never in it**, so the over-annotation question
is open for those six. Closing it is `D5`.

### 1c. Two independent classification axes, and rows can be in both

- **Axis 1 — reason for ignoring:** `base-debt-awaiting-repair` versus
  `ignored-by-policy`. Four rows are policy and will answer *"yes, still
  belongs"* forever:

  | row | class |
  |---|---|
  | `crates/ken-runtime/src/boundary_value_clif.rs:7473` | **COST** — `"~142s of arena work; the fast instance at depth 3000 runs by default"` |
  | `crates/ken-interp/tests/l1_acceptance.rs:242` | **UNBUILT CAPABILITY** |
  | `crates/ken-interp/tests/l1_acceptance.rs:284` | **UNBUILT CAPABILITY** |
  | `crates/ken-interp/tests/l1_acceptance.rs:334` | **UNBUILT CAPABILITY** |

- **Axis 2 — where the row dies:** some rows fail *upstream of their own
  property*, at an `expect` before any assertion runs.
  `RT-WORKER-FIXTURE-DECODE` and `RT-CARRIER-PRODUCER-OCCURRENCE` are both this
  shape. For them "still failing" is not the useful bit; "can the fixture even
  execute" is.

The axes are independent — a row can be base debt *and* die before its
assertion.

### 1d. The cost row is not merely noise, it is a budget

`boundary_value_clif.rs:7473` costs about 142 seconds. **A sweep that re-runs
the whole ignored population pays that on every run, unbudgeted.** It must be
exempt from the first run, not after someone notices.

### 1e. The two venues use DIFFERENT tools, and this decides the shape

Measured at `368ff87e`:

- **CI runs nextest.** Main gate is `cargo nextest run --workspace --locked`
  (`.github/workflows/ci.yml:121`); the dedicated native jobs are at `:213`,
  `:262`, `:315`.
- **Locally, nextest is NOT INSTALLED.** `cargo nextest --version` reports
  `no such command: nextest`. Local agents run `scripts/ken-cargo test`, which
  wraps cargo/libtest under a machine-wide build lock.
- **There is no `.config/nextest.toml`** in the tree.

Consequences the deliverables must respect:

1. **The `1b` hand-run used libtest syntax (`-- --ignored`). It does not
   transfer to the CI job.** nextest selects ignored rows by its own flag.
   **Verify the exact spelling against the landed tool before pinning it in a
   workflow** — this frame deliberately does not name it, because I could not
   run nextest to check.
2. **The implementer cannot iterate on a nextest invocation locally.** Develop
   and validate the *selection logic* per-crate with `ken-cargo test -p <crate>`,
   and treat the workflow wiring as CI-verified only.
3. **`COORDINATION §12` forbids a local `--workspace` run.** The sweep is a CI
   job. Locally, per-crate only.

### 1f. Eight owner nodes are queued against this population

`RT-CARRIER-BYTESPAN-OBSERVE`, `RT-CARRIED-RESOURCE-SCALAR`,
`RT-CLOSURE-BOUNDARY-LANE`, `RT-COMPMATCH-TREE-SCRUTINEE`,
`RT-FRAME-MARKER-ONCE`, `RT-PROCESS-EXIT-STATUS`, `RT-WORKER-FIXTURE-DECODE`,
`RT-CARRIER-PRODUCER-OCCURRENCE`. Each will land a repair whose row nothing
currently un-ignores.

### 1g. It has already failed once, and luck caught it

`RT-SRCBODY-BIND-ORDER` `D11` ignored `px7o` on a false premise and **would have
switched off a working repair.** It was caught only because `D12` happened to
run an enumeration that included ignored rows. A normal verification run cannot
see this by construction: `D13`'s `120 passed / 0 failed / 34 ignored` is
*disjoint* from the population it suppresses.

## 2. Deliverables

**`D0` — pin the venue and confirm the tooling gap.**
Record the exact nextest invocation that selects only ignored rows, verified
against the landed tool in CI, and state plainly that the `1b` local syntax is
not it. If nextest cannot express the selection, say so — that is a finding,
and the fallback (a per-crate libtest job) is a re-sizing conversation, not a
longer turn.

**`D1` — build the policy-exemption carrier, and seed it.**

**Decided, do not relitigate:** the carrier is an **explicit checked-in
registry** of policy-exempt rows, keyed on **test path** (`crate::module::fn`),
not on `file:line`, which drifts.

Rationale, so the constraint is legible rather than aesthetic:

- **Do not parse the reason string for prose.** These four rows are
  distinguishable today only by wording a human chose. A sweep that greps for
  `"not yet in scope"` is one reword away from silently re-including a row.
- **Fail toward sweeping, never toward skipping.** An unregistered row gets
  swept. A base-debt row omitted from the registry therefore costs *noise*; a
  policy row omitted costs a missed regression. **Noise is self-correcting and
  a missed regression is the thing this node exists to prevent**, so the
  default must be "sweep it".
- Seed it with exactly the four rows in `1c`, including the cost row from `1d`.

**`D2` — the sweep job itself.** Non-blocking by construction: a row that
starts passing is good news needing routing, not a red gate. It must not become
a fourth way for an unrelated candidate to be blocked.

**`D3` — the positive control.** Un-ignore one known-failing row, observe the
sweep reports the change, restore it.

**`D4` — routing.** Name where a finding goes. The owner node id already in each
base-debt `#[ignore]` string is the natural address; say what happens for a row
whose id names no live node.

**`D5` — close the open over-annotation question for the six unmeasured rows**
(three `ken-runtime`, three `ken-interp` from `1b`). Run them per-crate
locally. Expect the three `ken-interp` L1 rows to fail for unbuilt-capability
reasons, which is the `1c` policy class, not over-annotation.

## 3. Acceptance criteria

### `AC-1` — the sweep asserts a POSITIVE, never the absence of a failure token

> **MEASURED:** the job asserts the expected suppressed-row **count** and the
> exit status.
> **CLAIMED:** the sweep ran and observed the population.
> **THE GAP:** a job that selects zero tests and exits 0 satisfies a required
> check while carrying no signal. **That exact defect already shipped here** —
> two `px8f` jobs went to zero selection, and the aggregator
> (`ci.yml:296-304`) tests only `result == success`. A check for "no failure
> token in the output" passes identically when nothing ran.

### `AC-2` — the sweep is proved live by mutation, not by its colour

> **MEASURED:** with one known-failing row un-ignored, the sweep's report
> **changes**; restore it and the report returns.
> **CLAIMED:** the sweep reaches the suppressed population.
> **THE GAP:** a green sweep is exactly what a sweep that never ran produces.
> Record the observed before/after counts, not "control passed". Say which row
> you used.

### `AC-3` — the registry cannot deselect anything from the MAIN gate

> **MEASURED:** temporarily add a **live, passing** test to the policy registry;
> the main `cargo nextest run --workspace --locked` job still runs it. Remove it.
> **CLAIMED:** the exemption mechanism is scoped to the sweep.
> **THE GAP:** if the registry is implemented as a default nextest profile or a
> repo-wide filterset, it silently narrows every run, including the required
> gate. **A mechanism that can exempt a row from the sweep must be structurally
> unable to exempt it from the gate**, and this control is what distinguishes
> the two. If it fires, the carrier is in the wrong place — move it, do not
> add a warning.

### `AC-4` — non-blocking is demonstrated, not asserted

> **MEASURED:** with the sweep reporting a newly-passing row, the required
> `build + test` check is still green.
> **CLAIMED:** the sweep cannot block an unrelated candidate.
> **THE GAP:** "we set it non-blocking" is a claim about intent. The
> aggregator's wiring is the thing that decides, and it already reads job
> results.

### `AC-5` — bounded wall-clock, cost row exempt from run one

> **MEASURED:** the cost row at `boundary_value_clif.rs:7473` is in the registry
> from `D1`, and the job's observed duration is recorded.
> **CLAIMED:** the sweep is affordable to run standing.
> **THE GAP:** ~142 seconds is not visible in a pass/fail bit, and an
> instrument nobody wants to run is an instrument that gets disabled.

### `AC-6` — no regression

Green in CI. Per `COORDINATION §12` this means CI, **not** a local
`--workspace` run. Locally, per-crate only.

## 4. Banned scope

- **Do not un-ignore any row as part of this node.** The eight owner nodes own
  their repairs. This node builds the instrument that reports on them; it does
  not do their work, and a row that starts passing is routed, not fixed here.
- **Do not put policy exemptions in a default nextest profile or any repo-wide
  filterset.** See `AC-3`. If exempting a row from the sweep can also exempt it
  from the required gate, the carrier is wrong.
- **Do not classify by grepping the reason prose.** `D1` decides this; a
  substring match on wording is the one form explicitly excluded.
- **Do not make the sweep a required check.** `D2` and `AC-4`.
- **Do not re-baseline the expected count to whatever the first run produces.**
  If the observed population differs from `1a`'s 50, that is a finding to
  report — the anchored grep is reproducible and the delta has a cause.

## 5. Hard stop

Stop and report rather than proceeding if:

- nextest cannot express ignored-only selection (`D0`), or the CI job cannot be
  wired non-blocking without touching the aggregator's contract.
- `AC-3` fires — the registry can narrow the main gate. That is a mechanism
  question, not a tuning exercise.
- `D5` finds a row that **passes** while ignored. That is over-annotation: a
  live repair with its cover switched off, exactly `1g`'s shape. Report it to
  the Steward with the row and its owner node; do not un-ignore it here.
- The population at `368ff87e` is not 50 by the anchored grep.

Per the one-hour turn target, a genuine hard stop is a good outcome.

## 6. Contention

Touches `.github/workflows/ci.yml` and adds a registry file. **`ci.yml` is
shared and is the file `RT-SRCBODY-BIND-ORDER`'s CI companion last edited**, so
check for an in-flight candidate touching it before starting.

Adds no `crates/` change, so it does not contend with the runtime nodes on
source. It reads their ignored rows; it does not modify them.

The fleet is single-threaded, so this node runs when Runtime is not.
