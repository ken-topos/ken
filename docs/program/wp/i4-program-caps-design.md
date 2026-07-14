# WP · Runtime I-4 — ProgramCaps capability surface (Architect design round)

**Owner:** Architect (design) · **Consumers:** Runtime (build follow-on) ·
**Size:** S (design round — largely a transcription of the already-ruled Program
I contract §3.1/§2.1 plus a few open threading mechanics). **Base:**
`origin/main @ 5efa317b`. **Status: READY.**

**This is DESIGN work.** You produce the I-4 design artifact (an Architect
design ruling I transcribe into the Runtime build frame — the **I-2/I-3
pattern** — or a short companion design note / ADR if you judge it warrants
one; deciding **which** is itself a mandated question below). The Runtime build
WP follows from your artifact; I frame it once your design lands. **Read the
authoritative sources from `origin/main`.** Hand the artifact back to me.

Program I's capability model is **already staged and largely ruled** in the
published contract (§3 STAGED; §3.1 v1-coarse). I-4 is the **B.3 "coarse
capability threading" slice** — the design round exists to pin the handful of
threading mechanics the contract left to build time, not to re-open the coarse
model. Where a genuine product fork surfaces (the launch-grant surface), **lay
out options and hand them to me for Pat — do NOT pick.**

---

## Objective

Design the **ProgramCaps capability surface**: how a launched Ken `program`
receives, per-effect-family, the ambient capabilities it is granted (the `Cap`
values threaded into `main`), so that the I-3 FS write/delete path — built but
**not yet lit end-to-end** — actually runs under a `ProgramCaps`-minted `AFull`
grant. Coarse (authority-level) only; the scoped/least-privilege model is I-5.

## Fixed inputs (verify each current-implementation-state claim against landed
code at pickup — the frame lines are perishable)

**Program I contract — the governing design (published, Accepted).**
`docs/program/ken-cli-program-i-contract.md`:

- **§7 decomposition names I-4 exactly:** *"I-4 Coarse capability threading
  (Milestone B.3): `ProgramCaps` mint + per-family caps; write/delete gated
  behind `AFull` with the coarse caveat."* That sentence is the scope
  floor.
- **§1.1 / §1.3** fix the entrypoint ABI I-4 threads into:
  `proc main (input : ProcessInput) (caps : ProgramCaps) : HostIO ExitCode`;
  runner step 3 is *"Mint `ProgramCaps` from the launch grants (§3), one
  capability per granted family."* — the exact seam I-4 realizes.
- **§2.1** rules the `Auth`-index composition tension: **carry each
  capability as a *value* in its op, keep `HostOp` singly-indexed** (do NOT
  lift authority to a per-family tree type index); recommends **(a)
  authority-monomorphic `main`** (checked at the concrete authority the launch
  grant mints — today's mint-exactly discipline) for v1, with (b)
  authority-polymorphic noted as the alternative.
- **§3.1** rules the v1 model: scalar `Authority {None,Partial,Full}` minted
  **exactly** from the launch grant, carried in the op, checked `required ⊑
  granted` **before** the syscall; **the loud caveat** — coarse
  `authorizes(cap, path)` ignores `path`, so v1 write/delete are functional
  but **over-privileged** and ship behind `AFull` with the explicit "coarse
  authority — not path-confined" caveat, NOT advertised as least-privilege.
  **§3.2 (scoped model) is I-5 — out of scope here.**

**Prior floor frames — the ProgramCaps state I-4 inherits (verify at pickup):**

- **I-1** (`docs/program/wp/cli-i1-entrypoint-abi-runner.md`) introduced
  `ProgramCaps` as an ordinary kernel-checked record delivered to `main`,
  carrying the current-authority `Cap` mint, and explicitly deferred **"the
  real per-family threading"** to I-4.
- **I-2** (`docs/program/wp/i2-console-floor.md`, decision #2) ruled **v1
  Console is AMBIENT**: `ProgramCaps` stays `MkProgramCaps (Cap APartial)`
  (FS authority only); Console carries **no** per-op cap; and —
  load-bearing — **do NOT add a coarse no-op Console `ProgramCaps` field**
  (a field that gates nothing is honesty-theater the Architect explicitly
  rejected). Scoped Console authority is a fast-follow.
- **I-3** (`docs/program/wp/i3-fs-floor.md`, D6) built the per-op FS
  authority gate (read-class `APartial`, write/delete-class `AFull`) and
  tested it with a **directly-minted `Cap AFull`**, explicitly **"No
  ProgramCaps threading — that's I-4"**; its sequencing note pins the I-4
  payoff: *"end-to-end 'a Full-granted program writes a real file' lights
  up only when **I-4** threads a Full `ProgramCaps` mint."* **I-4 is what
  makes the I-3 write path reachable end-to-end.**

**Spec — settled capability semantics I-4 must stay faithful to (design, not
re-derivation):**

- **`spec/30-surface/36-effects.md`** — capabilities are **first-class
  static value tokens** (`OQ-8a` DECIDED), threaded by a
  **capability-passing translation**: a function of row `ρ` takes one `Cap
  E` per un-handled effect as an ordinary Π parameter (§2.5, §3); `Cap :
  Effect → Type`. The kernel gains **nothing** — authority is an ordinary
  typed value.
- **`spec/60-security/62-authority.md`** — **no ambient authority** (§1):
  no global `open`; a capability is minted, held, or attenuated-downward
  only; `Cap` is **opaque** (no public introduction form — unforgeability
  is load-bearing). Attenuation is monotone-downward (§3). **Tension to
  reconcile (see questions):** I-2 ruled v1 Console *ambient*, which sits
  against §62's no-ambient discipline — your design must state where that
  boundary honestly falls.
- **ADR 0009** (`docs/adr/0009-capability-supply-strategy.md`) and **ADR
  0004** (security tier-1 / IFC + capabilities as the bracketing substrate)
  are the standing capability/trust ADRs. **There is no dedicated
  `ProgramCaps` ADR** — whether I-4 warrants one is a mandated question
  below.

**Implementation-DAG note.** The Program I "I-series" is **not** the DAG's
`K/V/L/X` spine (`docs/program/05-implementation-dag.md` uses that naming);
I-4 lives in the Program I contract. The DAG substrate I-4 rides is **L5
effects** (the interaction-tree/capability hub) and **Sec2 capabilities**
(`spec/60-security/62`). The DAG table row to keep in view is **Sec2**
("first-class tokens", depends on L5) — I-4 is the *program-entrypoint*
realization of that token discipline for the CLI, at coarse authority.

## Mandated deliverable outline (each question ends in a concrete decision)

The v1-coarse model is ruled; these are the threading mechanics + boundary
calls the contract left to design time. Answer each with a **decision**, not
a survey.

1. **The `ProgramCaps` record shape.** Decide the exact v1 record: which
   per-family fields it carries and which families are ambient. I-2 pinned
   FS-only (`MkProgramCaps (Cap APartial)`), Console ambient, **no no-op
   Console field**. I-4 must decide whether the field stays a single FS cap
   or generalizes to *one field per authority-gated family present at v1* —
   and confirm Env/Process (contract §2.4) are **out of the v1 record**
   (they arrive with I-7), so the v1 record does not manufacture
   honesty-theater fields for families that gate nothing yet. **Decision:
   the exact v1 `ProgramCaps` constructor + field set.**

2. **Minting per-family caps from the launch grant.** Decide how the runner
   turns a launch grant (contract §1.3 step 3, §3.1 "minted exactly") into
   the `ProgramCaps` value: which authority each family's field is minted
   at, and whether the FS field is minted at `AFull` (to light the I-3
   write path) or defaults to `APartial` with `AFull` requested explicitly.
   **Decision: the mint function's input→field mapping and the default
   authority per family.**

3. **Authority-monomorphic vs -polymorphic `main` (contract §2.1).** The
   contract *recommends* (a) authority-monomorphic for v1. Confirm or
   override, and pin the **exact v1 `main` signature** (does `caps :
   ProgramCaps` fix the FS authority concretely, or is `main` polymorphic
   in `a` and instantiated at mint?). Keep `HostOp` singly-indexed either
   way. **Decision: the v1 `main` signature + how the runner instantiates
   authority.**

4. **The ambient-Console vs no-ambient-authority reconciliation.** I-2
   ruled Console ambient; `spec/60-security/62 §1` mandates no ambient
   authority. State **honestly** where v1 sits: is Console genuinely
   ambient at v1 coarse (a CLI process definitionally holds its stdio), and
   if so, what is the documented boundary/caveat and the migration path to a
   Console cap field (the I-2 fast-follow)? **Decision: the v1
   Console-authority posture + its stated caveat, and whether a Console
   `ProgramCaps` field is deferred (I-2 said yes) or lands now.**

5. **Failure mode when a required capability is absent/insufficient.**
   Contract §3.2/§4.1 mandate fail-closed: missing/insufficient capability
   → structured `CapabilityDenied` **value** (not a crash), checked before
   the syscall; the entrypoint type-check already gates `visits ⊆ granted`.
   Decide the I-4 behavior at **grant time** (a program `visits [FS]` but
   launched with no FS grant): a named entrypoint/mint error before the tree
   runs, vs a `CapabilityDenied` at first op. **Decision: the grant-time vs
   op-time failure boundary + the named error.**

6. **Zero-TCB confirmation.** Confirm (as I-1/I-2/I-3 did) that ProgramCaps
   threading adds **no** kernel rule / trusted primitive / postulate — `Cap`,
   `mint`, `ProgramCaps` are ordinary kernel-checked Ken + untrusted driver mint
   code (`capabilities.rs`). **Decision: the zero-`trusted_base()`-delta assertion
   the build frame inherits.**

7. **Artifact form — does I-4 warrant an ADR?** Decide whether this
   design lands as an **Architect design ruling** (an `evt_…` I transcribe,
   the I-2/I-3 pattern) or a **companion design note / new ADR**. The
   coarse model is already in the Program I contract; a new ADR is likely
   overkill, but the ProgramCaps *surface* (record shape + mint + launch
   grants) may deserve a durable home. **Decision: ruling-vs-note-vs-ADR,
   with the reason.**

## SURFACE TO OPERATOR (do NOT pre-decide) — the launch-grant surface

Contract §3.1 says caps are *"minted exactly from the launch grant"* but
does not fix **how the operator expresses that grant at the CLI** — e.g.
default-deny with explicit `--allow-fs-write[=…]` / `--allow-fs-read` flags;
a default-grant posture for an interpreter dev loop; a manifest; or
environment. This is a **product / security-posture fork** (what a user
types to grant a Ken program `AFull` FS authority, and the default when
they type nothing). **Lay out 2–3 concrete options with trade-offs and flag
it as an operator decision — do NOT pick.** I surface it to Pat. (v1 may
ship a minimal grant surface sufficient to light the I-3 path, with the
richer surface deferred — say so if that is your recommendation, still as
an option, not a unilateral pick.)

## Acceptance criteria (for the DESIGN deliverable)

- The artifact **pins each of questions 1–7** with a concrete decision
  (record shape, mint mapping, `main` signature, Console reconciliation +
  caveat, failure boundary + named error, zero-TCB assertion, artifact
  form).
- It **names the Runtime build follow-on WP** (the I-4 build) and states
  its acceptance shape (end-to-end: a Full-granted program writes a real
  file through `run_io` — the I-3 payoff — plus the fail-closed grant/op
  errors, all with **named-variant** asserts per the I-1/I-2/I-3 discipline,
  never `is_err`).
- The launch-grant surface is presented as **≥2 operator options, flagged as an
  operator decision** — no unilateral pick.
- It stays faithful to the settled inputs: coarse-only (no §3.2 scoped
  model), `HostOp` singly-indexed (§2.1), capabilities as opaque value
  tokens (§36/§62), no honesty-theater no-op field (I-2).
- **Design-only — zero code.** No `crates/**`, `spec/`, or
  `conformance/` bytes; the artifact is docs/ruling only. It names every
  WP it feeds.

## Do-not-reopen guardrails

- **Coarse v1 only.** The scoped/least-privilege model (rights × scope ×
  symlink × `openat`-TOCTOU × attenuation) is **I-5** — contract §3.2,
  explicitly out.
- **`HostOp` stays singly-indexed** — carry each cap as a *value* in the op; do
  NOT lift authority to a per-family tree type index (contract §2.1).
- **No honesty-theater fields** — a `ProgramCaps` field that gates nothing does
  not land (I-2 rejected the no-op Console field). Env/Process fields arrive with
  I-7, not here.
- **Zero-TCB** — no kernel rule / trusted primitive / postulate; if the
  design finds itself adding one, that is a contract breach — stop and flag
  the Steward.
- **Do NOT re-open** the entrypoint ABI (I-1), the Console/FS op surfaces
  (I-2/I-3), or the injectable-handler interface (I-6) — I-4 threads
  capabilities through the surfaces those WPs already fixed.

## Note

This design **feeds a Runtime build WP** (the I-4 build, which lights the I-3
write path end-to-end); Foundation #60 follows Runtime per the implementation
tracker. The Runtime ring is idle awaiting it. The Program I contract is
**current, not stale** — I-4 realizes its §3.1/§2.1 rulings, it does not
re-derive them; the only genuinely open surface is the operator-facing launch
grant, which comes back to me as options.
