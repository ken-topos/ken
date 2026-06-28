# Ken language specification — Overview

> Status: **DRAFT v0**. This chapter is normative for terminology and scope; the
> per-area chapters are normative for their subjects. Where this document and a
> later chapter disagree, the chapter wins and this overview is to be corrected.

Ken is an **MIT-licensed, Rust-hosted, interpreter-first verified topos
language**. It exposes a dependent type theory with observational equality at
the surface, discharges correctness obligations automatically where it can, and
makes proof *failures* legible to both humans and agents — so that a program can
be written *and proved correct* at commercial scale, on top of a small auditable
trust root.

This specification is what Ken's implementation is built against. It descends
from the design ideas explored for an AGPLv3 prototype but is a **clean-room**
design: it is written from those ideas and from type-theoretic first principles,
not from the prototype's source (`../CLEAN-ROOM.md`). The prototype is, at most,
a behavioral oracle for the Spec enclave to consult; this document is the
authority.

---

## 1. The verification thesis

Agents can already write working — often high-quality — code. The binding
constraint on *deploying* agent-written code is **verification**: the obligation
to assert that code does what it intends without putting users at risk. Today
that assertion is made at one of three levels:

- **L0 — Tests.** Empirical, sampled correctness. The industry default. Catches
  what you thought to check; silent on everything else.
- **L1 — Static safety.** Type- and borrow-checking (Rust, TypeScript). Rules
  out whole *classes* of error (memory unsafety, many type errors) but says
  nothing about a function computing the *intended* result.
- **L2 — Propositional correctness.** A machine-checked proof that the code
  satisfies a stated specification. Dependent-type systems (Lean 4, F\*, Coq,
  Agda) reach L2 but are aimed at mathematicians, demand interactive expertise,
  and are not shaped for commercial software delivery.

**Ken's thesis:** the empty middle — an L2 guarantee delivered with L1
ergonomics — is occupiable. A language whose dependent kernel is exposed to the
surface, whose proof obligations are discharged automatically when decidable,
and whose proof failures are *machine-readable*, is a language an agent can
drive: write a function, state a property, and get back a verdict it can act on
without reading the kernel.

"Verified," in this document, means **L2**: the deployable guarantee is a
kernel-checked proof, not a passing test suite. L0 and L1 remain available and
useful; they are not the claim.

---

## 2. North star

An agent (or programmer) submits a Ken definition together with a specification
— preconditions, postconditions, refinements, or a propositional goal. The
toolchain returns exactly one of:

- **Proved** — a kernel-checked certificate exists.
- **Disproved** — with a concrete **countermodel** (a witness world/assignment
  that falsifies the goal).
- **Incomplete** — with a **typed hole** marking what is unknown and structured,
  machine-readable next-step guidance; a partially-verified program still
  *runs*, propagating an explicit `unknown` rather than failing closed.

The verdict is actionable without reading the kernel. This loop — **write → spec
→ verify → repair** — driven unattended by an agent team, is the product
(strategy gate G7).

---

## 3. Shape of the system

```
  surface syntax
        │  parse
        ▼
   surface AST ──elaborate──►  core terms  ──check──►  ┌───────────────┐
        │         (V0)          (de Bruijn)            │ TRUSTED KERNEL │
        │                                              │  (small, Rust, │
        │                                              │   permanent)   │
        │  obligation gen (V2)                         │  type theory + │
        ▼                                              │  conversion +  │
   proof obligations ──classify──► prover (V3)         │  proof check   │
        │                            │ Z3 / Kripke      └───────────────┘
        │                            ▼                         ▲
        │                       certificate ──re-check─────────┘
        ▼
   diagnostics (V4): countermodel · typed hole · three-region decomposition
        │
        ▼
   interpreter (X1) — the reference operational semantics
```

Two invariants govern this shape:

1. **The kernel is the only thing that must be trusted.** Everything else — the
   elaborator, the prover, the obligation generator, later a native backend — is
   *untrusted*: its output is a core term or a proof certificate that the kernel
   re-checks. A buggy prover cannot make an unsound program type-check; at worst
   it fails to find a proof. This is the **de Bruijn criterion**, and it is why
   the prover may use a *classical* SMT solver (Z3) without importing classical
   axioms: the solver is an oracle that *proposes*; the kernel *disposes*.
2. **The interpreter is the reference semantics.** Meaning is defined by
   evaluation in the Rust interpreter (X1). A later native backend (X3) is
   correct iff it agrees with the interpreter on a differential corpus; the
   interpreter never stops being the oracle.

---

## 4. Design principles

1. **Small, permanent, auditable trust root.** The kernel — core dependent type
   theory with observational equality, decidable conversion, the proof checker —
   stays small and in Rust forever (Lean's C++-kernel model). The smaller
   observational core (no cubical machinery, ADR 0005) makes this trust root
   smaller still. The elaborator and prover build on top and may self-host
   later; the kernel does not.
2. **Soundness from day one.** Universes are checked (no `Type:Type`); `Sigma`
   is genuinely dependent; `J` reduces on non-`refl` equalities; conversion is
   decidable and certified. The prototype's documented soundness gaps are
   **not** reproduced (see `../docs/program/01-strategy.md` non-goals).
3. **Total by default.** Definitions admitted to the kernel are checked for
   termination (size-change termination over δ-unfolding); non-terminating or
   partial computation is explicit, not silent.
4. **Content-addressing as identity.** Values are identified by the hash of
   their structure, giving O(1) structural equality and global deduplication.
   Identity is a property of *what a value is*, not where it lives.
5. **Failure is a first-class output.** A proof that does not go through yields
   a structured artifact (countermodel / typed hole / region decomposition),
   never an opaque error. Legibility is a feature, specified in
   `20-verification/`.
6. **Real types from the start.** `Int` (arbitrary precision) and `Decimal` are
   foundational; floating point, if present, is one numeric type among several
   and never the substrate. Ken is not an f64-only calculator (the central
   correction from the reality-check; see `30-surface/35-numbers.md`).
7. **Clean-room and permissively licensed.** The design is reusable; the
   prototype's source is not. The MIT license depends on this discipline.
8. **Security is structural and tier-1.** Trust is **authorship-independent** —
   the kernel re-checks every certificate, so a property holds regardless of who
   (a human, an LLM, an adversary) wrote the code. On that base, Ken makes whole
   classes of bad behaviour **unrepresentable** (purity, least-authority
   capabilities, and **intrinsic information-flow control**) or **undeniable**
   (every assumption is listed in the `trusted_base_delta`). Security is a
   tier-1 goal, co-equal with verification, specified in `60-security/` (ADR
   0004) — and the limits a language cannot exceed are stated there, not hidden.

---

## 5. Scope of this specification

**In scope, at full rigor (unblocks the build spine):**

- The **trusted kernel**: core syntax, universes, Π/Σ, inductive types,
  identity/`J`, the observational equality layer, definitional equality and
  decidable conversion, the typing judgment and kernel API (`10-kernel/`).
- The **verification surface**: specification syntax, obligation generation, the
  prover architecture, diagnostics, and the machine-readable protocol
  (`20-verification/`).
- The **security model** (tier-1): the threat model, information-flow control,
  authority/capabilities, supply-chain consumption, and the trust model + honest
  limits (`60-security/`).

**In scope, at decreasing resolution:**

- The **surface language** and its elaboration to core (`30-surface/`).
- The **runtime / reference operational semantics**, value model, and
  termination (`40-runtime/`).
- The **prelude / core stdlib** shape (`50-stdlib/`).

**Out of scope (non-goals for this track; recorded in
`../docs/program/01-strategy.md`):**

- Full higher-order automated proving (interactive tactics serve instead).
- Native code generation before the verification loop works.
- The coalgebraic research layer (Store-comonad cells, process coalgebras,
  profunctor wires, co-Heyting boundaries), linear/affine types, and real
  delimited continuations — these are research (WS-R), harvested back as
  ordinary features if they earn it.
- Reproducing the prototype's f64-only numeric model, unchecked universes, or
  hard per-store slot ceiling.

Genuine design forks encountered while writing are recorded in
`90-open-decisions.md` for the operator, not silently resolved.

---

## 6. How to read this spec

| You want… | Read |
|---|---|
| The trust root / type theory | `10-kernel/` (start at its `README.md`) |
| The verification story (the differentiator) | `20-verification/` |
| The language a programmer writes | `30-surface/` |
| How programs run / are identified | `40-runtime/` |
| What's built-in | `50-stdlib/` |
| The unresolved choices | `90-open-decisions.md` |

Each area chapter is self-contained and cites the kernel rules it depends on.
Conformance cases (executable behavioral tests) live in `../conformance/` and
cite the spec section they pin.

---

## 7. Normative language & conventions

- **MUST / MUST NOT / SHALL** — a hard requirement; a conforming implementation
  that violates it is wrong.
- **SHOULD / SHOULD NOT** — a strong recommendation; deviations need a recorded
  rationale.
- **MAY** — genuinely optional.
- A claim tagged **(oracle)** is to be cross-checked against the prototype's
  observed behavior by the Spec enclave; it is provisional until confirmed.
- A claim tagged **(OQ-n)** is an open design decision tracked in
  `90-open-decisions.md`.
- Inline core terms use de Bruijn-free named notation for readability; the
  kernel representation is de Bruijn (`10-kernel/11-syntax.md`).

---

## 8. Glossary (orientation; precise definitions in the cited chapters)

- **Topos** — a category that behaves like a universe of sets with an internal
  (intuitionistic) logic; Ken's value/type world is modelled on one. Its
  internal logic is **Kripke**, which is why a classical solver embeds soundly
  (`20-verification/23-prover.md`).
- **Subobject classifier (Ω)** — the type of propositions; "is this element in
  this subset?" is a map into Ω. Ω is a Heyting algebra, not Boolean
  (`10-kernel/` + `20-verification/`).
- **Π / Σ** — dependent function and dependent pair types (`10-kernel/13`).
- **Identity type / `J`** — propositional equality and its eliminator
  (`10-kernel/15`).
- **Observational equality** — `Eq` computed by recursion on the type, with a
  `cast` transport that makes `J`/`subst` *compute* on non-`refl` equalities;
  funext/UIP definitional, set-level, no univalence (`10-kernel/16`, ADR 0005).
- **Definitional vs propositional equality** — the kernel's decidable conversion
  vs. equality one must prove (`10-kernel/17`).
- **Obligation / verification condition (VC)** — a proposition whose proof
  certifies a spec is met (`20-verification/22`).
- **de Bruijn criterion** — soundness rests only on a small re-checking kernel;
  see §3. Its security reading is **authorship-independence**
  (`60-security/64`).
- **Content-addressed** — identified by the hash of structure (`40-runtime/41`).
- **Information-flow control (IFC) / non-interference** — a *relational*
  (2-safety) discipline ensuring data flows only upward in a security-label
  lattice; "no secret reaches a public sink" (`60-security/61`).
- **Capability** — an unforgeable, static, attenuable authority token; a
  function's type is its authority manifest (`60-security/62`).
- **Trusted computing base (TCB)** — exactly the kernel + listed primitives +
  listed postulates; everything else is re-checked (`60-security/64`,
  `10-kernel/18 §5`).
- **`trusted_base_delta`** — the machine-readable ledger of every assumption an
  artifact introduces; empty = fully verified (`20-verification/25`,
  `60-security/63`).
