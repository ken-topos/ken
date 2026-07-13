# WP — Runtime I-4 §B: ProgramCaps mint from declaration + light the I-3 write path

**Owner:** Runtime ring (runtime-leader / runtime-implementer / runtime-qa) ·
**Size:** M · **Base:** `origin/main @ 66c6e15f` · **Consumes:** §C's
program-header capability clause (`wp/i4c-program-header-caps`, enclave) for the
real mint surface (step 2) + the typed capability-API surface (step 7) ·
**Design source of truth:** Architect I-4 ruling
`architect/work:docs/program/wp/i4-program-caps-ruling.md @ be52d82b`, §B +
§A.1/§A.2/§A.3/§A.5/§A.5.1(ii)/§A.6.

## Objective

Turn the landed fixed-authority `ProgramCaps` into an **authority-parametric
`ProgramCaps a`** minted **from the program's declared capability clause** (§C),
thread the FS `Cap a` through the runner, and **light the I-3 write path
end-to-end for the first time**: a program declaring FS `AFull` writes a real
file through `run_io`. Enforce **family containment statically** (a program with
no declared FS capability that performs an FS effect is **ill-typed**) and add
the **Option-(ii) typed-wrapper** authority-*level* static gate (an `APartial`
program that attempts a write is ill-typed), with an op-time `CapabilityDenied`
backstop as defense-in-depth. This is **coarse-v1** (contract §3.1); the
scoped/least-privilege model is I-5, explicitly **out**.

## Settled inputs — PINNED, do not re-derive (cite ruling `be52d82b`)

- **The reframe (Pat, 2026-07-13).** Capability **source = the program
  declaration** (the same header that carries `admits`), **not** a launch grant.
  The runner mints exactly what the header declares; there is **no external
  grant** to compare against, so a grant-mismatch failure model is the wrong
  frame. See memory
  [[ken-owns-program-validity-not-runtime-constraint-caps-declared-in-program]].
- **Record shape (§A.1).** `ProgramCaps` becomes **authority-parametric**:
  `data ProgramCaps (a : Auth) = MkProgramCaps (fsCap : Cap a)` — one FS field,
  authority in the type. (Landed shape is fixed `Cap APartial`, `prelude.rs`
  ~1434-1443 — generalize it via `declare_inductive`.)
- **Family containment is already static + kernel-backed (§A.5).** Opaque `Cap`
  + driver-only `mint` + downward-only `attenuate` ⟹ a program's reachable
  authority is statically bounded by its declaration; a program that performs an
  FS effect with **no** declared `Cap FS` elaborates to an unbound capability →
  **ill-typed**. This is the primary concern-(1) gate; do **not** re-implement
  it as a runner grant-compare.
- **§A.5.1 RESOLVED → Option (ii).** Authority-*level* sufficiency is **static**
  via a thin **typed capability-API wrapper** (`writeFile : Cap AFull -> …`,
  `readFile : Cap APartial -> …`) over the **UNCHANGED** polymorphic I-3
  producers; an `APartial` program that attempts a write is **ill-typed**. The
  wrapper **signature contract** is defined by §C; §B **implements** the package
  + the static-rejection test. **Do NOT re-type the raw I-3 producers** (that
  reopens the I-3 op surface — guardrail); the wrapper is the non-reopening
  realization.
- **Console is ambient (§A.4).** Unchanged — stdio is process context, no field,
  no cap. Keep the runner/docs caveat; do **not** add a Console capability.
- **Zero-TCB (§A.6).** `Cap`/`mint`/`attenuate`/`ProgramCaps`/`MkProgramCaps` +
  the typed wrapper are ordinary kernel-checked Ken + untrusted driver mint.
  **Zero `trusted_base()` delta** — inherit the I-1/I-2/I-3 before/after
  equality harness as an AC. If the build finds itself adding a trusted
  primitive, **stop and flag the Steward** (contract breach).

## Deliverables — mandated outline (all asserts NAMED-variant, never `is_err`)

1. **Record generalization.** `ProgramCaps` → `ProgramCaps a` /
   `MkProgramCaps (fsCap : Cap a)` (`prelude.rs` `declare_inductive`);
   kernel-checks green; **zero `trusted_base()` delta** (before/after equality
   harness AC).
2. **Mint from the declaration.** The runner mints `Cap a` at the FS authority
   the **program header declares** (§C), wrapped in `MkProgramCaps`, replacing
   the hard-coded `AUTH_PARTIAL` at `main.rs:293-299`. **Depends on §C's
   header capability clause;** a v1 build may read a **minimal declared
   authority** sufficient to light the path while §C's full grammar lands —
   **say so, don't hard-code** (a stale hard-code is exactly the
   perishable-current-state trap). Verify `main.rs:293-299` against the landed
   code at pickup.
3. **Static containment test.** A program that performs an FS effect **without a
   declared FS capability** is **rejected ill-typed** *before any run* —
   assert the **specific missing-capability diagnostic variant** (the
   concern-(1) family gate, §A.5).
4. **End-to-end payoff — the I-3 lit path.** A program declaring FS `AFull`
   (`main : … -> ProgramCaps AFull -> …`) **writes a real file through
   `run_io`** (virtual-FS `CaptureHost` + asserted `FsTrace`) — the I-3 write
   path lit for the first time. Assert the trace’s named write op, not a bare
   success.
5. **Op-time backstop.** `CapabilityDenied` (`IOError`) returned **before**
   syscall on an under-authorized op — asserted on the **specific variant**, no
   crash (§A.5 defense-in-depth).
6. **Console unchanged.** Ambient, no field, no cap (I-2 upheld); caveat in
   runner + docs (§A.4).
7. **Option-(ii) typed capability-API layer + static-level test.** Implement the
   typed wrapper package to §C's signature contract (`writeFile : Cap AFull ->
   …` etc.) over the unchanged polymorphic core, and assert a declared-`APartial`
   program that attempts a **write** is **ill-typed** (static authority-*level*
   rejection — the strongest concern-(1) story). Honor §C's resolved
   attenuation ergonomics (Ken-callable `attenuate` vs authority-polymorphic
   `readFile`). **Do NOT reopen the I-3 producers.**

## Acceptance criteria (testable)

- `ProgramCaps a` generalized; kernel-checks green; **`trusted_base()`
  before==after** (equality harness).
- Mint reads the declared authority (§C surface) — **not** a hard-coded
  `AUTH_PARTIAL`; the v1 minimal-read path is documented as such, not baked.
- Static family-containment: no-cap FS program **rejected** with the **named**
  missing-capability diagnostic (assert the variant).
- I-3 lit: `AFull` program writes a real file via `run_io`; asserted `FsTrace`
  names the write.
- Op-time `CapabilityDenied` backstop returns the **named** variant, no crash.
- Option-(ii): declared-`APartial` write is **ill-typed** (static, named
  diagnostic); the typed wrapper does **not** re-type the raw producers.
- **Validate TARGETED only** (`scripts/ken-cargo -p ken-interp` / `-p ken-cli`
  + affected crate build), **never `--workspace`** — CI owns the locked gate.
  Anchor line numbers verified against landed code at pickup.

## Sequencing

§B **consumes** §C (the enclave lead). **Steps 1, 3, 4 can start now** on a
minimal declared-authority read once §C's minimal surface lands; **step 2's real
mint surface and step 7's typed-wrapper signatures wait on §C**. The Steward
decides at kickoff whether the header **grammar-parser implementation** (parsing
the clause) is a small Language sub-WP or folded into §B step 2. The Architect's
I-4 design (`be52d82b`) is **authoritative** and he is **on-call** for build
design questions.

## Do-not-reopen guardrails

- **No launch-grant / CLI grant-deny / default surface** (concern (2), out of
  scope).
- **Do NOT reopen the I-3 op producers** — the typed API **wraps** them.
- **Do NOT re-decide** Option (ii), the reframe, or coarse-v1 scope (scoped
  least-privilege is I-5).
- **Do NOT add a Console capability** — stdio stays ambient.
- **Zero-TCB is a hard gate** — a trusted-primitive temptation is a contract
  breach; stop and flag the Steward.
