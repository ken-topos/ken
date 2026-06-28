# Security (a tier-1 design goal)

> Status: **DRAFT v0**. Normative. Security is a **tier-1** goal for Ken,
> co-equal with the verification thesis (`../00-overview.md`,
> `../../docs/adr/0004-…`). This section states the threat model, the
> structurally-enforced guarantees, the information-flow discipline, the
> supply-chain and trust model, and — explicitly — the limits a language cannot
> exceed. Audience: security architects, CISOs, and the build teams. The goal is
> that a security professional reads Ken and concludes *"this is how it should
> be done."*

## 1. The one idea

**Trust flows from independent re-verification, not from who (or what) wrote the
code.** Ken's kernel re-checks every proof certificate (the de Bruijn criterion,
`../00-overview.md §3`): a property holds in *your* kernel or it does not —
regardless of whether a human, an LLM, or a hostile party produced the code and
its proof. In an era where the author may be an unreliable or adversarial model,
this **authorship-independence** is the property that matters, and Ken's whole
security posture is built on it.

Everything below is either (a) a structural consequence of that idea, (b) a
discipline layered on Ken's existing machinery to extend it to *flow* and
*authority*, or (c) an honest statement of what proof cannot reach.

## 2. Threat model (the AI era)

Code generation has changed the shape of the threat, in three ways relevant to a
*language*:

1. **Volume.** Subtly-vulnerable code is produced faster than humans can review
   it. → Defence: make large classes of bad behaviour **unrepresentable** (ruled
   out by typing) or **undeniable** (any assumption is explicit and
   machine-auditable), so review scales by *checking*, not re-reading.
2. **Untrustworthy authorship.** You can no longer anchor trust in "a careful
   human wrote this." → Defence: authorship-independence (§1) — admit code into
   a trusted system on the *kernel's* terms, never the author's say-so.
3. **Accelerated exploitation.** Models find and weaponise bugs at scale,
   including attacks on the build/supply chain itself. → Defence: a small
   audited TCB (`64-trust-model.md`), re-checked dependencies
   (`63-supply-chain.md`), and least-authority confinement (`62-authority.md`).

Ken operationalises the standing security advice — *treat AI-generated code as
unverified input, not trusted output* — **at the language level**, rather than
bolting it on with scanners after the fact.

## 3. The property taxonomy (read this first)

Security properties are **not** all the same shape, and Ken treats them
differently. This taxonomy is the spine of the section:

| Class | Example | Shape | Ken's mechanism |
|---|---|---|---|
| **Unary / functional** | access-control logic, input validation, "result is sorted" | a predicate over **one** run, `Γ ⊢ φ : Ω` | the verification layer (`../20-verification/`) — already provided |
| **Relational / 2-safety** | non-interference (no secret→public flow), constant-time | a property over **pairs** of runs | the **information-flow discipline** (`61-information-flow.md`) + relational verification (product programs / `@ct` taint; `OQ-relational` DECIDED, `61 §5/§5a`) |
| **Authority** | "this helper may not touch the network" | a capability/effect bound | effects + **capabilities** (`62-authority.md`) |
| **Provenance / supply-chain** | "this binary came from that source + build" | an attestation over artifacts | re-check + signing/SLSA (`63-supply-chain.md`) |
| **Meta (out of reach)** | "the spec captures my intent" | not a program property | **not solvable by a language** (`64-trust-model.md §4`) |

The crucial, easily-missed point: **information flow and constant-time are
*relational*** — they compare two executions ("for inputs differing only in a
secret, the public outputs agree"). They are **not** expressible as ordinary
`ensures φ` clauses, which talk about a single run. This is why IFC is a
*discipline* in Ken, not just "more refinements" (`61-information-flow.md §1`).

## 4. What Ken provides structurally (not aspirations)

| Guarantee | Mechanism | Chapter |
|---|---|---|
| **Small, auditable TCB** | the only trusted thing is a small permanent Rust kernel + listed primitives + listed postulates | `64`, `../10-kernel/18 §5` |
| **Untrusted prover** | a bug in the classifier / Kripke embedding / Z3 → *failure to prove*, **never a false `proved`** | `64`, `../20-verification/23 §1` |
| **Explicit assumption ledger** | every artifact ships a `trusted_base_delta`; a genuinely-verified artifact's delta is **empty** | `63`, `../20-verification/25 §3` |
| **Pure-by-default, typed effects** | a `view` is pure unless it declares `visits [...]`; effects are statically checked + transitively inferred — a type *is* a capability manifest | `62`, `../30-surface/36` |
| **Intrinsic information-flow control** | a security-label lattice on the effect discipline; flow type-checked; declassification explicit + audited; **non-interference** the guarantee | `61` |
| **Least authority** | capabilities are static, visible, and **attenuable**; no ambient authority | `62` |
| **Re-checked dependencies** | consuming a package **re-checks** its proof terms in your kernel (cheap), never trusts the author | `63` |
| **Trusting-trust defence** | the permanent Rust kernel is an independent second checker even after self-hosting | `64 §3` |
| **No hidden native trust** | every `foreign`/FFI is a listed postulate in the delta | `63`, `../30-surface/38 §3` |

## 5. Chapter map

| File | Subject |
|---|---|
| `61-information-flow.md` | The label lattice, labeled types/effects, declassification, **non-interference**, integrity, the topos grounding — *the centerpiece* |
| `62-authority.md` | Capabilities, principle of least authority, attenuation, revocation, boundary audit |
| `63-supply-chain.md` | Package/proof-bundle/interface format, re-check-not-re-prove, `trusted_base_delta` audit, signing + SLSA provenance, the registry |
| `64-trust-model.md` | The TCB, the de Bruijn criterion as a security property, kernel audit, the trusting-trust defence, and the **honest limits** (spec≠intent, side channels, the social layer) |

## 6. The honest frame

A language is most valuable as defence not by *detecting* bad code but by making
classes of bad behaviour **unrepresentable** (purity, capabilities, IFC labels
rule them out by construction) or **undeniable** (proofs and the
`trusted_base_delta` make every assumption explicit and machine-auditable). Ken
aims at both — and is deliberately clear (`64 §4`) about the residue it cannot
reach: that a *spec* captures *intent* is a human engineering-discipline
problem, not something any checker can decide. Over-rotating on "it's verified"
while ignoring that residue is itself a security failure, and this section says
so out loud.
