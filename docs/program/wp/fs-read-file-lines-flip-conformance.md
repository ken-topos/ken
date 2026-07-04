# fs-read-file-lines-flip — D4/D5 conformance plan (enclave deliverable)

**CV companion to `fs-read-file-lines-flip.md`.** Authored by
conformance-validator, parallel to spec-author's D1 (`lines`/`splitOn`) + D2
(enriched-signature manifest) and Architect's D3 (Cap runtime rep). This is the
**e2e/discriminator *plan*** — the contract the Runtime build wires into
`crates/ken-cli/tests/` + the example/runner. It is **not** the test code
itself (that lands with the Runtime build); it is what the tests must satisfy,
grounded against the landed substrate (`origin/main@e391d843`). Findings →
Steward (via spec-leader).

> **Gate erratum (reconciled to the landed D1/D2/D3).** My original draft
> (`24efc59`, off the frame `c78df27`) used the frame-era illustrative spelling
> `using cap : Cap FS` and stated SEAM-A/B/C as *open*. The enclave then landed:
> **D2** — the enriched signature `Cap : Auth -> Type0` (`data Auth = ANone |
> APartial | AFull`), so a cap param is `(cap : Cap APartial)` — **`Cap a`, not
> `Cap FS`** (spec-author `238569e`, operator+Architect steer: authority-only
> index, effect stays on the `FS (…)` codomain); `read_bytes : (a : Auth) ->
> Cap a -> Bytes -> FS (…)` **authority-polymorphic** (design **α**). **D3** —
> the runtime cap is a real opaque **`EvalVal::Cap(capabilities::Cap)`**, *not*
> `EvalVal::Int(level)` (Architect ruling; sole producer = CLI mint,
> `authorizes` fail-closes on anything else). This doc is reconciled to those
> tokens throughout; all three seams are now **resolved** (§6). A stale token I
> introduced is mine to erratum at the gate
> ([[disclaimed-framing-still-binds-your-own-companion-artifact]]).

## 0. What D4/D5 must pin

The frame's AC2/AC3/AC4 (+ the AC5/AC6 conformance faces) turn on e2e evidence:

- **AC2** — the rosetta corpus flips **15 PASS / 1 gap → 16 / 0**:
  `read-file-lines` PASSES through the real CLI; `KNOWN-GAP.md` deleted; an
  `expected` oracle pins the exact stdout.
- **AC3** — the e2e drives a **real elaborated cap-param program** end-to-end
  through the CLI entry-point; **no** test hand-constructs the cap (D3's
  `EvalVal::Cap(…)`, or a bare `EvalVal::Int`) at the `read_bytes` site
  ([[conformance-hand-feeds-the-deliverable]]).
- **AC4** — the grant is **precisely the declared authority**: a
  discriminating pair keyed on `main`'s **own declaration** (the `Auth` index
  on its cap param), not a hand-fed attenuation. This is what pins the
  operator's least-privilege ruling, not decoration.
- **AC5/AC6 faces** — unforgeability preserved (grep, §5); totality /
  fail-closed (insufficient/missing → total `Result`, never panic — the
  insufficient arm doubles as the AC6 witness).

Each resolves to a concrete test shape + expected observable, keyed on a
**structural** discriminator (the declared authority index; the producer
path), never a self-reported flag — so a case cannot pass green-vs-green under
a silently-disabled provider (COORDINATION §7).

## 1. Substrate grounding (fixed inputs — `origin/main@e391d843`)

Line numbers perishable; verify against the landed code. The D2/D3 **design**
(enriched `Cap`, `EvalVal::Cap`) is not yet on `origin/main` — the build lands
it; the items below are the landed substrate it builds on.

- **`authorizes(cap, _path)`** (`ken-interp/src/eval.rs:1772`) today decodes the
  cap as `EvalVal::Int(n) → Authority(n as u8)` and checks
  `check_authority_sufficient(cap, READ_BYTES_REQUIRED_AUTHORITY, …)`; a
  malformed/non-`Int` cap → **fail-closed** (`return false`). **D3 re-points
  this** to `EvalVal::Cap(cap) => check_authority_sufficient(cap, …), _ =>
  false` (§6 SEAM-C) — reading the `Authority` off the real minted struct, no
  re-mint from a bare scalar.
- **Authority levels** (`capabilities.rs:33-35`): `AUTH_NONE=0`,
  `AUTH_PARTIAL=1`, `AUTH_FULL=2`; the surface `Auth` ctors map
  `ANone/APartial/AFull ↔ 0/1/2` (D2↔D3 contract).
  `READ_BYTES_REQUIRED_AUTHORITY = AUTH_PARTIAL = 1` (`eval.rs:1751`). **The
  only level below required is `ANone (0)`.**
- **`read_bytes` is the sole real FS op.** `run_io`'s only FS arm is
  `ReadFile` (`eval.rs:1847`+); there is **no FS `WriteFile` driver arm** (the
  `write_id` in `run_io` is the **Console** `Write`, not FS). **⇒ read +
  `ANone` is the only driver-level authority discriminator the substrate can
  express today.**
- **The CLI gap** (`ken-cli/src/main.rs:133-136`): `run_file` builds
  `main_term = Term::const_(main_id, vec![])`, `eval`s it, and hands the
  result straight to `run_io`. It **never mints a cap, never inspects `main`'s
  type, never `apply`s**. A `main : (cap : Cap APartial) -> …` `eval`s to a
  **closure** → `run_io` → `NotAnIOTree`. D2 fills exactly this gap (read the
  `Auth` index off `main`'s type → mint-exactly → `apply` `EvalVal::Cap` →
  `run_io`).
- **The AC3 anti-pattern is landed and named.** The Phase-2 acceptance test
  (`ken-interp/tests/fs_driver_build_capability_acceptance.rs`) hand-feeds the
  cap: `cap_evalval(cap) = EvalVal::Int(authority(cap).0)` applied at
  `apply(f, cap_evalval(cap), …)` — it exercises the **driver gate**,
  bypassing the surface `main`/CLI entirely. **This test may remain as a
  driver-unit test (updated by the build to D3's `EvalVal::Cap`); AC3 is about
  the *new* e2e genuinely exercising the CLI provider.**
- **Rosetta runner** (`ken-cli/tests/rosetta.rs`): each example dir declares
  **exactly one** oracle — an `expected` file (**stdout must equal it
  byte-for-byte**) **XOR** a `KNOWN-GAP.md` (recorded non-blocker, not run).
  `run_example` runs the **real CLI binary** (`ken_bin()`) and captures
  stdout. A `collections_prelude()` prepend convention exists for helpers not
  in the core prelude.
- **Hermetic fixture** (`conformance/fs/fixtures/three-lines.txt`) =
  `alpha\nbeta\ngamma\n` (checked in, **trailing newline**). Read through the
  real driver; `bytes_decode : Bytes -> String` bridges to the D1 `lines`
  helper (the fixture is valid UTF-8, so `bytes_decode` reduces — D1).

## 2. AC3 — the no-hand-fed-cap guard (drive the real producer)

The capability this WP **produces** is *a cap value obtained from `main`'s
declaration via the CLI mint path*. A test that hand-builds the cap (a bare
`EvalVal::Int(1)`, or a hand-constructed `EvalVal::Cap(…)`) and `apply`s it
re-validates the **Phase-2 driver** (already covered) with **zero** exercise of
the new provider — the textbook green-vs-green
([[conformance-hand-feeds-the-deliverable]]). So AC3's guard is a
**producer-grep, not a value assertion**:

- **The e2e must enter through `ken-cli`'s `run_file`** (or the rosetta
  runner, which shells the real CLI binary) — so the cap **originates inside
  `run_file`'s manifest→mint→`apply` path**, not from the test body.
- **Grep the new e2e/runner test source:** it contains **no** hand-constructed
  cap value (`EvalVal::Cap(…)` / `EvalVal::Int(…)` / `cap_evalval`) at the
  `read_bytes`/`apply` site. The cap is never built in the test — it is minted
  by the CLI under test.
- **Structural discriminator:** the code path is `run_file`
  (manifest→mint-exactly→`apply`→`run_io`), *verified by the test invoking
  `run_file`/the CLI binary*, not `apply(f, <hand-built cap>)`. If a build
  shortcut reintroduced the hand-feed, the grep fails — that is the discipline
  making the guard load-bearing, not decorative.

## 3. AC4 — the precisely-declared-authority discriminating pair (the net)

**Structural discriminator: the `Auth` index on `main`'s cap param (D2's
enriched signature).** A single positive case is green-vs-green under a
full-minting CLI, so the net is a **non-degenerate pair** — two `main`s
**identical except for the declared `Auth` index**, same `read_bytes` call,
same fixture path, same D1 line-processing:

| # | `main`'s cap param | CLI mints (correct = exactly-declared) | Driver `authorizes` | Expected observable |
|---|---|---|---|---|
| **M-suff** | `(cap : Cap APartial)` (or `Cap AFull`) | that level | `≥ APartial` → ok | **reads** the fixture → line output |
| **M-insuff** | `(cap : Cap ANone)` | `ANone` | `ANone < APartial` → deny | **`Err CapabilityDenied`** (total `Result`) |

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
  targets (CLI mints `AFull`/ambient instead of exactly-declared):
  - **correct** (mint-exactly): mint `ANone` → driver denies → `Err
    CapabilityDenied`.
  - **buggy** (mint-full): mint `AFull` → driver allows → `Ok bytes` → reads.
  - **Opposite observables** (green-vs-red) ⇒ **non-vacuous**. A full-minting
    CLI passes M-suff but **fails** M-insuff — exactly the frame's
    load-bearing claim.

**Right-reason gate (absence-assertion discipline).** M-insuff's rejection
must be the **driver's `authorizes` gate**, not a coincidental earlier
failure:

- M-insuff must **elaborate cleanly** — it carries a cap param `(cap : Cap
  ANone)`, so it clears the static `check_capabilities` face (the FS effect is
  still declared; the check keys on the `Cap` head, not the `Auth` level). A
  reject at elaboration would be the **wrong reason** — that is the *distinct*
  `MissingCapability` arm of a `main` with **no** cap param (bucket (b),
  [[attribute-a-suite-arm-reject-before-calling-it-a-gap]]).
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
- **SEAM-B — LOCKED (D1, spec-author `238569e`).** D1 locks **terminator
  semantics** (`str::lines()`): a trailing `\n` does **not** yield a trailing
  empty line, so `lines "alpha\nbeta\ngamma\n" = ["alpha", "beta", "gamma"]`
  (exactly 3). **The AC2 `expected` oracle is therefore `alpha\nbeta\ngamma\n`**
  (no trailing blank line). This is the value-set + order invariant I pinned,
  now locked to D1's terminator choice (a `splitOn`/separator build would have
  needed a trailing blank — D1 explicitly drops it).

## 5. AC5/AC6 conformance faces

- **AC5 (unforgeability) — producer-grep, not a value case** (a value cannot
  witness "no surface term forges a cap"). Assert **structurally**: (a) no
  surface intro form constructs a `Cap`/an `EvalVal::Cap` — `mint` stays
  non-surface-callable, and (per Architect's D3 grep) `attenuate`/`mint` have
  **no** surface binding, so the only surface term of type `Cap a` is the bound
  cap param; (b) the sole `EvalVal::Cap` producer reaching the driver is the
  CLI mint. This is Architect's D3 lane (soundness); the conformance face is the
  grep, carried here for completeness — no surface path emits an `EvalVal::Cap`
  into the cap position except the CLI mint.
- **AC6 (totality/fail-closed)** — the M-insuff arm (§3) and a **missing-file**
  arm (a `main` reading an absent path → `Err NotFound`, total, no panic) are
  the conformance faces: outer-ring failure surfaces as a total in-language
  `Result`, the pure core untouched.

## 6. Cross-deliverable seams — all resolved

- **SEAM-A (D2) — CONFIRMED (design α, spec-author `238569e` + Architect
  `evt_fgkd29xbf35q`).** AC4's driver-level discriminator is real: `read_bytes`
  is **authority-polymorphic** (`(a : Auth) -> Cap a -> …`, no static
  sufficiency check), so a `main : (cap : Cap ANone) -> FS (…)` **keeps its cap
  param** (clears `check_capabilities` — the FS effect is still declared), gets
  a **bound `ANone` cap minted + applied** (`EvalVal::Cap`), **reaches the
  driver**, and is denied at `authorizes` with `CapabilityDenied`. *"Declares
  `ANone`" is not "provide no cap"* — the cap is minted and bound, simply
  insufficient at the runtime gate. The alternative **β** (a static minimum-gate
  on `read_bytes`) would reject at elaboration, contradicting locked AC4
  ("refused **at the driver**") and deadening the sole runtime net — so AC4
  forces α. **My load-bearing concern is answered; no fork.** (The distinct
  no-cap-param → `MissingCapability`-at-elab arm is the wrong-reason foil, kept
  separate.)
- **SEAM-B (D1) — LOCKED to terminator semantics** (§4); oracle
  `alpha\nbeta\ngamma\n`.
- **SEAM-C (D3) — RESOLVED: real opaque `EvalVal::Cap(capabilities::Cap)`**
  (Architect ruling, not `EvalVal::Int`). The §2/§5 grep targets and the §1
  `authorizes` decode are pinned to `EvalVal::Cap` accordingly. The plan's
  discriminator (the declared `Auth` index) is representation-agnostic; D3's
  choice only fixes the concrete decode/grep tokens, now settled.

## 7. Independence + gate note

Per spec-leader's routing, my D4/D5 piece is **cross-checked against
spec-author's D1/D2 + Architect's D3** at the gate. At the Decision I state
precisely: my **Spec/fidelity vote** attests spec-author's D1/D2 (the enriched
signature + helper I did **not** author) and reconciles them against this
plan's seams (all three resolved above); **this D4/D5 conformance plan is my
authored contribution**, for **Architect's soundness review** — I do **not**
self-review it
([[disclaimed-framing-still-binds-your-own-companion-artifact]]). The AC3/AC4
seam: D2 pins *how* `main` declares authority (the `Auth` index); D3 pins the
*runtime rep* (`EvalVal::Cap`); I pin the **discriminating shapes + expected
observables + right-reason gates** (this file, §2-§4) that prove the provider
grants exactly-declared, not full.
