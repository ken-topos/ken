# ADR 0004 — Security is a tier-1 goal; information-flow control is intrinsic

- **Status:** Accepted
- **Date:** 2026-06-27
- **Deciders:** the operator

## Context

Ken's thesis is machine-checked correctness for the agentic era
(`01-strategy.md`, `spec/00-overview.md`). A security review of the published
spec (by a security professional + a research agent) reached two conclusions:

1. **Ken's existing design is already an unusually strong security baseline** —
   small auditable TCB, untrusted prover (no false `proved`),
   `trusted_base_delta` as an assumption ledger, pure-by-default effect rows as
   a capability manifest, FFI-as-listed-postulates, encapsulated mutable state,
   content-addressed dependencies. The structural properties are real, not
   aspirational.
2. **But Ken's verification is currently *unary and functional*** — an
   obligation is `Γ ⊢ φ : Ω`, a predicate over one execution. The dominant
   AI-codegen failure class — secret leakage, missing access control, data
   crossing trust boundaries — is an **information-flow** problem, not a
   functional-correctness one, and information flow / non-interference is a
   **relational (2-safety)** property that ordinary `ensures φ` clauses cannot
   express. The supply-chain story (a package/proof-bundle/interface format and
   signed provenance) is also unspecified.

The operator — who handles data-flow security and compliance professionally —
directs that **software security be a tier-1 priority for Ken**, co-equal with
the verification thesis, and that **data-flow control be intrinsic to the
design**, not a bolt-on. The target: a CISO / security architect reads Ken and
concludes "this is how it *should* be done."

## Decision

1. **Security is a tier-1 design goal.** A dedicated, normative
   `spec/60-security/` section states the threat model, the structural
   guarantees, and the honest limits. Security is a design principle in
   `00-overview.md`, not an appendix.

2. **Trust-from-re-verification is recognised as Ken's core *security*
   property,** not merely a soundness mechanism. The de Bruijn criterion makes
   trust **authorship-independent**: a proof checks in *your* kernel or it does
   not, regardless of whether a human, an LLM, or a hostile party produced it.
   This is the property the AI era demands and Ken's security case is built on
   it.

3. **Information-flow control (IFC) is intrinsic.** Ken gains a **security-label
   lattice** and a **labeled, indexed-effect discipline** — a *direct extension
   of the effect encoding* (`OQ-8`, `Eff [E] A`): labels index the same
   machinery that already indexes capabilities. Flow is type-checked (data may
   only flow upward in the lattice); **declassification is the only downgrade
   and is explicit, capability-gated, and audited**; **non-interference** is the
   guarantee. Confidentiality and integrity are the dual lattices. **This lives
   at the surface + effect-system + verification level and does NOT enlarge the
   trusted kernel** — the label discipline elaborates to the kernel's existing
   indexed-Π/monadic machinery, keeping the TCB small (Decision 1 of ADR 0001).

4. **Authority is PoLA, static, visible, and attenuable.** Capabilities
   (extending `36 §3` / `OQ-8a`) can be *attenuated* (hand a child a weaker
   token), and authority use at trust boundaries is auditable. A function's type
   is its authority manifest.

5. **Supply-chain consumption is re-check, not re-prove.** A Ken package is
   `(content-hashed source, artifact, proof-bundle / Σ-fragment,
   trusted_base_delta)` plus a compiled **interface file** carrying exported
   types + contract certificates. A consumer's kernel **re-checks** the proof
   terms (cheap — checking ≪ proving) rather than trusting the author's
   attestation; it does not re-run the prover. Cryptographic signing +
   SLSA-style build provenance (an axis *complementary* to Ken's program-level
   proofs) complete the story.

6. **The permanent Rust kernel is the independent second checker.** Because the
   Rust kernel stays small and permanent even after self-hosting (ADR 0001), the
   self-hosted toolchain always has an *independent* re-checker — Ken's
   structural defence against a "trusting-trust" attack (diverse
   double-compilation built in).

7. **Honest limits are stated, not papered over.** Three things a language
   cannot fix are recorded as explicit non-goals of the *language* (not denied):
   - **Spec ≠ intent.** The kernel proves code matches its spec, never that the
     spec matches intent; an adversary (or AI) can prove a *weak* spec. This is
     the dominant residual risk and a **complementary engineering-discipline
     project** (spec authoring/review/coverage — the principal-engineer/SWEBOK
     realm), not a Ken language feature.
   - **Side channels & resource bounds** (timing/constant-time, WCET/space, DoS)
     are not covered by functional proofs; constant-time is itself relational,
     so it shares the foundation IFC needs (`OQ-relational`). Optional/research,
     not core.
   - **The social/registry layer** (namespace ownership, key compromise) lives
     above the language and needs ecosystem governance.

## Rationale

- **Coherent with the existing design.** IFC rides the indexed-effect machinery
  Ken already chose; labels are a lattice (Ken has Heyting/lattice structure
  natively, and a label reads category-theoretically as a modality in the topos
  setting); declassification is a capability + an entry in the existing
  `trusted_base_delta` ledger. The TCB does not grow.
- **It attacks the dominant AI-codegen failure** (secret leakage / broken access
  control) at the type level, by construction, rather than via post-hoc
  scanning.
- **It makes the strongest part stronger.** "Trust flows from re-verification,
  not authorship" is precisely the inversion the AI era requires; naming it a
  security property (and specifying re-check-on-consume) turns it into a
  supply-chain guarantee.

## Consequences

- New normative section `spec/60-security/` (README + information-flow +
  authority + supply-chain + trust-model).
- The effect system (`30-surface/36`) is extended: effect rows / channels carry
  **labels**; capabilities become **attenuable**.
- A new verification *mode* is acknowledged: **relational / 2-safety**
  properties (non-interference, constant-time) are distinct from unary `Γ ⊢ φ :
  Ω` and need their own treatment (`OQ-relational`) — either a static label
  discipline giving non-interference *by typing*, or relational obligations
  (product programs / relational refinements) for bespoke claims.
- New open decisions: `OQ-ifc` (label model + static-vs-relational),
  `OQ-relational`, `OQ-provenance` (signing/SLSA/`.keni`); `OQ-8a` and
  `OQ-Space` gain a stated security requirement (attenuation/revocation; a
  *proven* isolation property, no longer "deliberate choice, not inherited").
- The kernel stays small (security additions are surface/discipline/tooling,
  never new trusted primitives) — Decision 3 preserved.

## Revisit if

- The label discipline cannot deliver non-interference without kernel changes
  (then the cost/benefit of a kernel-level relational facility is reconsidered —
  but the bar is high: enlarging the TCB for IFC is a last resort).
- Regulated-industry requirements (DO-178C, ISO 26262, IEC 62443) demand a
  specific assurance artifact Ken does not yet emit (extend the protocol, not
  the kernel).
