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

> **(OQ-ifc)** The exact label model — fixed level lattice vs. principal-set
> (decentralised label model, DLM-style) vs. fully user-defined lattices;
> whether labels are first-class values, type indices, or both — is open. The
> *commitment* (a lattice + upward-only flow + audited declassification +
> non-interference) is fixed; the model is `OQ-ifc` in
> `../90-open-decisions.md`.

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

- **Committed (intrinsic to Ken):** a label lattice (confidentiality +
  integrity); labeled types/effects extending the effect discipline; upward-only
  flow; channel/sink clearances; declassification as the sole, explicit,
  capability-gated, audited downgrade; non-interference as the guarantee; no
  kernel enlargement.
- **Open:** `OQ-ifc` (label model + first-class-value vs. type-index);
  `OQ-relational` (how relational claims are proved; termination/progress
  sensitivity). These are sub-decisions *within* the committed design, in
  `../90-open-decisions.md`.

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
