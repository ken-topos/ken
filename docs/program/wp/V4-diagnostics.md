# WP V4 — Proof-failure diagnostics (the agentic differentiator)

> **Status:** Steward frame — **next enclave WP** (deps met: V2 + V3 on `main`).
> spec-leader elaborates `spec/20-verification/24-diagnostics.md` (DRAFT →
> implementation-ready), then **Team Verify** builds it (after V3-build).
>
> **Team:** Verify · **Deps:** **V3** (`c43cdfb`, the prover — its `unknown`/
> `disproved` outcomes are what V4 explains) + **V2** (`c684273`, obligations) ·
> **Size:** M–L · **Risk:** ★★ (**untrusted** — a diagnostic is *advisory UX*; a
> bad one is unhelpful, never unsound) · ► **Closes the verification spine**
> V1→V2→V3→**V4** — and per `24` "the feature that **most differentiates Ken for
> agentic use**": a proof that doesn't go through yields a *structured,
> machine-readable explanation*, not an opaque error.

## Objective

Elaborate `24-diagnostics` — the **four diagnostic mechanisms** that turn a failed
or partial proof into a structured explanation an agent can act on, each derived
from the **topos/Heyting** structure (not a Boolean error string):
1. **Kripke countermodels** (`§1`) — a finite falsifying model naming the **world
   that breaks `φ`**, and crucially distinguishing **`φ` is false** (refuted) from
   **`φ` is unknown** (no world forces `φ` *or* `¬φ` — needs more facts). This
   `false`-vs-`unknown` split is invisible to a Boolean counterexample and is the
   actionable signal (fix the spec vs. supply more facts).
2. **Typed holes + `unknown` propagation** (`§2`) — an undischarged obligation
   becomes a **typed hole**; the program **still type-checks and runs**, and
   evaluation that depends on an open hole yields the runtime third value
   **`unknown`** (Kleene/Heyting: `unknown ∧ false = false`, `unknown ∨ true =
   true`). Unifies obligation / typed hole / visible postulate.
3. **The three-region Heyting decomposition** (`§3`) — **proved / false /
   `unknown`** regions, where `unknown = {x | ¬¬φ x} ∖ S_φ` is the `φ`/`¬¬φ` gap
   (empty in a Boolean algebra, **nonempty** in a Heyting one — the home of a
   classically-valid-but-intuitionistically-unprovable goal).
4. **The fourth mechanism** (`§4`) — elaborate per the landed chapter.

## The framing that sets the risk level

V4 is **untrusted** — ★★. Diagnostics never affect soundness (the kernel already
decided `proved`/not via the cert); a V4 bug is a **misleading or missing
explanation**, an agentic-UX regression. Build it **faithful + actionable**. The
load-bearing properties are **fidelity to the prover's actual verdict** (the
diagnostic must reflect what V3 truly concluded — never label an `unknown` as
`false`) and the **`false`-vs-`unknown` distinction is honest** (the same
kernel-structural-status discipline: `unknown` is the `¬¬φ` gap, not a
counterexample). The serialization is **out of scope** (`25-protocol`).

## Scope

**IN:** the four mechanisms above as **structured, machine-readable** diagnostic
*values* (the data an agent consumes); the **`false` vs `unknown`** discriminator
(Kripke model present ⇒ false; no forcing world ⇒ unknown); the **typed-hole**
representation + `unknown` runtime propagation rule (the Kleene/Heyting table);
the **three-region** classification of a goal; consuming V3's `disproved`+model /
`unknown`+hole outputs faithfully.

**OUT — other WPs:** the **wire serialization** (`25-protocol`, separate); the
prover itself (`23`/V3); the obligation generation (`22`/V2); any change to the
*verdict* logic (V4 explains the verdict V3 produced, it does not re-decide it).

## The elaboration this needs (spec-leader → spec-author + Architect)

Elaborate `24` to builder rigor: each mechanism's diagnostic value + how it is
**derived from V3's output** (countermodel → Kripke diagnostic; hole → typed-hole
+ propagation; verdict → three-region); the `false`/`unknown` discriminator stated
precisely. **Ground against landed V3 (`23`, the verdict + countermodel + hole
shapes) + `24` + the Heyting/topos structure — the files, not status.** **Pin the
verdict→diagnostic mapping at the source** (the V3-prover carry — an unpinned
"false or unknown" is a latent bug; a classically-valid goal diagnoses as
**`unknown`**, never `false`, by Glivenko). Conformance (`conformance/verify/
diagnostics/`): a refuted `φ` → Kripke countermodel + `false`; a `¬¬φ`-gap `φ`
(e.g. `p∨¬p`) → `unknown` region, **not** false (verdict-flip + the cross-case
metatheory-consistency sweep); a hole propagates `unknown` per the Kleene table;
the three regions partition correctly.

## Acceptance (testable)

1. **`false` vs `unknown` is honest:** a refuted goal yields a Kripke countermodel
   tagged `false`; a `¬¬φ`-gap goal yields `unknown` (no forcing world) — **never**
   mislabel the gap as `false` (the load-bearing fidelity property; verdict-flip).
2. **Typed holes run:** a program with an open hole type-checks + runs; `unknown`
   propagates per the Kleene/Heyting table (`unknown ∧ false = false`, etc.).
3. **Three-region partition:** proved / false / `unknown` classify a goal
   correctly; the `unknown` region is the `¬¬φ` gap.
4. **Fidelity to V3:** each diagnostic reflects V3's *actual* verdict + evidence
   (countermodel/hole), consumed unchanged.
5. **No regression:** a fully-`proved` program emits **zero** diagnostics; V3's
   pipeline is unaffected.

## Sequencing

Next enclave WP after V3 (deps V2 + V3 landed). **Closes the verification spine**
→ the **G2/G3/G4 thesis** (`05`-DAG). Build follows V3-build on Team Verify; the
wire format (`25`) pipelines after. Build queries: diagnostic semantics → Spec;
the consumed-by-agent shape → Architect. Clean-room: landed `23`/`24` + Heyting/
topos first principles; no copyleft.
