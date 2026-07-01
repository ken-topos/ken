# Assuring agentic outputs — the boundary

> Status: **impl-ready (B4)** — the **WS-B capstone**. Normative for the
> **reduction** by which assuring an embedded agent's outputs is *nothing new*:
> it is the existing seam (`61`/`62` + `71`/`72`/`73` + the `OQ-relational`
> 2-run path) aimed at a **maximally-nondeterministic component**. **Ken adds no
> agentic mechanism** — no kernel rule, no new judgment, no "agent" elaborator
> surface. **`OQ-agentic-oracle` DECIDED** (operator, 2026-06-27; ADR 0006). The
> deliverable of this chapter is the reduction itself, pinned tightly enough
> that conformance (`../../conformance/behavioral/agentic/`) drives the **real**
> producers (Sec2 caps, B1 export, B2 Temporal, B3 trace, the relational path),
> never a synthetic agentic literal.
>
> **Perishable — pin against the *landed* Sec1/Sec2/B1/B2/B3 code, not this
> banner.** The reduction rests on machinery that is already merged: the
> capability discipline (`62`, `crates/ken-elaborator/src/effects/`, `ifc.rs`),
> the five-field export projection (`71`, `ken-elaborator/.../export.rs`),
> the `Σ`/trace instrumentation (`73 §2`, the single `Vis` site in
> `ken-interp`), and the `OQ-relational` product-program reduction (`61 §5.3`).
> B4 **composes** these; it builds no new primitive. **If a build finds itself
> writing a new mechanism, it has mis-scoped — stop and flag the Steward.**

## 1. The problem, and why it is not a new Ken mechanism

A Ken-built system may **embed an agent** at runtime (an LLM proposes a plan, a
query, a tool call, a config). Such outputs have **no propositional oracle** —
you cannot write a spec `φ` that says "this generated summary is correct." The
question "what assurance is possible?" looks like it needs new machinery. It
does not. The decisive reframe — the same one that runs through the whole
project:

> You do not trust the agent; you verify the **boundary**. At *authoring* time
> the boundary is the kernel (re-check the proof). At *runtime* the boundary is
> a verified envelope (constrain the action). Same de-Bruijn-criterion spirit at
> both layers: **untrusted producer, verified check.**

Structurally, **an embedded agent is just a maximally-nondeterministic input** —
the strongest case of an assumption `P` in the export (`71 §2`). Ken already
proves systems safe *for all values* of a nondeterministic input; the agent is
that input at its most adversarial. So the problem **reduces to the existing
seam**, pointed at the most adversarial environment. The rest of this chapter
makes that reduction precise (§2), demonstrates its three faces (§3–§5), and
draws the honesty line it must not cross (§6).

## 2. The reduction, formally

**The agent is a source of `P`, never of `Q`.** An agent output is an *untrusted
value* — it enters Ken exactly as the assumption boundary `P` already models a
nondeterministic environment (`71 §2.1`: `tested`/`unknown` claims ride `P`,
never `Q`). Ken's theorems are conditional — "**given `P`, then `Q`**" — and
that implication is intuitionistically valid and **kernel-checked regardless of
how `P` is later realized** (`71 §5`, soundness-by-assume-guarantee). Setting
`P` to *the agent's entire output domain* (maximal nondeterminism) instantiates
the theorem at its most adversarial: a `Q` proved "for all `P`" is, in
particular, proved for every value the agent can emit. **No new theorem, no new
rule — one instantiation of the landed export contract.**

### 2.1 The status partition — the spine (verdict-mapping pinned here)

The four assurances one *can* offer an agent-bearing system partition
**exactly** onto the four-way epistemic status (`../20-verification/21 §5.2`:
`proved`/`tested`/`delegated`/`unknown`). Each mechanism lands **one** status
and **one** export field, and the "never" column is normative — it is the
boundary a conformance author must not fill differently (the verdict-mapping
silence is the highest-risk kind; it is closed here at the source):

| Assurance (mechanism) | Status | Field | Never | Grounded in |
|---|---|---|---|---|
| **Safety envelope** — invariant proved for *all* agent outputs | **`proved`** | **`Q`** | never `tested`/`unknown` | `62` caps + `22`/`23` obligation → `71` `Q` (§3) |
| **Metamorphic relation** — oracle-free relational check across runs | **`tested`** | **`P`** | never `proved` (agent is `P`, not a pure fn) | `OQ-relational` `61 §5.3` + L2 test-gen (§4) |
| **RV watchdog** — a temporal/behavioral obligation over agent actions | **`delegated`** | **`T`** | never `proved`/`tested`/`unknown` | `72`/`73`; `72 §5`: `Temporal`↦`delegated`↦`T`, total (§5) |
| **Output quality** — "the summary is faithful / the plan is wise" | **`unknown`** | *(hole in `P`)* | never `proved`/`Q`; **not dischargeable** | no oracle; `21 §5.4` honesty guard (§6) |

Two properties make this a *partition*, not a menu:

- **It is total.** Every assurance question about an agent output is one of
  these four rows — a safety invariant (proved), a relational consistency
  (tested), a temporal/ordering property (delegated), or a quality judgment
  (unknown). There is no fifth kind that would need a new mechanism.
- **The rows do not leak.** The one-way gate (`71 §5.1`, invariant I4; `73 §2.6`
  TC5) forbids any promotion path: a green metamorphic run, a green monitor, a
  `Ward` verdict — none re-enters Ken as `proved`. The envelope's `proved` is
  the *only* `Q`, and it is `Q` because the kernel checked its certificate
  (`21 §5.4`), **not** because a downstream engine went green.

**The reduction is the claim that these four rows are the whole story** — that
"assure the agent" decomposes with no residue into {prove the envelope, relate
the runs, watch the actions, and *honestly decline* quality}. §3–§6 pin one
conformance obligation per row; §6 pins the fourth as the boundary.

### 2.2 The proof shape — invariant independent of the agent

The envelope's guarantee is a proposition proved **quantified over the agent's
output**, i.e. with the agent output bound as an ordinary parameter and never
inspected by the proof:

```
-- schematic; the agent output `a : Proposal` is universally quantified,
-- and the safety invariant `I` is proved WITHOUT a hypothesis about `a`.
prove envelope_safe :
  (a : Proposal) → I (step_of_validator a)
```

Because `a` is bound but the proof of `I` does not rely on any property of `a`
(only on the *validator's* gate, §3), the guarantee holds **for all** `a` —
including every adversarial agent output. This is the ordinary assume-guarantee
shape (`71 §5`), not a new modality: `a` is the `P`, `I` is the `Q`, and `∀a. I`
is the kernel-checked conditional read at maximal `P`.

## 3. The safety envelope — propose/act, proved for all outputs

The first face is the **verified shield**: the system stays safe *whatever* the
agent emits, because the agent is confined by capabilities (`62`) to
*proposing*, and a **verified validator** holds the authority to *act*.

- **The agent holds only `propose`.** By no-ambient authority (`62 §1`, the L5
  capability-passing translation, `36 §2.5`), a `view` with no `Cap E` parameter
  is **inert by its type** — it can compute a proposal *value* and nothing else.
  The agent's "output" is data, not an effect.
- **A verified validator holds `act`.** The `act` capability `Cap E` gates the
  world-effect (`62 §2`). The validator performs the effect **only** behind a
  checked precondition:

```
view act_on (c : Cap E) (a : Proposal) : Unit  visits [E]
  requires safe a                 -- the gate: only safe proposals are enacted
  = perform_E (enact a)
```

- **The invariant is `proved` for all agent outputs.** Because the *only* typed
  path to `perform_E` runs through `requires safe a` (no-ambient: the agent
  cannot `perform` — it lacks `Cap E`), the system invariant `I` is discharged
  as an ordinary `ensures`/space-invariant obligation (`21 §5`, `22 §2`),
  **quantified over `a`** (§2.2). It lands `proved` → `Q` (`71 §2.1`), and it is
  **kernel-backed**: no-ambient confinement is kernel-backed (the cap is a real
  Π value, `62 §H`) and the `safe`-gate discharge is a `22`/`23` obligation the
  kernel re-checks.

**Honest trust-level split (do not over-claim).** The invariant's discharge and
the *capability* confinement are **kernel-backed** (`62 §H`). Where the envelope
*also* uses IFC labels to confine data flow (`61`), that flow rule is
**trusted-by-typing** (labels erased before the kernel, `61 §9 N1`/`62 §3.1`)
— so a label-mediated confinement projects to `P`/`tested`, **never** `Q`. The
`proved`/`Q` headline is the invariant + capability half; the label half is the
Sec1 posture, stated as such. (Filing a label-mediated guarantee as
kernel-certified would over-claim — the `61 §H` erasure boundary.)

**AC1 / AC2 — the demonstration and the flip.** AC1: a conformance case routes
an agentic scenario through the **real** `71` export, showing the agent output
projected into `P` (maximal nondeterminism) and the invariant into `Q` — not a
new mechanism, the landed projection. **AC2 (the flip):** the invariant is
`proved` for-all outputs *when the agent holds only `propose`*; the **verdict
flips to reject** when the agent is instead handed `act` — then a world-effect
is reachable without the `safe` gate, `I` is no longer derivable for all `a`,
and the obligation no longer discharges. The flip is on **exactly** the
propose/act capability boundary, driving the **real** Sec2 `Cap E` / no-ambient
machinery (`62 §1`), never a synthetic "isAgent" flag.

## 4. Metamorphic relations — oracle-free, `tested`

The second face assures **relational consistency** without any ground-truth
oracle: even with no `φ` that says an output is *correct*, one can assert a
**relation between runs** of the producer — round-trip
(`decode (encode x) = x`), permutation-invariance (`f (perm xs) ≈ f xs`),
monotonicity — and check it by running the producer, not by consulting an answer
key.

**Mechanism (grounded — note the sourcing).** A relation-between-runs is the Ken
**2-run / relational** property, decided as `OQ-relational` (`61 §5.3`): the two
runs are renamed to disjoint copies and reduced, by a **product program**, to a
**single unary obligation** the prover discharges and the kernel re-checks
(`61 §5.3`). The events the runs range over are the interaction-tree `Vis`
signatures — the same `Σ` substrate the effect discipline already fixes
(`36 §3.1`, `71 §2`). L2 **test-gen** produces the input pairs/permutations that
exercise the relation. *(The "metamorphic" framing and the named relations are
this chapter's; the underlying machinery is `OQ-relational` + test-gen — `36 §3`
is the capability/`Vis` substrate the runs ride, **not** a metamorphic source.
Do not cite `36 §3` as the metamorphic mechanism.)*

**Verdict-mapping (pinned).** Over an **agent** producer the relation lands
**`tested`**, never `proved`: the agent is maximal `P` (§2), not a pure Ken
function, so no static certificate exists — the relation is *exercised* across
sampled runs (test-gen), an honest `tested` obligation in `P` (`21 §5.2`). Only
when a relation is asserted of a **pure Ken function** does the `OQ-relational`
*by-proof* mode apply and inherit the
`proved`/`disproved`-with-witness/`unknown` trichotomy (`61 §5.3`) — but the
agent's *output quality* stays `unknown` regardless (§6). B4's AC3 is the
agentic case: `tested`.

**AC3.** An oracle-free relational check (e.g. permutation-invariance)
**passes** on a nondeterministic producer, driven by the **real**
`OQ-relational` 2-run reduction + L2 test-gen over the real `Σ`/`Vis` events —
not a synthetic assertion over a canned output. The case observes the relation
holding across the generated run-pairs and the claim carried as `tested` (never
`proved`).

## 5. RV watchdogs — agent actions as `Σ`-events, `delegated`

The third face watches the agent's **observable actions** at runtime. Each
authority-relevant action is a `Vis` node the type already declares (`62 §5`,
`36 §3.1`) — so the agent's actions are **already** events in the alphabet `Σ`
(`72 §3`: `Temporal` ranges over the perform-node signatures; `73 §2.1`: the
monitored vocabulary *is* the interaction tree's `perform` nodes). Nothing new
is instrumented.

- **State the obligation.** A temporal/behavioral property over those events —
  an ordering, a never-claim, an eventual-consistency property — is stated in
  source as a `Temporal` value (a `temporal { … }` block, `72 §4`). Its status
  is **`delegated`**, its field **`T`** — total and constant (`72 §5`:
  `Temporal`-in-source ↦ `delegated` ↦ `T`, **never** `proved`/`tested`/
  `unknown`).
- **Synthesize the monitor.** `T` projects to a runtime monitor via
  `compile : Temporal Σ → Monitor` (`73 §2.4`; a *projection* of the export's
  `T`, not a re-authored property — distinct from `71`'s `→ WardFormula`). The
  monitor reads exactly the `73 §2.1` trace events (same alphabet).
- **Catch the violation.** A monitor that **rejects** a live trace signals a
  conformance violation (`73 §3`): the agent's actions left the model's allowed
  behaviors, or a boundary `P` it relied on was breached. Crucially this is
  **not a re-verdict of `Q`** — Ken's theorem is the conditional "given `P`,
  then `Q`"; a runtime violation is an assumption-side (`P`) failure, not a
  refutation of the kernel certificate (`73 §2.3`/`§3`). And it **never
  promotes**: a green monitor is evidence *for* a `delegated` `T`, never a
  `proved` (`72 §5` I4; `73 §2.6` TC5, the one-way gate).

**AC4.** An agent-action `T` obligation is monitored **end-to-end** via the
**real** B2 Temporal + B3 trace (source `temporal` → `T` → `compile` → monitor →
trace), and a **violating action is caught** (the monitor rejects). The case
drives the real `Σ`/trace instrumentation (the single `Vis` site in
`ken-interp`, `73 §2`), not a hand-built event log; and it asserts the caught
violation stays `delegated` (no promotion).

## 6. The honesty boundary — safety, never quality

This chapter assures **safety, structural validity, and relational consistency**
— never **quality**. Ken/`Ward` can prove the agent *cannot harm* (escape the
envelope, exceed authority, violate an invariant — §3; break a relation — §4;
violate a temporal obligation — §5). They **cannot** prove the agent is *good* —
that the summary is faithful, the plan wise. Output quality has **no
propositional oracle** (§1), so:

- it **cannot** be `proved` (no certificate — there is no `φ` to discharge);
- it **cannot** be `tested` (a `tested` obligation needs a checkable relation or
  runtime assertion; "faithful" has none — that is the very absence of an
  oracle);
- it **cannot** be `delegated` (it is not a temporal/behavioral property over
  `Σ`);
- so it is **`unknown`** — the honest "no oracle, not established" (`21 §5.2`):
  were one to *write* a quality obligation, it would elaborate to a **typed hole
  = postulate** of its goal (`21 §5`, `24 §2`), which **stays in
  `trusted_base()`** and is **never dischargeable** by any Ken mechanism.

The four-way status keeps this legible (`21 §5`):

> the **envelope** is `proved`; the **agent output** is `tested` / `delegated` /
> `unknown` — **never `proved`.**

Claiming more would be the dishonesty a verified language must refuse: **"a
verified language that over-claims is itself a security risk"** (`64 §4`; the
spec≠intent residual, `64 §4.1` — the kernel proves code-matches-spec, never
that the spec, or the agent's output, is *right*).

**AC5 (soundness — the honesty guard).** A **quality** obligation maps to
`unknown`, **never** `proved`/`Q`, and is **not dischargeable**. The
discriminator is the `21 §5.4` honesty guard applied structurally: the quality
goal sits as a postulate in `trusted_base()` with **no** certificate — so it
**flips** against a genuinely proved safety obligation (which is *absent* from
`trusted_base()` and carries a kernel-checked certificate). A test that fed a
`proved` label onto a quality claim must be rejected: no code path promotes it
(the §2.1 partition's no-leak property; `71 §5.1`). This is the safety-≠-quality
line made testable.

## 7. What B4 delivers, and its acceptance

B4 is a **composition** WP: spec + conformance establishing the reduction over
**already-landed** machinery. It adds **no** kernel rule, **no** new judgment,
**no** "agent" elaborator surface (the structural absence — grep the kernel and
elaborator for a new agentic former and find **none**; the assurance is the
existing seam at maximal `P`). The implementable deliverables:

1. **The reduction + status partition (§2).** Agent-output-assurance ≡ the `71`
   export at `P` = maximal nondeterminism; the four assurances partition the
   four-way status (§2.1), verdict-mapping pinned, no-leak by the one-way gate.
2. **The safety envelope (§3).** Propose/act capability split (`62`), invariant
   `proved` for-all outputs → `Q`, with the honest kernel-backed-vs-trusted
   split stated.
3. **The metamorphic face (§4).** Oracle-free relational check → `tested`, via
   `OQ-relational` (`61 §5.3`) + L2 test-gen over the real `Σ`/`Vis` substrate.
4. **The RV face (§5).** Agent actions as `Σ`-events; a `T` obligation →
   `delegated`, monitored via `72`/`73`; a violation caught, never promoted.
5. **The honesty boundary (§6).** Quality → `unknown`, not dischargeable, never
   `proved`/`Q` — the `21 §5.4` guard, the `64 §4` over-claim refusal.

**Acceptance criteria.** *Names align with the frame's AC1–AC5.*

- **AC1 (reduction).** A conformance case routes an agentic scenario through the
  **real** `71` export with the agent output as maximal `P` — not a new
  mechanism; the landed projection classifies it into `P`, the invariant into
  `Q`.
- **AC2 (envelope, the flip).** A safety invariant is `proved` **for-all** agent
  outputs, driving the **real** Sec2 capabilities (propose/act split); the
  **verdict flips to reject** when the agent is handed `act` instead of
  `propose` (a world-effect reachable without the `safe` gate — privilege
  escalation). The flip is on the real `Cap E`/no-ambient check.
- **AC3 (metamorphic).** An oracle-free relational check **passes** on a
  nondeterministic producer via the **real** `OQ-relational` 2-run reduction +
  L2 test-gen; carried as `tested`, never `proved`.
- **AC4 (RV).** An agent-action `T` obligation is monitored end-to-end via the
  **real** B2/B3 (source `temporal` → `T` → `compile → Monitor` → trace); a
  **violating action is caught** (monitor rejects) and stays `delegated`.
- **AC5 (honesty, soundness).** A **quality** obligation maps to `unknown`,
  **never** `proved`/`Q`, and is **not dischargeable** — a postulate in
  `trusted_base()` with no certificate, flipping against a real proved safety
  obligation (`21 §5.4`).

**Conformance (`../../conformance/behavioral/agentic/`).** AC1–AC5 as
**discriminating** cases, each **routing a real checked program through the
actual producers** (the `71` emitter, the `62` capability check, the
`OQ-relational` reduction, the `72`/`73` monitor+trace) and observing the result
— **not** a synthetic agentic literal that re-validates a pre-existing consumer.
This is the **highest-risk producer-grep gate in the corpus**: because B4 is a
doc/composition WP, the hand-feeds-the-deliverable trap is most dangerous here.
The QA gate **greps the real producer src, not the test**: AC2 must reach the
real `Cap E`/no-ambient path (not an `isAgent` boolean); AC3 the real 2-run
product-program reduction (not an asserted relation over a canned pair); AC4 the
real `Vis`/trace site (not a hand-built event list); AC5 the real
`trusted_base()` membership (not a status string). The **cross-case sweep**
groups by the §2.1 status partition and asserts agreement — {envelope→`proved`/
`Q`}, {metamorphic→`tested`/`P`}, {RV→`delegated`/`T`}, {quality→`unknown`} —
with the two boundary invariants pinned: **no agent output ever lands in `Q`**
(the honesty direction, AC5) and **no downstream verdict ever lands in
`proved`** (the one-way direction, AC4; `71 §5.1`).

**No new mechanism — the reconciliation (WS-B capstone).** Every construct this
chapter names is landed: the export projection (`71`), the capability discipline
(`62`), the `Σ`/trace instrumentation (`73 §2`), the `Temporal`→`Monitor`
projection (`73 §2.4`), the `OQ-relational` product-program reduction
(`61 §5.3`). B4 introduces **no** formation rule, level rule, or kernel judgment
(the level-discipline reconcile is N/A — editorial). WS-B **completes on
merge**: the agentic case is the existing seam aimed at a
maximally-nondeterministic component, and Ken adds zero agentic mechanism.
