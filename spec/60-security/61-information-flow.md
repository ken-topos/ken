# Information-flow control

> Status: **DRAFT v0**. Normative for the *commitment and shape*; the label
> model and the static-vs-relational mechanics are flagged **OQ-ifc /
> OQ-relational**. This is the centerpiece of Ken's tier-1 security goal (ADR
> 0004): **data-flow control is intrinsic to Ken**, built on the machinery Ken
> already has, and it directly attacks the dominant AI-codegen failure — secret
> leakage and data crossing trust boundaries.

## 1. Why flow is a *discipline*, not "more refinements"

Ken's verification layer proves **unary** properties: an obligation `Γ ⊢ φ : Ω`
is a predicate over **one** execution ("this result is sorted"). Information
flow is different in kind. "This secret never reaches the network" /
**non-interference** is a statement about **pairs** of executions:

> for any two runs that differ only in their *secret* inputs, the *public*
> outputs are identical.

This is a **relational (2-safety)** property. It **cannot** be written as an
`ensures φ` clause, because `φ` can only see one run. So information-flow
control is a **typed discipline** that makes flow visible in types and
*guarantees* non-interference, plus (for bespoke claims) a relational
verification mode — not a library of ordinary refinements. Conflating the two is
the trap this chapter exists to prevent.

## 2. The security-label lattice

Ken adds a **label lattice** `ℒ` — a bounded lattice of security levels with
order `⊑` ("flows to"), join `⊔` (least upper bound), and meet `⊓`. Two
standard, composable lattices:

- **Confidentiality** — e.g. `Public ⊑ Internal ⊑ Secret`, or, more generally,
  the powerset of *principals/compartments* a value is readable-by (ordered by
  ⊇: fewer readers = more secret). Data flows **upward** only; a `Secret` value
  may not be observed where only `Public` is allowed.
- **Integrity** — the **dual** lattice: `Trusted ⊑ Untrusted` (low-integrity,
  attacker-influenced data may not flow into high-integrity sinks). This is
  precise **taint tracking**.

The full label is the product `(confidentiality, integrity)` (and, optionally,
compartments / data-residency regions for compliance). `ℒ` is an ordinary
lattice value in Ken (Ken has Heyting/lattice structure natively,
`../10-kernel/12 §5`), so **no new kernel primitive is introduced** to host it.

> **(OQ-ifc) — DECIDED (operator, 2026-06-27).** The **discipline is
> lattice-parametric**: non-interference (§5) is proved **once, for any bounded
> lattice `ℒ`** — so the concrete lattice is a *policy* choice (`65-policy.md`),
> not baked into the metatheory, and "fixed levels vs principal-sets vs
> user-defined" dissolves (all are instances). The **standard lattice is the
> decentralised label model (DLM)** — confidentiality = the set of principals a
> value is readable-by, integrity = the dual (endorsers) — with `Public ⊑
> Internal ⊑ Secret` as totally-ordered **sugar** and compartments /
> data-residency regions as **products**. Labels are **static type indices by
> default** (erasable, by-typing) with **first-class label values admitted at
> boundaries** for data-derived classification (§3). The concrete lattice,
> classifications, clearances, and declassification edges are supplied by a
> **separately-authored policy** (`65-policy.md`, ADR 0007).

## 3. Labeled values and labeled effects (riding the effect machinery)

A label attaches to data via a **labeled type** `A @ ℓ` (read "an `A` classified
at `ℓ`"), with a **labeled, indexed effect** governing observation and
combination. This is a **direct extension of Ken's effect encoding**
(`../30-surface/36 §2`, `OQ-8`): the effect that already indexes *capabilities*
(`Eff [E] A`) is the same machinery that here indexes *labels* — a protection
monad `T_ℓ`/`Labeled ℓ` in the DCC (Dependency Core Calculus) lineage.

- **Combination raises the label.** Computing on labeled data joins labels: `(x
  @ ℓ₁) ⊕ (y @ ℓ₂) : _ @ (ℓ₁ ⊔ ℓ₂)`. A function of secret input produces secret
  output by construction; you cannot "forget" a label by computing.
- **Observation requires clearance.** Eliminating/branching on a labeled value
  taints the context with its label; you may only act on it where the context's
  clearance `⊒` the value's label.
- **Channels carry clearance.** An effectful sink — `Net`, `FS`, a log, an RPC,
  a `space` cell (`../30-surface/36`) — has a **clearance label**. Writing data
  `@ ℓ` to a sink with clearance `κ` is well-typed **iff `ℓ ⊑ κ`**. So:

  ```
  view send (s : Socket κ) (msg : Bytes @ ℓ) : Unit  visits [Net]
    -- type-checks only when ℓ ⊑ κ
  ```

  "This secret must never reach the network" becomes: the `Net` sink's clearance
  is below `Secret`, so a `Secret`-labeled value flowing to it is a **type
  error** at compile time — not a lint, not a scan, a typing failure.

The effect row therefore evolves from *what a function may do* to *what may flow
where*: a function's type is simultaneously its **capability manifest** and its
**flow manifest**.

**Where labels live (riding the decided machinery).** Concretely, a label
**annotates each `perform` node of the interaction-tree denotation** (`OQ-8`,
`../30-surface/36 §2`) — so the same structure that carries effects and
authority carries flow, and non-interference is a property of the *labeled
tree*. Labels are **erasable type indices** (zero runtime cost) in the common
case. Across spaces, **shared-nothing messages carry labels** (`OQ-Space`,
`../30-surface/36 §4`): cross-space flow is checked at `send` against the
receiving space's clearance — **distributed IFC** for free — and Ward may
monitor labeled message events at runtime (`../70-behavioral/`).

### Data-derived classification — static by default, dynamic at the boundary

Most classification is **static** (policy- or source-determined; §`65-policy`)
and costs nothing at runtime. But real systems also classify **from data at
runtime** — e.g. per-tenant isolation: *"customer X's records go only to bucket
X."* Ken supports this **minimally**, not via full dynamic IFC:

- **Tag at ingestion.** A boundary reads the datum's compartment (the tenant)
  and assigns a label — a **first-class label value** — producing `A @
  Tenant[X]`. The label is then carried **statically** through the computation
  as an **existential** `∃ ℓ. A @ ℓ`; only the *value* of `ℓ` is dynamic.
- **Check at the sink.** Writing `@ Tenant[X]` to a sink whose policy clearance
  is `Tenant[Y]` is a flow violation — so misrouting to the wrong bucket is
  caught by the channel rule (§3), with a **single runtime label comparison**
  where two dynamic labels meet. The static majority pays nothing.
- **Ingestion is a trusted, audited point.** Asserting "this record *is*
  Tenant[X]" is a classification, not a proof — so it is **capability-gated and
  audited**, the dual of declassification (§4), and only **policy-sanctioned**
  ingestion points may classify (`65-policy`).

Deliberately **excluded**: faceted execution / pervasive runtime taint / dynamic
lattices (full dynamic IFC). That power is not worth its cost for the cases Ken
targets — *better is the enemy of good*. Ken pays for dynamism only at the
boundary where the data actually demands it.

## 4. Declassification — the only downgrade, explicit and audited

Pure label-monotonicity (§3) would make secrets permanently unusable: nothing
could ever be released. Controlled release is **declassification**
(confidentiality) / **endorsement** (integrity), and it is the *only* way a
label moves down:

```
declassify : Cap_declassify[ℓ→ℓ'] → A @ ℓ → A @ ℓ'      -- requires ℓ' ⊑ ℓ
```

- It is **capability-gated** (`62-authority.md`): only code holding the specific
  declassification authority may downgrade, and only along the permitted lattice
  edge.
- It is **explicit and local** — a syntactic operation at a named point, never
  implicit, so "where does PII get released?" is answered by *grepping for
  `declassify`*.
- It is **audited**: each declassification is a recorded event at a trust
  boundary (`62 §audit`), and a declassification authority used by a dependency
  appears in its **`trusted_base_delta`** (`63`, `../20-verification/25 §3`). A
  package that downgrades secrets cannot hide it.
- A **declassification policy** (which edges are permitted, under what proven
  precondition — e.g. "may release an average only over ≥ k records") can itself
  be a `requires` obligation on the declassify, tying release to a *proven*
  condition.

This is exactly what data-flow compliance wants: not "no secret ever moves" but
"every downgrade is explicit, authorised, conditional, and logged."

## 5. The guarantee: non-interference

A program that type-checks under the label discipline is **non-interfering** up
to its declassifications: public outputs do not depend on secret inputs except
where an authorised `declassify` permits. Ken offers this at two strengths:

1. **By typing (the automatic guarantee).** Well-labeled ⇒ non-interfering, as a
   **meta-theorem about the discipline** (proved once over the elaboration, in
   the DCC/sealing-calculus tradition). The programmer gets non-interference for
   free from passing the type-checker — the scalable, default property.
2. **By proof (bespoke / quantitative claims).** For finer statements — a
   specific declassification policy, a quantitative bound ("at most `n` bits
   leak"), or integrity of a specific pipeline — the claim is a **relational
   obligation** discharged by the relational verification mode
   (`OQ-relational`): product programs / relational refinement types comparing
   two runs, with the certificate kernel-re-checked like any other
   (`../20-verification/23 §1`).

> **(OQ-relational)** How relational/2-safety obligations are generated and
> proved (self-composition / product programs vs. relational refinement types
> vs. a dedicated logic), and whether the **default** discipline is
> termination-sensitive or -insensitive (does non-termination or a crash leak?),
> are open. The same machinery serves **constant-time** (`64`,
> `../40-runtime/43`) — a relational property over timing — so IFC and
> side-channel reasoning share a foundation.

**Crucially, none of this enlarges the trusted kernel.** Labels are
surface/effect constructs; the discipline elaborates to the kernel's existing
indexed-Π / monadic machinery; the relational certificates are re-checked by the
*same* small kernel. The TCB does not grow (ADR 0004 Decision 3, ADR 0001).

## 6. The topos grounding (why this is native, not bolted on)

A security label reads category-theoretically as a **modality** — an
(idempotent) monadic operator on Ken's topos of values — and the labeled monad
`T_ℓ` is exactly the protection modality of that level. Non-interference is then
a **relational/parametricity** statement about maps that respect the modality.
Ken's topos + dependent foundation makes information flow an *instance of
structure Ken already has* (modalities, the subobject lattice), not a foreign
type system stapled on. This is the design reason IFC fits Ken better than it
fits a mainstream language. *(The precise modal/sheaf formulation is
design-direction, tagged for the Verify enclave to develop; the engineering
above stands without it.)*

## 7. Worked examples

```
type Email   = String @ Secret[user]            -- PII, readable by `user` only
type ApiKey  = Bytes  @ Secret[server]          -- never client-visible

-- A type error: Secret PII reaching a Public log sink.
view audit (e : Email) : Unit  visits [Log Public]
  = log e            -- REJECTED: Secret[user] ⋢ Public

-- Allowed only via an authorised, conditional, audited downgrade.
view audit (e : Email) (d : Cap_declassify[Secret[user]→Public]) : Unit
  visits [Log Public]
  requires consented(e)                          -- release tied to a proof
  = log (declassify d (redact e))                -- explicit + logged

-- Integrity (taint): untrusted input may not reach a high-integrity sink.
view run (cmd : String @ Untrusted) : Unit  visits [Shell Trusted]
  = exec cmd         -- REJECTED: Untrusted ⋢ Trusted  (no command injection)
```

A CISO reads these and sees: PII boundaries, API-key confinement, and injection
resistance are **typed, compile-time, proven** properties with an **explicit,
audited** escape hatch — the exact controls compliance frameworks (GDPR/CCPA
data boundaries, PCI key handling, "no untrusted data to `exec`") ask for,
enforced by construction instead of by scanning.

## 8. What is committed vs. open

- **Committed + decided (`OQ-ifc`):** a **lattice-parametric** discipline (non-
  interference proved once for any bounded `ℒ`); the **DLM** standard lattice
  (levels as sugar, compartments as products); labeled types/effects riding the
  interaction-tree denotation; **static type-index labels** by default with
  **first-class labels at audited boundaries** for data-derived classification;
  upward-only flow; channel/sink clearances; **declassification** as the sole,
  explicit, capability-gated, audited downgrade; **non-interference** as the
  guarantee; **no kernel enlargement**. The concrete lattice/classifications/
  clearances/edges are supplied by a **separately-authored policy**
  (`65-policy.md`, `OQ-policy`, ADR 0007).
- **Still open:** `OQ-relational` — how relational/2-safety claims are *proved*
  (self-composition / product programs vs. relational refinement types vs. a
  dedicated logic) and whether the default is termination-sensitive. **Deferred
  and decided with `64`/constant-time** (both are relational over a hidden
  channel — shared foundation). The **by-typing** default (§5.1) needs none of
  it.

## 9. What WS-V / WS-L must deliver here

The label lattice + labeled types/effects in the surface and effect system
(`../30-surface/36`); the flow typing (combination raises, observation requires
clearance, sinks carry clearance); declassification/endorsement as
capability-gated, audited, optionally `requires`-conditioned operations
(`62-authority.md`); the **by-typing non-interference** meta-theorem; and the
**relational verification mode** for bespoke claims, with kernel-re-checked
certificates. Acceptance: a `Secret`→`Public` flow is a compile error; an
authorised `declassify` is accepted, logged, and shows in the
`trusted_base_delta`; a documented non-interference statement holds.
Conformance: `../../conformance/security/information-flow/`.
