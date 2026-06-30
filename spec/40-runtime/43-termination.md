# Totality, termination, and partiality

> Status: **DRAFT v0**. Normative for *where Ken is total and where partiality
> can appear*. Mostly a runtime-side recap of the kernel's SCT gate
> (`../10-kernel/17 §4`) plus the honest accounting of the partial sources.

## 1. The total core

Ken's checked core is **total**: every definition the kernel admits terminates
on all inputs. Two mechanisms, both at admission time (`../10-kernel/`):

- **Structural recursion via eliminators** (`../10-kernel/14 §3`) — `match`/
  recursion over inductives compiles to `elim_D`, which recurses only on
  structurally smaller sub-values; always terminating, no extra check.
- **General recursion gated by SCT** (`../10-kernel/17 §4`) — δ-definitions that
  are not plainly structural are admitted only if the **size-change
  termination** check certifies every loop has a decreasing thread. Uncertified
  recursion is **not** admitted as a transparent (total) definition.

Totality is what makes definitional equality decidable (`../10-kernel/17 §5`)
and the logic consistent (`../10-kernel/18 §6`) — it is load-bearing, not a
nicety.

## 2. Where partiality *can* appear (and is marked)

A program is not "all total all the time"; partiality enters only at **marked**
points, never silently:

1. **Open verification holes** → `unknown` at runtime (`42 §4`,
   `../20-verification/24 §2`). The hole is a *listed postulate*
   (`../10-kernel/18 §5`); a fully-verified program has none.
2. **Partial primitive operations** — division by zero, fixed-width overflow
   under a non-wrapping op (`../30-surface/35 §3`), array index out of bounds.
   These either (a) carry a **refinement precondition** making them total (`{ d
   | d ≠ 0 }`), (b) return `Option`/`Result`, or (c) at an unguarded use produce
   a **runtime fault / `unknown`** — but the *obligation* to avoid them is
   generated (`../20-verification/22`), so unguarded partiality is a visible,
   provable concern, not a silent trap.
3. **FFI / effects** — a `foreign` call may diverge or fault outside Ken's
   control; it is a trusted postulate (`../30-surface/38 §3`) and its partiality
   is part of the (listed) trusted boundary.
4. **Opaque (SCT-rejected) definitions** — if the elaborator admits a non-
   terminating definition as **opaque** (`../10-kernel/17 §4`), it never
   δ-reduces (so it cannot break conversion) but may diverge at *runtime*. This
   is an explicit, surfaced choice, not a default.
5. **Resource-limit exhaustion** — a `space`'s interning store or arena hitting
   its capacity bound raises `CapacityExhausted` (`44 §2`): a **loud,
   catchable** fault surfaced at the `space` boundary, **never** a silent
   drop/alias. **Distinct** from the partial-primitive class (case 2): the
   program stays logically total — this is a runtime *resource* bound, so Ken
   generates **no** static "never exhausts" obligation (the stance is
   detect-and-fail-loud, `44 §2`). True OS memory exhaustion is the
   loud-*fatal* variant (`44 §2`).

## 3. The honest statement

> Ken's **verified** core is total and its logic consistent. **Runtime**
> partiality is confined to: open holes (→`unknown`, listed), unguarded partial
> primitives (→ obligation, then fault/`unknown`), the FFI/effect boundary (→
> listed postulate), and opt-in opaque non-total definitions — plus,
> **distinct** from these, **resource-limit exhaustion** (`CapacityExhausted`,
> `44 §2`), a **loud** runtime resource bound that halts at the `space` boundary
> rather than returning a wrong or `unknown` value. Every one is *marked* — in
> the type, the obligation set, the trusted base, or the loud `space`-boundary
> fault — so a reviewer or agent can see exactly where totality is not
> guaranteed.

This is stronger than "tests pass" (L0) and honest about the boundary, which is
the posture a *verified* language must take.

## 4. Coinduction / productivity (`OQ-coinduction` DECIDED — deferred)

Infinite/streaming data (`../30-surface/37 §3`) needs **productivity** (each
step makes progress), the dual of termination. **Decision (operator,
2026-06-27): Ken's core is inductive/total — no coinductive types, no
productivity checker.** A productivity/guardedness checker is real TCB growth
(the dual of SCT, the same guarded-modal machinery `OQ-temporal` declined), and
infinitude is **already routed away from the value layer**: the interaction-tree
denotation of a *total* program is **finite** (it reaches `Ret`), and genuinely
forever-running processes are total per-message handlers + the runtime loop +
Ward's temporal model (`OQ-Space`, `OQ-temporal`). Streaming needs are met by
**generators**, **`Lazy` streams with a fuel/depth bound**, or the **seam**
(`../30-surface/37 §3`) — each **finite-by-construction** at every use, so the
totality story is unchanged.

**Re-open trigger.** A concrete, recurring need for first-class infinite
*values* or coinductive/bisimulation proofs the three idioms cannot serve. The
route then — consistent with `OQ-temporal`/`OQ-12` — is a **contained
sized-types/guardedness discipline** or a **deep-embedded coinductive layer with
reflective productivity**, never native clock-modal kernel growth.

## 5. What WS-X/WS-K must deliver here

The runtime honoring the kernel's totality (the total core never diverges); the
marked partial sources (§2) with their runtime behaviour (`unknown`/fault) and
their static obligations; and the opaque-definition escape hatch behaving as
specified (no δ, may diverge at runtime). Conformance:
`../../conformance/runtime/termination/` — a total recursive function runs to
completion, an unguarded partial op generates an obligation and
faults/`unknown`s when unproven, and an opaque non-total definition is δ-inert
but runtime-divergent.
