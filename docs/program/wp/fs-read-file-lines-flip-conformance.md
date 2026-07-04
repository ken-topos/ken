# fs-read-file-lines-flip — D4/D5 conformance plan (enclave deliverable)

**CV companion to `fs-read-file-lines-flip.md`.** Authored by
conformance-validator, parallel to spec-author's D1 (`lines`/`splitOn`) + D2
(manifest entry-point) and Architect's D3 (Cap runtime rep). This is the
**e2e/discriminator *plan*** — the contract the Runtime build wires into
`crates/ken-cli/tests/` + the example/runner. It is **not** the test code
itself (that lands with the Runtime build); it is what the tests must satisfy,
grounded against the landed substrate (`origin/main@e391d843`). Findings →
Steward (via spec-leader).

## 0. What D4/D5 must pin

The frame's AC2/AC3/AC4 (+ the AC5/AC6 conformance faces) turn on e2e evidence:

- **AC2** — the rosetta corpus flips **15 PASS / 1 gap → 16 / 0**:
  `read-file-lines` PASSES through the real CLI; `KNOWN-GAP.md` deleted; an
  `expected` oracle pins the exact stdout.
- **AC3** — the e2e drives a **real elaborated `using cap` program**
  end-to-end through the CLI entry-point; **no** test hand-feeds the cap as
  `EvalVal::Int(level)` at the `read_bytes` site
  ([[conformance-hand-feeds-the-deliverable]]).
- **AC4** — the grant is **precisely the declared authority**: a
  discriminating pair keyed on `main`'s **own declaration**, not a hand-fed
  attenuation. This is what pins the operator's least-privilege ruling, not
  decoration.
- **AC5/AC6 faces** — unforgeability preserved (grep, §5); totality /
  fail-closed (insufficient/missing → total `Result`, never panic — the
  insufficient arm doubles as the AC6 witness).

Each resolves to a concrete test shape + expected observable, keyed on a
**structural** discriminator (the declared authority level; the producer
path), never a self-reported flag — so a case cannot pass green-vs-green under
a silently-disabled provider (COORDINATION §7).

## 1. Substrate grounding (fixed inputs — `origin/main@e391d843`)

Line numbers perishable; verify against the landed code.

- **`authorizes(cap, _path)`** (`ken-interp/src/eval.rs:1772`) decodes the cap
  as `EvalVal::Int(n) → Authority(n as u8)` and checks
  `check_authority_sufficient(cap, READ_BYTES_REQUIRED_AUTHORITY, …)`. A
  malformed/non-`Int` cap → **fail-closed** (`return false`).
- **Authority levels** (`capabilities.rs:33-35`): `AUTH_NONE=0`,
  `AUTH_PARTIAL=1`, `AUTH_FULL=2`. `READ_BYTES_REQUIRED_AUTHORITY =
  AUTH_PARTIAL = 1` (`eval.rs:1751`). **The only level below required is
  `AUTH_NONE = 0`.**
- **`read_bytes` is the sole real FS op.** `run_io`'s only FS arm is
  `ReadFile` (`eval.rs:1847`+); there is **no FS `WriteFile` driver arm** (the
  `write_id` in `run_io` is the **Console** `Write`, not FS). **⇒ read +
  level-0 is the only driver-level authority discriminator the substrate can
  express today.**
- **The CLI gap** (`ken-cli/src/main.rs:133-136`): `run_file` builds
  `main_term = Term::const_(main_id, vec![])`, `eval`s it, and hands the
  result straight to `run_io`. It **never mints a cap, never inspects `main`'s
  type, never `apply`s**. A `main : using cap : Cap FS -> …` `eval`s to a
  **closure** → `run_io` → `NotAnIOTree`. D2 fills exactly this gap
  (mint-exactly → `apply` → `run_io`).
- **The AC3 anti-pattern is landed and named.** The Phase-2 acceptance test
  (`ken-interp/tests/fs_driver_build_capability_acceptance.rs`) hand-feeds the
  cap: `cap_evalval(cap) = EvalVal::Int(authority(cap).0)` applied at
  `apply(f, cap_evalval(cap), …)` — it exercises the **driver gate**,
  bypassing the surface `main`/CLI entirely. **This test may remain as a
  driver-unit test; AC3 is about the *new* e2e genuinely exercising the CLI
  provider.**
- **Rosetta runner** (`ken-cli/tests/rosetta.rs`): each example dir declares
  **exactly one** oracle — an `expected` file (**stdout must equal it
  byte-for-byte**) **XOR** a `KNOWN-GAP.md` (recorded non-blocker, not run).
  `run_example` runs the **real CLI binary** (`ken_bin()`) and captures
  stdout. A `collections_prelude()` prepend convention exists for helpers not
  in the core prelude.
- **Hermetic fixture** (`conformance/fs/fixtures/three-lines.txt`) =
  `alpha\nbeta\ngamma\n` (checked in, **trailing newline**). Read through the
  real driver.

## 2. AC3 — the no-hand-fed-cap guard (drive the real producer)

The capability this WP **produces** is *a cap value obtained from `main`'s
declaration via the CLI mint path*. A test that constructs the cap as
`EvalVal::Int(1)` and `apply`s it re-validates the **Phase-2 driver** (already
covered) with **zero** exercise of the new provider — the textbook
green-vs-green ([[conformance-hand-feeds-the-deliverable]]). So AC3's guard is
a **producer-grep, not a value assertion**:

- **The e2e must enter through `ken-cli`'s `run_file`** (or the rosetta
  runner, which shells the real CLI binary) — so the cap **originates inside
  `run_file`'s manifest→mint→`apply` path**, not from the test body.
- **Grep the new e2e/runner test source:** it contains **no**
  `EvalVal::Int(…)` / `cap_evalval` construction at the `read_bytes`/`apply`
  site. The cap is never named in the test — it is minted by the CLI under
  test.
- **Structural discriminator:** the code path is `run_file`
  (manifest→mint-exactly→`apply`→`run_io`), *verified by the test invoking
  `run_file`/the CLI binary*, not `apply(f, cap_evalval(…))`. If a build
  shortcut reintroduced the hand-feed, the grep fails — that is the discipline
  making the guard load-bearing, not decorative.

## 3. AC4 — the precisely-declared-authority discriminating pair (the net)

**Structural discriminator: `main`'s *declared* authority level (D2's
manifest).** A single positive case is green-vs-green under a full-minting
CLI, so the net is a **non-degenerate pair** — two `main`s **identical except
for the declared authority level**, same `read_bytes` call, same fixture path,
same D1 line-processing:

| # | `main` declares | CLI mints (correct = exactly-declared) | Driver `authorizes` | Expected observable |
|---|---|---|---|---|
| **M-suff** | **sufficient** authority (≥ `AUTH_PARTIAL`) | that level | `≥1` → ok | **reads** the fixture → line output |
| **M-insuff** | **insufficient** authority (`AUTH_NONE`/0) | `AUTH_NONE` | `0 < 1` → deny | **`Err CapabilityDenied`** (total `Result`) |

**This is a multi-dimensional net; each arm guards a *different* dimension
(promoted K2c-series-2 — a case discriminating on one dimension is vacuous on
another):**

- **M-suff guards the *provider-exists* dimension.** Under the current gap
  (CLI mints nothing), `main` stays a closure → `NotAnIOTree` → fails. M-suff
  reading proves the CLI now **provides** a cap at all. **M-suff does *not*
  discriminate the mint-full bug** — under a full-minting CLI it *also* reads
  (both green).
- **M-insuff guards the *exactly-declared-not-full* dimension** (the
  operator's ruling). Trace both implementations under the **precise bug** AC4
  targets (CLI mints FULL/ambient instead of exactly-declared):
  - **correct** (mint-exactly): mint `NONE` → driver denies → `Err
    CapabilityDenied`.
  - **buggy** (mint-full): mint `FULL` → driver allows → `Ok bytes` → reads.
  - **Opposite observables** (green-vs-red) ⇒ **non-vacuous**. A full-minting
    CLI passes M-suff but **fails** M-insuff — exactly the frame's
    load-bearing claim.

**Right-reason gate (absence-assertion discipline).** M-insuff's rejection
must be the **driver's `authorizes` gate**, not a coincidental earlier
failure:

- M-insuff must **elaborate cleanly** — it carries a `using cap : Cap FS`
  param, so it clears the static `check_capabilities` face (which checks param
  *presence*, not level). A reject at elaboration would be the **wrong reason**
  (bucket (b), [[attribute-a-suite-arm-reject-before-calling-it-a-gap]]).
- The failure is an **in-language `Err CapabilityDenied`** value — assert on
  the exact `capabilitydenied_id` payload (**"not e.g. `NotFound`"**,
  mirroring the Phase-2 `r2` assertion), **not** a bare "reduction failed" /
  `NotAnIOTree` / panic. Asserting the payload pins the driver's authority gate
  as the mechanism, and doubles as the **AC6** fail-closed-totality witness.

## 4. AC2 — corpus flip + the exact expected oracle

- Delete `examples/rosetta/read-file-lines/KNOWN-GAP.md`; add an `expected`
  file so the runner's oracle flips KNOWN-GAP → must-match.
- **The `expected` content is exact stdout** (byte-for-byte). If `main` prints
  each line, stdout = the three lines each `println`'d.
- **⚠ SEAM-B (D1-dependent — pinned to D1's lock, not guessed).** The fixture
  has a **trailing newline** (`…gamma\n`). The expected content depends on
  D1's `lines` **trailing-newline semantics**:
  - **terminator semantics** (Haskell `lines`; `\n` terminates, not
    separates): `lines "alpha\nbeta\ngamma\n" = ["alpha","beta","gamma"]` →
    stdout `alpha\nbeta\ngamma\n` (3 lines).
  - **separator semantics** (`splitOn '\n'`): `= ["alpha","beta","gamma",""]`
    → stdout gains a **trailing blank line** (`alpha\nbeta\ngamma\n\n`).
  - **Pin:** I pin the **invariant** — the three content lines
    `alpha`,`beta`,`gamma` appear **in order** — and pin the **exact
    `expected` file to D1's chosen semantics**. **Recommendation to
    spec-author (D1): lock terminator semantics** (drop the trailing empty —
    the universal `lines` convention), so `expected` = `alpha\nbeta\ngamma\n`.
    If D1 chooses `splitOn`/separator, the `expected` file **must** carry the
    trailing blank line. Over-freezing the wrong spelling yields a case that
    falsely fails a valid `lines` (K2/T1 exact-granularity) — so the token is
    D1's to lock; I pin the value-set + order invariant and hold the literal
    until D1 lands.

## 5. AC5/AC6 conformance faces

- **AC5 (unforgeability) — producer-grep, not a value case** (a value cannot
  witness "no surface term forges a cap"). Assert **structurally**: (a) no
  surface intro form constructs a `Cap`/the D3 runtime rep — `mint` stays
  non-surface-callable; (b) the only cap-producing site reaching the driver is
  the CLI mint. This is Architect's D3 lane (soundness); the conformance face
  is the grep, carried here for completeness. If D3 blesses `EvalVal::Int`, the
  grep must confirm no surface path emits an `EvalVal::Int` into the cap
  position except the CLI mint.
- **AC6 (totality/fail-closed)** — the M-insuff arm (§3) and a **missing-file**
  arm (a `main` reading an absent path → `Err NotFound`, total, no panic) are
  the conformance faces: outer-ring failure surfaces as a total in-language
  `Result`, the pure core untouched.

## 6. Cross-deliverable seams surfaced

- **⚠ SEAM-A (D2, LOAD-BEARING — AC4's expressibility contracts D2).** For AC4
  to be a genuine **driver-level** discriminator, D2's manifest must let a
  `main` declare an authority **level of `AUTH_NONE`** while **(i)** keeping
  the `using cap : Cap FS` param (so it clears `check_capabilities` — reject at
  elaboration is the wrong reason), **and (ii)** having the CLI **mint that
  level-0 cap and bind it** so it *reaches the driver* and is denied at
  `authorizes`. If D2 instead treats "declares `AUTH_NONE`" as "**provide no
  cap**", the insufficient arm fails at bind/elaboration (an unbound `using
  cap` / `NotAnIOTree`), **not** at the authority gate — and AC4 collapses to
  green-vs-green (vacuous), because with the substrate's only sub-required
  level being 0, there is **no other present-but-insufficient level** to
  witness "exactly-declared, not full" (there is no FS write op requiring
  `FULL` to use as an alternative lever, §1). **This is not a build detail — it
  is the condition under which AC4 tests the operator's ruling at all.
  Requesting spec-author (D2) confirm the manifest admits a bound level-0
  declaration reaching the driver; if the manifest cannot express a
  sub-`PARTIAL` bound level, that is a genuine sub-fork → Steward → operator**
  (the alternative — deferring AC4's driver-level face to a richer authority
  lattice — weakens the WP's load-bearing AC).
- **SEAM-B (D1)** — the `lines` trailing-newline semantics fixes the AC2
  `expected` oracle (§4). Recommend terminator semantics; pinned to D1's lock.
- **SEAM-C (D3, Architect)** — AC3/AC4 assume the CLI-minted cap **decodes at
  the driver as the declared level** (`authorizes` reads `EvalVal::Int(level)`).
  If D3 blesses a real opaque `EvalVal::Cap`, `authorizes`/the discriminator
  keys must be re-pointed to read the level from the new rep — the *plan* (the
  declared-level discriminator) is representation-agnostic, but the **grep
  targets in §2/§5 and the decode in §1** must track D3's choice.

## 7. Independence + gate note

Per spec-leader's routing, my D4/D5 piece is **cross-checked against
spec-author's D1/D2 + Architect's D3** at the gate. At the Decision I state
precisely: my **Spec/fidelity vote** attests spec-author's D1/D2 (the manifest
+ helper I did **not** author) and reconciles them against this plan's seams;
**this D4/D5 conformance plan is my authored contribution**, for **Architect's
soundness review** — I do **not** self-review it
([[disclaimed-framing-still-binds-your-own-companion-artifact]]). The AC3/AC4
seam: D2 pins *how* `main` declares authority; D3 pins the *runtime rep*; I pin
the **discriminating shapes + expected observables + right-reason gates** (this
file, §2-§4) that prove the provider grants exactly-declared, not full.
