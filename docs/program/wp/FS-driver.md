# FS-driver — real file I/O via a capability-carrying `[FS]` effect (VAL2 #9 / OQ-B)

**Steward frame → spec enclave (elaborate) → Runtime + Sec + conformance
(build).** Design-settled capability WP closing VAL2 finding #9
(`read-file-lines`: `[FS]` exists at the type level but `read_bytes` has **no**
`ken-interp` reduction — no Ken program can do file I/O). Product decision
**locked by the operator** (OQ-B, `VAL2-gap-design-OQs.md`, 2026-07-03). **This
is a multi-lane mini-workstream, not one small fix** — the enclave/Architect may
**decompose it into a short WP series** (driver / capability / conformance);
propose the decomposition back to Steward. Owner: spec-leader elaborates
semantics; **Runtime** (driver) + **Sec** (capability) + **conformance** build.
Gate: **/spec + /conformance touching** → spec enclave elaboration, **Architect**
(soundness: totality + kernel-untouched + capability model), **Spec review**
(conformance-validator), team QA, CI. Findings → **Steward**.

## The locked decision — DO NOT REOPEN

- **Real FS driver — NO mock.** File I/O runs through a **real driver** extending
  the existing `ITree`/Console effect-interpreter (the same free-monad Console
  runs on). The operator ruled a synthetic mock FS **wasted work** (we will
  ultimately need real FS interaction). Do **not** build a mock/virtual FS.
- **Conformance determinism via checked-in hermetic fixture files.** I/O is
  nondeterministic; determinism comes from **version-controlled fixture files**
  the test reads through the **real** driver (real code path, deterministic
  input) — **not** a throwaway mock and **not** by stubbing the read.
- **Capability-carrying effect.** `[FS]` carries the capability/paths a program is
  authorized to touch (aligns the **Sec / effect-capability** workstream). File
  access is **not** ambient authority — an undeclared access is rejected.
- **Totality preserved.** The effect is a **pure description** (an `ITree` node);
  nondeterminism lives in the **handler**; the program handles failure as
  `Option`/`Result` (file-not-found, etc.) in-language. The pure core stays
  total. **Kernel / `trusted_base` untouched** (outer-ring effect + runtime
  driver, like Console).

## ★ LANDED SUBSTRATE (fixed inputs — elaborate against THIS, not a survey)

Grounded against `origin/main` (2026-07-03; this `steward/work` worktree checks
out only stub crate `lib.rs`s — **cite via `git show origin/main:<path>`**). All
line numbers are **perishable — verify at pickup, do not trust this frame over
the landed code.** The headline: **the type-level `[FS]` side is already built
and tested; the gap is purely the runtime driver + capability thread + fixtures.**

- **The Console effect is the DIRECT precedent to MIRROR — not invent anew.**
  - Surface/prelude: `data ConsoleOp = Write String`, `console_resp : ConsoleOp →
    Type = Unit`, `IO A := ITree ConsoleOp console_resp A`, `print_line : String →
    IO Unit` (`crates/ken-elaborator/src/prelude.rs` ~`:194,693-724`). The `ITree`
    decl is already **generalized** off the old Console-hardwired 1-param form.
  - **The REAL runtime driver to mirror is `run_io`/`drive_h`** in
    `crates/ken-interp/src/lib.rs` (re-exports `:8-10`; `Vis (Write s) k →
    println!(s); run_io(k unit)`; test `val1_c2_write_prints_and_resumes`
    ~`:2270`). **The FS driver is the file-I/O analog of `run_io`.** (NB: there is
    **no `ken-runtime` crate** — the "Runtime lane" is `ken-interp`.)
- **`read_bytes` is ALREADY registered at the type level with `[FS]` — and
  tested.** `crates/ken-elaborator/src/bytes.rs` registers `read_bytes`/`write_bytes`/
  `append`/`send`/`recv` as prims (`~:125-130`) with `io_effect_rows` mapping
  `read_bytes|write_bytes|append → EffectRow::singleton("FS")`, `send|recv →
  "Net"` (`~:161-171`). The escape/capability check REJECTS an untracked call —
  `tests/l6_acceptance.rs::read_bytes_untracked_is_type_error`. Note `bytes.rs`
  ~`:120-123`: the arg type is a **placeholder `Bytes → Bytes`** ("real Path/Socket
  types are an L7/FFI concern; the EFFECT ROW is what matters").
- **THE GAP (VAL2 #9, crisp):** `read_bytes` has **no `ken-interp` reduction** —
  at runtime it is an **inert `EvalVal::CtorPending`** (`crates/ken-interp/src/
  eval.rs` ~`:1318-1322`); the saturated-prim path (~`:1461-1467`) wires only
  `print_line`/`string_to_list_char`/… and otherwise falls to a `Neutral`
  catch-all. So `[FS]` type-checks and tracks but **drives nothing when run.**
  This WP adds: the reduction dispatching to a real host-I/O handler, the
  capability thread to it, and the fixture conformance — **onto a working
  type-level substrate.**
- **Effect machinery to REUSE (do not fork a second effect system):**
  `crates/ken-elaborator/src/effects/` — `itree.rs` (`ITree` Ret/Vis, `perform`,
  `bind`, `handler_fold`, dependent response type landed via the `[State]` WP),
  `row.rs` (`EffectRow(BTreeSet<EffectName>)`, `is_subset_of` for `ρ_inf ⊆
  ρ_decl`; `EffectName`'s doc lists `FS` first), `check.rs::check_capabilities`
  (~`:127-155`, `EffectError::MissingCapability`), `algebra.rs::CapParam`
  (`using name : Cap E`).
- **Capability tokens (Sec lane) are ALREADY landed** —
  `crates/ken-elaborator/src/capabilities.rs`: unforgeable `CapToken`, `mint`
  (root), **`attenuate`** (monotone-downward, "one directory not the filesystem"),
  **revocation handle**, combined cap+flow gate. Spec: `spec/60-security/
  62-authority.md` (no ambient authority; `Cap_FS`/`Cap_Net` tokens; attenuation;
  revocation). The AC3 discriminating pair (with-cap succeeds / without-cap
  rejected) threads THIS machinery to the FS op at **runtime** (currently the
  token exists but no `Cap_FS` is minted/threaded through an FS op).
- **Cross-effect dependency to account for:** `[FS]` shares the `Vis (inr o)`
  pass-through dispatch path with `[State]`, and **EFF6 console-commute is tagged
  DEFERRED** (`#245 5d723e3`, pending a Console-lift). The elaboration should note
  whether the FS driver needs that lift or is independent (State-effect-build.md
  ~`:86-87,171-173`).

## Means are the enclave's + Architect's call

Fixes the **goal + properties + acceptance**, not the mechanism. The exact op set
(`read_bytes`/`read`/`write`/`open`/…), the effect-row + capability
representation, how a program *declares* its FS capability, and the driver
implementation are the **enclave's elaboration + Architect's** call. Illustrative
signatures are tagged *verify/decide against the landed effect + capability
system, not this line.*

## Mandated deliverable outline (each item resolves to a concrete choice)

1. **Effect ops + real reduction.** Give `read_bytes` (and the enclave-chosen op
   set) a **real `ken-interp` reduction** dispatching to the driver. Reuse the
   `ITree`/Console effect node machinery — **no second effect system** (coordinate
   with the `[State]` WP so both share one effect-dispatch path).
2. **Runtime driver.** The handler that performs the actual file I/O in the
   runtime (real syscalls, outside the pure core). Runtime lane. Specify where it
   lives and how it surfaces failure as `Option`/`Result` to the program.
3. **Capability model.** How `[FS]` carries the authorized paths/capabilities and
   how an **undeclared access is rejected** (Sec lane) — the discriminating case:
   a program without the capability must **fail**, one with it must succeed.
4. **Conformance — real driver + checked-in fixtures.** Tests drive the **real**
   driver against **version-controlled hermetic fixture files** (deterministic,
   real code path). The `read-file-lines` shape (VAL2 #9) is the canonical
   example. **No mock.** Grounded against the landed producer
   ([[conformance-hand-feeds-the-deliverable]]).
5. **Totality/capability statement.** Explicit spec + a check that the effect is
   erasable to its description, the pure core stays total, and the capability is
   enforced (not decorative).

## Acceptance criteria

- **AC1 — Kernel untouched (load-bearing).** `git diff origin/main --
  crates/ken-kernel/` empty; `trusted_base()` unchanged; no new kernel variant.
  `[FS]` is an outer-ring effect + runtime driver + capability surface — the touch
  is `ken-interp` (driver reduction) + `ken-elaborator` (capability thread, if any)
  + `conformance/` (fixtures); **verify by grep, not a test.** (There is no
  `ken-runtime` crate — the driver lives in `ken-interp`, mirroring `run_io`.)
- **AC2 — `[FS]` drives real I/O end-to-end.** `read_bytes` has a real reduction;
  `read-file-lines` reads a **checked-in fixture** through the real driver and
  produces the correct content; failure surfaces as `Option`/`Result`.
- **AC3 — Capability enforced.** A discriminating pair: an access **with** the
  declared capability succeeds; the same **without** it is **rejected** (the
  capability is load-bearing, not decorative). [[taint-axis-orientation-needs-distinguishing-pair]]-style: both arms on the same op.
- **AC4 — Determinism, no mock.** Conformance uses checked-in fixture files +
  the real code path; grep confirms **no mock/virtual FS** was introduced.
- **AC5 — Totality preserved.** The pure core stays total; the effect is a pure
  description; nondeterminism is confined to the handler.
- **AC6 — No regression.** `cargo test --workspace` green; Console and other
  effects unaffected.

## Guardrails (do-not-reopen)

- **NO mock FS** — real driver + checked-in hermetic fixtures.
- **Reuse the `ITree`/Console effect machinery** (shared with `[State]`) — one
  effect-dispatch path, not two.
- **Capability-carrying, not ambient authority** — undeclared access rejected.
- **Kernel / `trusted_base` off-limits**; totality preserved in the pure core.
- **Multi-lane** — if too large for one WP, **decompose and propose to Steward**;
  do not silently narrow scope.

## Sequencing

- **Gate:** /spec + /conformance touching → spec enclave elaborates semantics +
  capability model on this WP branch (may split into a WP series), merges to
  `main` via the Integrator, then Runtime/Sec are kicked. Architect (totality +
  kernel-untouched + capability soundness) + Spec review (conformance-validator) +
  team QA + CI.
- **Suggested decomposition (enclave's call — propose the series back to
  Steward):** the landed substrate makes three lanes natural — **(a) Runtime
  driver** (`ken-interp` reduction dispatching `read_bytes` → a real host-I/O
  handler, mirroring `run_io`; failure as `Option`/`Result`) — the AC2 core;
  **(b) Sec capability thread** (mint/thread a `Cap_FS` from surface `[FS]` to the
  driver so an undeclared access is rejected at runtime — the AC3 discriminating
  pair, reusing the landed `CapToken`/`attenuate`); **(c) conformance** (checked-in
  hermetic fixtures + real driver, the `read-file-lines` shape). (a) is the
  spine; (b)/(c) layer on it. If the enclave keeps it as one WP, that is fine too —
  the point is **don't silently narrow scope**.
- **Lane:** Runtime (`ken-interp` driver) + Sec (`ken-elaborator` capability) +
  conformance. Branch off `origin/main`.
- **Relation to siblings:** shares the effect-handler mechanism with `[State]`
  (both extend the Console `ITree` interpreter) — build the shared effect-dispatch
  path once. Sequence after `[State]` if they'd contend on the same effect-system
  files.
