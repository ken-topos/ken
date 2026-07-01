# Agentic-boundary conformance — seed cases (B4, WS-B capstone)

Format: `../../README.md`. These pin the **reduction** by which assuring an
embedded agent's outputs is **nothing new** (`spec/70-behavioral/74-agentic.md`,
impl-ready B4): an embedded agent is a **maximally-nondeterministic input** —
the strongest assumption `P` in the `71` export — so "assure the agent" is the
existing seam (`61`/`62` + `71`/`72`/`73` + the `OQ-relational` path) aimed at
its most adversarial environment. **Ken adds no agentic mechanism** — no kernel
rule, no judgment, no "agent" surface. These cases net the **four-way status
partition** (`74 §2.1`): each of the four assurances one can offer an
agent-bearing system lands **one** status and **one** export field, and the
"never" column is normative.

Grounding (landed `§`-bodies + landed code on `origin/main`, content-reconciled
— not the frame): `74 §1`–`§7` (the reduction, the partition, the three faces,
the honesty boundary); `71 §2.1`/`§5`/`§5.1` (the export projection `Q`/`P`/`T`,
assume-guarantee soundness, the one-way gate I4); `21 §5.2`/`§5.4` (the four-way
epistemic status + the honesty guard); `62 §1`/`§2`/`§H` (no-ambient authority,
`Cap E` as a real Π value, the kernel-backed-vs-trusted split); `36 §2.5`/`§3.1`
(capability-passing translation, the `Vis`/`Σ` substrate); `61 §5.3` (the
by-proof relational path + its `[rel-deferred]` product-program reducer);
`72 §5` (`Temporal`↦`delegated`↦`T`, total/constant); `73 §2.1`/`§2.4`/`§2.6`
(trace events as witnesses, the monitor-is-image-of-`T` projection, the one-way
gate TC5); `24 §2` (a typed hole is a postulate); `64 §4` (over-claim refusal).
**Landed code pinned against:** the export emitter
(`crates/ken-elaborator/src/export.rs`); `CapParam { name, effect }` + `cap_set`
(`effects/algebra.rs`), no-ambient inertness of a no-`Cap` `view`;
`trusted_base` membership (`ken-kernel`); the single `Vis` trace site
(`ken-interp`); the `Temporal` inductive via the L2 `data` machinery.
**Named-deferred (carried, not driven):** the `OQ-relational` 2-run
product-program reducer (`[rel-deferred]` — `ifc.rs` `TRIGGER_REL_DEFERRED`,
hand-simulated in `sec1_acceptance.rs`); L2 test-gen (no landed producer); the
live Büchi monitor (`compile` faithfulness, `(oracle)`/B2).

## Reading these cases — the B4-specific disciplines

**B4 is a projection/partition-fidelity corpus, not an engine-execution one
(`74 §2.1`, the producer-grep honesty).** The observable is the **projected
status/field** (through the real `71` export emitter), a **`trusted_base()`
membership**, or a **`Cap E`/no-ambient elaboration verdict** — **never** the
run of a discharge engine. Each AC drives the **landed projection** (the same
static face `71` already fixes); the engines that would later *close* a
`tested`/`delegated` obligation — the `OQ-relational` reducer
(`[rel-deferred]`), L2 test-gen, the live Büchi monitor (`(oracle)`/B2) — are
**named-deferred runtime faces carried as triggers, not asserted as real**. An
AC that drove a deferred engine would be the hand-feeds-the-deliverable trap —
the highest-risk gate in the corpus, because B4 is a doc/composition WP.
**AC1/AC2/AC5 are fully-landed** (export projection, the real `Cap E`/no-ambient
flip, the `trusted_base()` flip); **AC3/AC4 pin the landed projection and carry
their engines as named triggers** — the static-vs-runtime-face split, mirroring
`../../security/ifc/seed-ifc.md` D3/D4/G1 and `../trace/seed-trace.md` TR-E.

**The agent is a source of `P`, never of `Q` — the honesty direction.** An agent
output is an untrusted value; it enters exactly as the assumption boundary `P`
models a nondeterministic environment (`74 §2`, `71 §2.1`). Ken's theorem is the
conditional "given `P`, then `Q`", kernel-checked regardless of how `P` is
realized; setting `P` = the agent's whole output domain instantiates it at
maximal adversariality. The load-bearing net across AC2b/AC3/AC5 is: **no agent
output ever lands in `Q`** (quality, relational consistency, or the raw
proposal). A case that let an agent-derived claim reach `Q` is the over-claim
the export must refuse.

**No downstream verdict ever promotes to `proved` — the one-way direction
(`71 §5.1` I4, `73 §2.6` TC5).** A green metamorphic run, a green monitor, a
`Ward` verdict — **none** re-enters Ken as `proved`. The envelope's `proved` is
the *only* `Q`, and it is `Q` because the kernel checked its certificate
(`21 §5.4`), **not** because a downstream engine went green. AC4 pins this
structurally.

**Kernel-backed vs trusted-by-typing — do not over-claim the envelope's `Q`
(`74 §3`, `62 §H`).** The envelope's `proved`/`Q` is the **invariant discharge +
capability confinement** half: `Cap E` is a **real Π value** (`62 §2`,
`36 §2.5`) and the `safe`-gate is a `22`/`23` obligation the kernel
**re-checks** — **kernel-backed**. Where the envelope *also* uses IFC labels to
confine data flow (`61`), that flow rule is **trusted-by-typing** (labels erased
before the kernel, `61 §9 N1`/`62 §3.1`), so a label-mediated confinement
projects to `P`/`tested`, **never** `Q`. AC2's `proved`/`Q` assertion is the
capability + `safe`-gate half only; filing a label-mediated guarantee as
kernel-certified would over-claim (`61 §H`).

**QA gate — real landed producer → real projection; grep the producer src, not
the test.** Every case routes a real checked program through the **actual**
producer and observes the projected result (or a `trusted_base()`/elaboration
fact). **AC2** must reach the real `Cap E`/no-ambient path (`62 §1`), never an
`isAgent` boolean; **AC3** the real `tested`→`P` projection through the emitter
(the 2-run reducer is `[rel-deferred]` — carry the trigger, do **not**
hand-simulate it as "real"); **AC4** the real `delegated`→`T` projection + the
`Vis`/trace site (the live-monitor catch is `(oracle)`/B2 — carry the trigger,
not a hand-built rejection); **AC5** the real `trusted_base()` membership, never
a status string. A synthetic agentic literal that re-validates a pre-existing
consumer guards nothing and is not a conforming case.

**Tags.** `(soundness)` = a real honesty/one-way commitment of the seam trust
model that must never regress — the agent-never-`Q` direction (AC5, AC2b) and
the never-promote-to-`proved` direction (AC4), netted **solely** by conformance
(the kernel is blind to the export bytes, and the erased-before-kernel `P`/label
posture). `[rel-deferred]` = the `OQ-relational` 2-run product-program reducer,
named not built (`61 §5.3`); a case carrying it drives the projection, not the
reducer. `(oracle)` = the live Büchi monitor / `compile` faithfulness (B2), and
any literal serialized export field key (finalized by `Ward`) — referred to by
concept, spelling deferred.

## AG-A. The reduction — agent → `P`, invariant → `Q` (AC1)

### agentic/scenario-projects-agent-to-P-invariant-to-Q (AC1)
- spec: `74 §2`/`§2.2`/`§7 AC1`, `71 §2.1`/`§5`, `21 §5.2`
- given: the envelope scenario routed through the **real** `71` export emitter —
  an agent proposal `a : Proposal` bound as an ordinary (universally quantified)
  parameter, a verified validator
  `act_on (c : Cap E) (a : Proposal) : Unit visits [E] requires safe a`, and a
  system invariant `I` proved **quantified over `a`** without any hypothesis
  about `a` (`74 §2.2`)
- expect: through the landed projection, the invariant `I` lands in **`Q`**
  (concept `guarantees`; status `proved`; **absent** from `trusted_base()`, its
  certificate `check`s) **while** the agent output `a` rides **`P`** (the
  maximal nondeterminism assumption) — **never** `Q`
- why: AC1 — the reduction is the landed `71` export at `P` = maximal
  nondeterminism; "given `P`, then `Q`" read at its most adversarial, no new
  mechanism. **Alone it is green-vs-green** (a sensible emitter also passes);
  the net is the pair with AC2's flip + AC5's agent-never-`Q` + the cross-case
  sweep. Drives the real `export.rs` projection, not a synthetic agentic
  literal.

## AG-B. The safety envelope — propose/act, the flip (AC2, soundness)

### agentic/agent-holds-propose-invariant-proved-for-all (AC2)
- spec: `74 §3`/`§7 AC2`, `62 §1`/`§2`, `36 §2.5`, `22 §2`
- given: an agent `view` with **no `Cap E` parameter** (inert by no-ambient
  authority, `62 §1` — it computes a proposal *value* and nothing else); the
  verified `act_on` above holds `act` and performs the world-effect **only**
  behind `requires safe a`; the system invariant `I` discharged as an
  `ensures`/space-invariant obligation quantified over `a` (the only typed path
  to `perform_E` runs through the `safe` gate)
- expect: `I` discharges **`proved`** → **`Q`**, **kernel-backed** (the `Cap E`
  confinement is a real Π value, `62 §H`; the `safe`-gate discharge is a
  `22`/`23` obligation the kernel re-checks). **Accept.**
- why: AC2 half — the invariant is proved *for all* agent outputs because the
  agent is confined to *proposing*. The `proved`/`Q` is the capability +
  `safe`-gate half (kernel-backed), **not** any label-mediated flow (`74 §3`,
  the honest split). Alone green-vs-green; netted by the flip below.

### agentic/agent-handed-act-invariant-not-derivable-rejects (AC2, soundness)
- spec: `74 §3`/`§7 AC2`, `62 §1`/`§2`
- given: the **same** scenario, but the agent is handed the **`act`** capability
  `Cap E` directly (a world-effect now reachable **without** the `safe` gate)
- expect: the invariant `I` is **no longer derivable for all `a`** (an un-gated
  `perform_E` escapes the envelope) — the obligation **does not discharge**.
  **Reject.**
- why: (soundness) AC2 — the load-bearing **verdict-flip pair**. The flip is on
  **exactly** the propose/act capability boundary — the real `Cap E`/no-ambient
  check (`62 §1`), a **structural** discriminator that flips accept↔reject,
  never an `isAgent` string. A single accept case is green-vs-green; only the
  pair (one scenario, two capability states) pins that the envelope's `proved`
  rests on confinement. Drives the real Sec2 machinery.

## AG-C. The metamorphic face — `tested`→`P`, engine deferred (AC3)

### agentic/relational-over-agent-projects-tested-never-proved (AC3)
- spec: `74 §4`/`§7 AC3`, `61 §5.3`, `71 §2.1`, `21 §5.2`
- given: an oracle-free relational property (e.g. permutation-invariance
  `f (perm xs) ≈ f xs`, or round-trip) stated over a **nondeterministic (agent /
  maximal-`P`) producer**, routed through the **real** `71` export emitter
- expect: the relational obligation is classified **`tested`** → **`P`**
  (concept `assumptions`), **never** `proved`/`Q` (the agent is maximal `P`, not
  a pure Ken function — no static certificate exists). The 2-run discharge
  carries a **`[rel-deferred]`** reify-trigger (the `OQ-relational`
  product-program reducer, `61 §5.3`, "named not faked") and the test-gen
  sampling is a named-deferred runtime face (no landed producer) — **neither is
  driven**
- why: AC3 — the **landed** face is the `tested`→`P` projection through the real
  emitter (the same static face AC1/AC5 drive). The bug it targets is the
  **over-claim**: a lazy emitter that marks the relational obligation
  `proved`/`Q` ("assume the relation holds") lands an agent-derived claim in `Q`
  → **red** (it violates the agent-never-`Q` honesty direction). Correct =
  `tested`/`P` + the `[rel-deferred]` trigger present; the flip is on the
  honesty axis, structurally pinned (status + trigger + `Q`-absence). The Sec1
  precedent is `seed-ifc` D3/D4/G1 — pin the verdict-mapping, carry
  `[rel-deferred]`, drive no reducer. (The `OQ-relational` *by-proof* mode —
  `proved`/`disproved`/`unknown` — applies only to a **pure Ken function**, not
  the agent; `36 §3` is the `Vis` substrate, **not** the metamorphic mechanism —
  `74 §4`.)

## AG-D. The RV face — `delegated`→`T`, monitor deferred (AC4, soundness)

### agentic/agent-temporal-delegated-never-promotes (AC4, soundness)
- spec: `74 §5`/`§7 AC4`, `72 §5`, `73 §2.1`/`§2.4`/`§2.6`, `71 §5.1`
- given: an agent-action temporal obligation — a `temporal { … }` block over the
  agent's `Vis`-action events (the agent's authority-relevant actions are
  already events in `Σ`, `62 §5`/`36 §3.1`) — routed through the **real** export
  emitter
- expect: it lands in **`T`** (concept `obligations`) tagged **`delegated`**
  (`72 §5`: `Temporal`↦`delegated`↦`T`, total/constant — never
  `proved`/`tested`/`unknown`/`Q`/`P`); the monitor is the **image of `T`**
  (monitor-is-image-of-`T`, changes with `T`, `73 §2.4`); a trace event is a
  **witness** carrying no status; and the `delegated` entry **never promotes**
  to `proved` — a green monitor/trace is evidence *for* the `delegated` `T`,
  never a `proved` (`71 §5.1` I4, `73 §2.6` TC5, the one-way gate). The **live
  monitor *rejecting* a violating trace** (Büchi acceptance / `compile`
  faithfulness) is the named-deferred **`(oracle)`/B2** runtime face — carried,
  not driven
- why: (soundness) AC4 — the **landed** face is the `delegated`→`T` projection +
  monitor-is-image-of-`T` + the trace-witness/one-way gate (real `export.rs` +
  the `Vis`/trace site). The bug it targets is a **promotion path**: any code
  that re-entered a green monitor verdict as `proved`/`Q` → **red** (the one-way
  gate). Drives the real `Σ`/trace instrumentation, never a hand-built event
  log; the live-catch is `(oracle)`/B2 (`../trace/seed-trace.md` TR-E), carried
  as the deferred face.

## AG-E. The honesty boundary — quality → `unknown` (AC5, soundness)

### agentic/quality-maps-to-unknown-not-dischargeable (AC5, soundness)
- spec: `74 §6`/`§7 AC5`, `21 §5.4`, `24 §2`, `64 §4`
- given: a **quality** obligation — "the summary is *faithful*", "the plan is
  *wise*" — authored as a spec claim over an agent output
- expect: it elaborates to a **typed hole = postulate** of its goal (`24 §2`),
  **stays in `trusted_base()`**, status **`unknown`** (rides `P` as a hole);
  **never** `proved`/`Q`; and it is **not dischargeable** by any Ken mechanism
  (there is no `φ` — no propositional oracle — to discharge)
- why: (soundness) AC5 — output quality has no oracle, so the honest status is
  `unknown`. Half of the honesty pair.

### agentic/safety-obligation-proved-absent-from-trusted-base (AC5, soundness)
- spec: `74 §6`/`§7 AC5`, `21 §5.4`, `71 §2.1`
- given: a genuine **safety** obligation (the envelope invariant `I`) with a
  discharged certificate, run through the same emitter
- expect: it lands **`proved`** → **`Q`**, **absent** from `trusted_base()`, its
  certificate kernel-re-checked
- why: (soundness) AC5 — the load-bearing **honesty flip**. The discriminator is
  **`trusted_base()` membership** on the same emitter path: the quality goal is
  a postulate *present* with **no** certificate (→ `unknown`, never `Q`)
  **while** the safety goal is *absent* with a certificate (→ `proved`/`Q`) — a
  structural flip (`21 §5.4`), not a status string. This is the safety-≠-quality
  line made testable; a case that fed a `proved` label onto a quality claim must
  be rejected — no code path promotes it (the §2.1 no-leak, `71 §5.1`). Same
  discriminator as `../export/seed-export.md` EX-A, on the quality axis.

## Coverage map

| Case | AC | Spec | Status→field | Landed / deferred face |
|---|---|---|---|---|
| `scenario-projects-agent-to-P-invariant-to-Q` | AC1 | `74 §2` | invariant→`Q`, agent→`P` | landed (export projection) |
| `agent-holds-propose-invariant-proved-for-all` | AC2 | `74 §3` | `proved`→`Q` | landed (real `Cap E`) |
| `agent-handed-act-invariant-not-derivable-rejects` | AC2 | `74 §3` | reject (flip) | landed (real no-ambient) |
| `relational-over-agent-projects-tested-never-proved` | AC3 | `74 §4` | `tested`→`P` | landed projection; `[rel-deferred]`+test-gen deferred |
| `agent-temporal-delegated-never-promotes` | AC4 | `74 §5` | `delegated`→`T` | landed projection; live monitor `(oracle)`/B2 |
| `quality-maps-to-unknown-not-dischargeable` | AC5 | `74 §6` | `unknown` (hole in `P`) | landed (`trusted_base()`) |
| `safety-obligation-proved-absent-from-trusted-base` | AC5 | `74 §6` | `proved`→`Q` | landed (`trusted_base()`) |

## Cross-case sweep (the §2.1 status partition)

The cases group by the four-way partition and must **agree**: {envelope →
`proved`/`Q`} (AC1, AC2-propose) · {metamorphic → `tested`/`P`} (AC3) · {RV →
`delegated`/`T`} (AC4) · {quality → `unknown`} (AC5-quality). Two boundary
invariants are pinned across the groups:

- **No agent output ever lands in `Q`** (the honesty direction) — AC5-quality
  (`unknown`, never `Q`), AC3 (relational over the agent → `tested`, never `Q`),
  AC2-flip (agent handed `act` → reject, no proved-for-all). The agent's outputs
  — quality, relational consistency, or the raw proposal — never reach `Q`.
- **No downstream verdict ever lands in `proved`** (the one-way direction) — AC4
  (a green monitor never promotes), AC3 (a test-gen run never promotes). A green
  engine never re-enters as `proved`; the only `Q` is the kernel-checked
  envelope certificate.

A corpus that let AC5-quality reach `Q`, or AC4's monitor promote to `proved`,
would contradict the partition — the two invariants are the load-bearing
structural nets (the kernel is blind to the export bytes, so conformance is the
sole net, the B1 N1 / Sec1 N1 posture).

## Subsumed, not duplicated

These cases **compose** upstream producers; they do not re-net upstream meaning:
the `Q`/`P`/`T` status→field projection itself is `../export/seed-export.md`
(B1); the `Cap E`/no-ambient/attenuation authority discipline is
`../../security/capabilities/seed-capabilities.md` (Sec2); the
`Temporal`→`delegated`→`T` flow + monitor-is-image-of-`T` is
`../temporal/seed-temporal.md` / `../trace/seed-trace.md` (B2/B3); the by-proof
relational verdict-mapping + `[rel-deferred]` trigger is
`../../security/ifc/seed-ifc.md` D-series (Sec1). B4 nets only the **agentic
reduction** — that these landed producers, aimed at a maximally-nondeterministic
component, partition the four-way status with no residue and no leak.

## Build-sequencing note (Team Kernel — light, composition)

B4 adds **no** kernel rule, judgment, or "agent" surface (grep the kernel and
elaborator for an agentic former — **none**). The build **composes** the landed
producers to satisfy AC1–AC5: it drives the real `export.rs` projection, the
real `Cap E`/no-ambient check, the real `trusted_base()` membership, and the
real `Vis`/trace site. The discharge engines the corpus **carries as triggers**
— the `OQ-relational` 2-run reducer (`[rel-deferred]`), L2 test-gen, and the
live Büchi monitor (`(oracle)`/B2) — are **not** in scope here; they close the
`tested`/`delegated` obligations in later WPs and are named, not faked. WS-B
**completes on merge** (owned by Team Verify, per the WS-Sec/B routing).
