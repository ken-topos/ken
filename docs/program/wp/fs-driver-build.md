# FS-driver Phase 2 — the build (real driver + capability thread + fixtures)

**Steward build frame → Runtime (lead) + Verify + conformance.** Phase 2 of the
FS-driver series: implement, end-to-end, the file-I/O the merged Phase-1 spec
elaborated. Phase 1 (`fd5451b`, PR #280) front-loaded **all** the design
judgment; this phase is a **mechanical build against that spec** — do not
redesign, do not reopen the locked decisions.

## The authoritative spec is on `main` — build against it, not this frame

This frame is a **thin task/ownership/AC wrapper**. The shovel-ready design lives
in the two merged Phase-1 docs — read them from `main`, they are canonical:

- **`docs/program/wp/FS-driver.md`** (`git show origin/main:…`): deliverables
  **D1** (effect ops + real reduction — mirror `build_print_line_tree`; re-type
  `read_bytes` to the `FS`-monad form; `FSIds`), **D2** (runtime driver arm in
  `ken-interp` `run_io`: `authorizes` check → the sole `std::fs::read` → total
  `Result`; `IOError` sum), **D3** (capability two-face: landed static
  `check_capabilities` + new runtime `authorizes(cap, path)` thread), **D5**
  (totality / kernel-untouched / trust-level statement), and the **"What Phase 2
  builds"** lane table.
- **`docs/program/wp/FS-driver-conformance.md`**: **D4** — the fixture strategy
  (`conformance/fs/fixtures/`, hermetic, real code path, no mock) + the AC3
  discriminating pairs (**S1/S2** static, **R1/R2** runtime authority-level,
  **R2′** path-exclusion Phase-2-deferred with reason).

Every line-number in those docs is **perishable — verify against the landed code
at pickup, never trust the doc over the code** (§2c stale-frame rule).

## Scope (do not narrow, do not widen)

**One op driven end-to-end: `read_bytes` (read).** Reduction + driver + capability
+ fixtures, per D1–D4. `write_bytes`/`append` (`[FS]`) and `send`/`recv`
(`[Net]`) stay **registered-but-undriven** — a named follow-on, written so their
dispatch arms are additive (no re-architecture), **not** silently in scope and
**not** silently dropped (FS-driver.md "Scope of the op set").

## Ownership & branch model (Steward-set — one bundled branch, atomic merge)

- **Branch: `wp/fs-driver-build`** (this branch, off `origin/main@fd5451b`).
  **ONE branch, ONE atomic merge** — not three merge points.
- **Runtime leads** (the driver is the spine, `ken-interp` the primary touch).
  - **runtime-implementer** builds **D1 + D2** (reduction + driver arm +
    `FSOp`/`fs_resp`/`IOError` prelude decls + the `read_bytes` re-type), plus
    the D4 fixtures/harness wiring.
  - **verify-implementer** contributes the **D3 runtime capability thread** —
    the `authorizes(cap, path)` gate + the `Cap_FS` path-scope representation
    (the one open representation choice, see AC8/guardrails) — **onto this same
    branch**, sequenced *after* the spine by the Runtime-leader.
  - Conformance fixtures (D4) land on the same branch (CV reviews fidelity at
    the gate; she authored the plan and does **not** self-review it).
- **Single merge Decision**, opened by **Runtime-leader → Integrator**.

**Why one atomic merge (the load-bearing reason — do not split):** the runtime
driver must **never** sit on `main` without its capability gate live. Landing D1
+ D2 (the driver) alone, even transiently between merges, would put an
**ambient-authority file read on `main`** — a direct violation of the OQ-B-locked
"capability-carrying, not ambient authority" guardrail. Bundling D1–D4 into one
merge is **forced by the locked design**, not a preference.

## Acceptance criteria

**Inherit AC1–AC6 verbatim from `FS-driver.md` §Acceptance criteria** (kernel
untouched / real I/O end-to-end / capability enforced / determinism-no-mock /
totality / no-regression — the no-regression AC is **`cargo test --workspace`
green**, not a single crate). **Plus two hard ACs promoted from Architect's
Phase-1 soundness review** (they gate this build; do not treat as prose):

- **AC7 — Reachability precondition is a HARD, producer-grepped AC (not prose).**
  The FS driver's `tested-not-trusted` posture rests on **"FS ops run only on
  kernel-admitted core."** Prove it structurally, by grepping the producer:
  the FS-driver entry (the `run_io` file-I/O arm) is invoked **only
  post-elaboration**, exactly as `run_io`/`drive_h` is today — a Ken term reaches
  the driver **only after** passing kernel admission; there is no path that
  invokes the FS reduction/driver on an un-admitted term. **Verify-and-pin by
  grep** (mirror the `run_io` invocation site: `ken-cli/src/main.rs` drives it on
  the already-elaborated core), do **not** assume-by-construction. Same shape as
  the X3a precedent where a reachability precondition was elevated from prose to
  a hard build AC. [[tested-not-trusted-posture-needs-reachability-precondition]]
- **AC8 — Trust-level statement is exact (kernel-backing precision).** The doc /
  QA report / test names must state the trust level **precisely**, per Architect:
  - the **runtime** FS gate (`authorizes`/`authority_flows_to`/`is_satisfied` — a
    plain Rust `bool`, zero emission) is **trusted Rust / conformance-netted,
    NOT kernel-backed**; no name or label may claim otherwise
    ([[kernel-backed-claim-grep-the-emission-not-the-name]]);
  - `attenuate`'s **static** obligation **is** kernel-re-checked, but
    "kernel-backed" there precisely means **the kernel verifies the discharge
    certificate** (`Refl` over same-vs-distinct opaque postulates — confirmed to
    reject an over-strong obligation), **not** that the kernel re-derives the
    authority arithmetic (`authority_meet` + the `child == bound` discrimination
    that *constructs* the obligation are **trusted Rust**). Do not let the runtime
    gate borrow the static obligation's kernel-backing.

## Guardrails (do-not-reopen — inherited + build-specific)

- **NO mock FS** — real `std::fs`-backed driver + checked-in hermetic fixtures;
  grep-verifiable no `MockFs`/`VirtualFs`/in-memory shim (AC4).
- **Reuse the `ITree`/Console effect machinery** (shared with `[State]`) — **one**
  effect-dispatch path, exhaustive `Vis` dispatch, **no** second effect system,
  **no** catch-all.
- **Capability-carrying, not ambient authority** — the runtime `authorizes` gate
  is load-bearing (R2 flips on it); a no-op always-true `authorizes` = ambient =
  fails AC3.
- **Kernel / `trusted_base` off-limits** — zero `ken-kernel/` diff, no new
  `Term`/`Decl` variant (AC1, grep not test); totality preserved in the pure core
  (the `eval.rs` FS interception builds an `ITree` and calls **no** `std::fs`).
- **Preserve the landed static face** — the D1 re-type must keep
  `read_bytes_untracked_is_type_error` (+ the `io_effect_rows`/`check_capabilities`
  row check) **green**; if the re-type would regress it, reconciling the two faces
  is an in-scope Phase-2 step, never a silent drop of the static check.
- **Path-scope representation** — the one open choice (extend `Authority`/`Cap`
  with a path-scope, or an FS-specific scope alongside): Architect-approved as
  delegated to this build; the **contract is fixed** (`authorizes` gates,
  `attenuate` only narrows, unauthorized ⇒ `CapabilityDenied`). If the chosen
  representation surfaces a genuine soundness fork PRINCIPLES can't settle, route
  to Steward (→ operator), do not silently pick.

## Gate & sequencing

- **Compaction handoff gate FIRST (§2c — Steward runs it before kickoff).** All
  five involved members — Runtime leader/implementer/QA + verify-implementer +
  verify-QA — are **compact-verified then mentioned**, one act. Build teams
  compact **unconditionally** before a WP (no ctx threshold).
- **Build sequencing within the branch:** D1/D2 spine (runtime-impl) → D3
  capability thread (verify-impl) layered on → D4 fixtures → workspace-green.
- **Single gate:** **Architect** (soundness: totality + kernel-untouched +
  capability model + AC7 reachability + AC8 trust-level) + **conformance-validator**
  (Spec/conformance fidelity; CV's authored D4 goes to Architect's soundness lane,
  not self-reviewed) + **Runtime-QA** + **Verify-QA** + **CI**. Findings →
  **Steward**.
- **Single merge:** Runtime-leader opens the merge Decision → **Integrator** (only
  the Integrator touches `main`). Closes **VAL2's last gap end-to-end**
  (`read-file-lines` → 16 PASS / 0 KNOWN-GAP).
