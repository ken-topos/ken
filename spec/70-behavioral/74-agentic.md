# Assuring agentic outputs — the boundary

> Status: **DRAFT v0**. Normative for **what Ken does *not* build** and why that
> is safe. **`OQ-agentic-oracle` DECIDED** (operator, 2026-06-27): Ken adds **no
> agentic mechanism** — assuring agent outputs reduces to already-decided
> machinery (`61`/`62` + the seam). This chapter states the boundary so the next
> reader's "but what about assuring the agent?" finds its answer. ADR 0006.

## 1. The problem, and why it is not a new Ken mechanism

A Ken-built system may **embed an agent** at runtime (an LLM proposes a plan, a
query, a tool call, a config). Such outputs have **no propositional oracle** —
you cannot write a spec `φ` that says "this generated summary is correct." The
question "what assurance is possible?" looks like it needs new machinery. It
does not. The decisive reframe — the same one that runs through the whole
project:

> You don't trust the agent; you verify the **boundary**. At *authoring* time
> the boundary is the kernel (re-check the proof). At *runtime* the boundary is
> a verified envelope (constrain the action). Same de Bruijn-criterion spirit at
> both layers: untrusted producer, verified check.

Structurally, **an embedded agent is just a maximally-nondeterministic input** —
the strongest case of an assumption `P` in the export (`71`). Ken already proves
systems safe *for all values* of a nondeterministic input. So the problem
**reduces to the existing seam**, pointed at the most adversarial environment.

## 2. The three mechanisms — all already decided

| Mechanism | What it gives | Where it lives (already decided) |
|---|---|---|
| **Safety envelope / verified shield** | the system is safe *whatever* the agent emits | **capabilities (`62`) + IFC (`61`) + contracts** — the agent holds only a *propose* capability; a verified validator holds the *act* capability; the system's invariants are proved independent of the agent. `proved`. |
| **Metamorphic relations** | oracle-free *relational* checks (round-trip, permutation-invariance, monotonicity) — no ground truth needed | **`OQ-relational`** (decided, `61 §5`/`§5a`, `30-surface/36 §3`) + test-gen (L2) exercises them. |
| **RV watchdogs** | the agent's observable actions obey safety/temporal obligations | **`73` + `72`** — agent actions are events in `Σ`; monitors synthesized from `T`. |

The unifying point: the agent = maximal `P`; the **envelope** proves `Q` for all
of it (static); **metamorphic** relates runs (oracle-free); **RV** watches the
actions (runtime). No new mechanism — the agentic case is the existing seam
aimed at a maximally-nondeterministic component. The verified shield is the
agentic reading of *least authority* (`62`): the agent plays inside a Ken-proved
FSM and can only *choose* among permitted transitions.

## 3. The honesty boundary — safety, never quality

This assures **safety, structural validity, and relational consistency** — never
**quality**. Ken/`Ward` can prove the agent *cannot harm* (escape the envelope,
exceed authority, violate an invariant); they **cannot** prove the agent is
*good* (the summary is faithful, the plan is wise). Output *quality* is the
**`unknown`** quadrant of the four-way status (`../20-verification/21 §5`) — a
human/eval-set judgment, **explicitly outside** the assurance boundary. The
four-way status keeps this legible:

> the **envelope** is `proved`; the **agent output** is `tested` / `delegated` /
> `unknown` — **never `proved`**.

Claiming more would be the dishonesty a verified language must refuse (`64 §4`).

## 4. What this means for scope

Ken builds **nothing new** for agentic assurance. The verified envelope is a
composition of `61`/`62`/contracts; metamorphic assurance depends on
`OQ-relational`; RV is `73`. This chapter exists to **state the boundary** — the
assurance Ken offers for agent-bearing systems (safety, by the envelope) and the
assurance it deliberately does not (quality) — so the boundary is legible rather
than discovered. The consuming engines (the shield's runtime, the metamorphic
tester, the RV monitor) are the same downstream family as the rest of the seam
(`73 §4`). Conformance: covered by `61`/`62`/`73` cases; no new corpus.
