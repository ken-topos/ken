# PX6 — Interp/native effect differential harness (Verify)

- **ID:** PX6 · **Owner:** Team Verify · **Size:** M · **Risk:** Medium
  (new crate; correctness of the equivalence judge itself).
- **Objective:** Create `crates/ken-verify` and build the harness that runs the
  SAME Ken program under the interpreter and the produced native executable and
  proves they are **externally equivalent** — comparing canonical external
  deltas (stdout, stderr, filesystem deltas, error identity, ordered effect
  trace, exit status), never return values — against twin byte-identical real
  temp roots. It is the guard that native effects cannot disappear, reorder, or
  diverge silently.
- **Depends on:** PX4 (merged, `513955fe`) for the native artifact + interp
  baseline; **PX5** for the canonical observation *types* and the native-effect
  *production* it compares. **Gate:** G-Ward-seam / native-effect lane. Guards
  report §16 risk "native effects disappear or reorder silently."
- **Sibling:** PX5 (Runtime) — released concurrently (see coordination note).

## Fixed inputs — DO NOT REOPEN (cite, do not relitigate)

Controlled by **ADR-0018** (`docs/adr/0018-native-effect-execution-contract.md`,
Decision `dec_2pgkqkddt6eh3` RESOLVED, grounded `513955fe`) — esp. §4 (canonical
observation/trace/error) and §5 (PX6 home + twin real roots). Also binding:

- **PX6 lives in a NEW `crates/ken-verify`, owned by Team Verify.** It owns
  orchestration, comparison, mismatch diagnostics, and the mutation net.
  **Canonical types (observation, trace, error, delta) are DEFINED by PX5** in
  `ken-host`/`ken-runtime`; `ken-verify` **consumes** them and does NOT redefine
  them (ADR §5).
- **The existing `ken-runtime` native-execution differential/provenance report
  remains Runtime's substrate.** PX6 may call it but MUST NOT make Runtime the
  owner + judge of its own effect equivalence (ADR §5, §"Rejected alternatives").
- **The passing FS lane uses twin byte-identical REAL temp roots** — interp
  `PosixHost` cap at root A vs. the produced native artifact's host-context cap at
  root B, identical ProcessInput/stdin/ambient/`ProgramCaps`/scenario.
  **`CaptureHost` is unit/negative-control evidence ONLY** — a virtual host
  cannot pass the lane; it does not exercise the linked Linux artifact (ADR §5,
  §"Rejected alternatives").
- **No comparator-side normalization may repair PX5** (ADR §"Consequences for
  framing"): the harness compares, it does not paper over a real divergence.
- **Equality excludes** pointers, cap slots, fds, inode/device IDs, absolute temp
  roots, addresses, timestamps, non-observable perms, and diagnostic prose; error
  comparison is on the **enum identity**, not message strings; directory order is
  canonical byte order (ADR §4). No nondeterminism normalized away (two real
  `Clock.WallNow` reads are not equal just for both being clocks).
- Trust: `ken-verify` is a tested/target-validated harness — **zero kernel rule,
  zero new Ken postulate, no proof-of-confinement claim.**

## Scope

**In scope (PX6 owns):** the `crates/ken-verify` workspace crate; the independent
executor orchestration (run both lanes on one scenario); the twin-real-root FS
harness; the canonical comparator over `EffectObservationV1`; the mismatch report
+ diagnostics; and the full discriminator/mutation net.

**Out of scope (belongs to PX5):** defining the canonical types; native-effect
production; the host shim / capability carriage / marshalling. Do NOT put PX6 in
`ken-runtime`. Do NOT use `CaptureHost` as the passing substrate. Do NOT invent
comparator-side normalization to mask a PX5 divergence.

## Coordination note (concurrent with PX5 — read this)

PX6 has substantial independent work it can start immediately: create the crate,
build the two-lane orchestration, the interp lane (fully available today via
`ken-interp` `run_io` + `PosixHost::new_at(tempdir)` + `mint_scoped_fs_cap`), the
native-lane spawn of the PX4 artifact, the twin-real-root scaffolding, and a
**first differential over the PX4-observable deltas** (stdout, stderr, exit
status — available now, no PX5 dependency). The canonical `EffectObservationV1`
comparison arm (filesystem delta, error-identity enum, ordered effect trace) is
**typed by ADR-0018 §4** — code the comparator against those pinned shapes and
**import the concrete types from PX5's `ken-host`/`ken-runtime` placement** as
PX5 lands them (PX5 is directed to land the canonical observation vocabulary
early for exactly this). A lane moves to `NativeTested` only when PX5 has made
that op executable AND its external-delta evidence passes here — the two campaigns
meet at the initial 5-op native-tested subset (Console.Write/Flush/IsTerminal,
FS.WriteFile/ReadFile).

## Mandated deliverable outline — each section ends in a concrete, implementable choice

1. **The `ken-verify` crate.** Add `crates/ken-verify` to the workspace
   (`Cargo.toml` members). It depends on `ken-interp` (interp lane + oracle),
   `ken-runtime` (native artifact build/spawn + the existing differential
   substrate), and `ken-host` (canonical types once PX5 places them). It is
   Verify-owned; no Runtime ownership of the equivalence judgment.

2. **Two-lane scenario runner.** One `Scenario` = raw ProcessInput + scripted
   stdin/ambient inputs + declared `ProgramCaps` shape + the Ken entry. The
   runner executes it (a) through the interpreter (`run_io` with a real
   `PosixHost` rooted at temp root A + minted scoped cap) and (b) through the
   produced native executable (PX4 packaging, host-context cap at temp root B),
   both from byte-identical inputs. Returns two `EffectObservationV1`.

3. **Twin-real-root FS harness.** Create two byte-identical real temp-dir roots
   A/B per scenario; seed both identically; run each lane against its own root;
   snapshot both roots after. Deltas are computed from real filesystem state
   (relative raw-byte path, node kind, file bytes, symlink-target bytes),
   excluding inode/device/timestamps/absolute-root/non-observable-perms.

4. **Canonical comparator.** Compare the two `EffectObservationV1` field-by-field
   using ADR §4 equality: raw stdout/stderr bytes; FS delta by canonical
   byte-order path + kind + bytes; **error identity by enum** (exact cap denials
   + `IOError` incl `Other(raw_os_code)` where observable + FS-op/path/nested);
   ordered `effect_trace` by `{sequence, HostOpV1, cap trace-identity, request,
   outcome}`; exit status via PX4's shared mapper. A mismatch is a hard fail with
   a precise diagnostic; **no normalization repairs a divergence.**

5. **Discriminator / mutation net (the teeth).** Include an executable
   discriminator for each failure mode ADR §5 enumerates: silent skip, duplicated
   resume, reordered events, stdout/stderr swap, path-byte normalization,
   weakened error identity, wrong capability token, denied-before-host-action,
   filesystem-mutation-without-trace, trace-without-mutation, target/effect
   manifest mismatch, and each op status transition. Each must be a control that
   **stays green under a runner-only/return-value proxy but fails here** (the PX4
   retro carry: gate the artifact end-to-end, not a proxy).

6. **Status-transition gate.** A catalog op moves
   `RepresentedUnavailable`/`Unsupported` → `NativeTested` ONLY when its exact
   artifact + external-delta evidence passes. The initial pass covers the 5-op
   subset; the deferred 9 remain named unavailable lanes and PX6 asserts they
   stay explicit/named (not silently skipped) on the artifact.

7. **Honesty + trust disclosure.** `ken-verify` discloses tested/target-validated,
   not proved. Enumerate any new trusted surface (should be none beyond a test
   harness). `CaptureHost`-based cases are labelled unit/negative-control, never
   the passing lane.

## Acceptance criteria (testable)

- **AC1 — twin-root external equivalence, subset.** For each native-tested op
  (Console.Write/Flush/IsTerminal, FS.WriteFile/ReadFile), a scenario run through
  both lanes against twin real roots produces byte-equal `EffectObservationV1`
  (stdout/stderr/FS-delta/error-identity/ordered-trace/exit). Interp is the
  oracle; the native artifact is the subject.
- **AC2 — every discriminator bites.** Each mutation in the §5 net (silent skip,
  reorder, stdout/stderr swap, weakened error identity, wrong cap token,
  denied-before-host-action, mutation-without-trace, trace-without-mutation,
  manifest mismatch, path normalization, duplicated resume) is caught: an
  injected divergence fails the comparator, and each control **would stay green
  under a return-value/runner-only proxy** (proving the harness gates the
  artifact's external behavior, not a proxy).
- **AC3 — real-artifact substrate.** The passing lane spawns the produced native
  executable and uses real temp roots; a `CaptureHost`-only variant is present
  ONLY as a negative/unit control and is asserted to be insufficient to pass.
- **AC4 — no comparator-side repair.** A test proves the comparator does not
  normalize away a genuine divergence (e.g. an injected reorder or byte-diff is
  NOT masked); error comparison is on enum identity, not strings.
- **AC5 — home + ownership.** PX6 lives in `crates/ken-verify` (Verify-owned),
  consumes the canonical types from `ken-host`/`ken-runtime` (does not redefine
  them), and does not make `ken-runtime` the judge of its own equivalence.
- **AC6 — deferred lanes stay named.** The 9 non-native-tested ops are asserted
  to remain explicit named unavailable lanes on the artifact (not silent skips);
  the status-transition gate only promotes on real evidence.
- **AC7 — CI-green, honesty.** **No-regression = green in CI** (full locked
  workspace), never a local `--workspace` run (this box builds targeted only, via
  `scripts/ken-cargo -p ken-verify`). Disclosure tested/validated, not proved.

## Do-not-reopen guards

- ADR-0018 is controlling; do NOT redefine canonical types (import from PX5), do
  NOT relocate PX6 out of `ken-verify`, do NOT use `CaptureHost` as the passing
  lane, do NOT compare return values / UTF-8 / error strings / normalized clock
  reads (all REJECTED in ADR §"Rejected alternatives").
- Do NOT invent comparator-side normalization to make a PX5 divergence pass —
  a divergence is PX5's to fix; route it, don't paper it.
- A genuinely new fixed boundary (only) hard-stops and routes to Steward/Architect.

## Grounding anchors (landed `513955fe`)

**Interp lane (available now):** `eval.rs run_io:4216`, `drive_h:1832`,
`drive_h_instrumented:4423` (ordered trace), `HostHandler:2280`, `PosixHost:2379`
(`new_at:2389`, `mint_scoped_fs_cap:2439`), `CaptureHost:2897` (negative control;
`fs_nodes:3001`), errors `RunIoError:3611`. **Native lane (PX4):**
`native_process_entrypoint.rs:147/155` (JIT + `_with_stderr`),
`object_linker_packaging.rs:338 build_process_starter_executable_artifact`,
spawn `smoke_executable:664 → Command::output()` (stdout `:676`, status `:677`,
stderr `:657`), determinism tests `:1703–1733`. **Substrate to call, not own:**
`native_execution_differential.rs` (`RuntimeObservation` stdout+exit only `:169`;
golden pattern `report_hash:698`, `canonical_…_bytes:1130`, suite `:603`); policy
lane `NativeEffectForeignExecutableStatus:237`. **Capabilities:**
`capabilities.rs` (RightSet/Cap/FsScope/Authority). Golden-test style:
`ken-interp/tests/f1_bignum_acceptance.rs`, `i5_scoped_capability.rs`.
Workspace members: `Cargo.toml:3–10` (add `ken-verify`).
