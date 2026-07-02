# Ken design principles — the reasoning charter

> **Audience: every agent working on Ken — especially Steward, Spec, and
> Implementation.** These are the *implicit* values of the project, made
> explicit. They emerged while resolving the open-decisions register
> (`spec/90-open-decisions.md`, ADRs 0005–0008); decision after decision turned
> on the same handful of commitments. **When the spec dictates an answer, follow
> it. When it does not, reason from these.** They are priors for judgment, not a
> substitute for it.

Each principle is stated, justified in a line, and shown *in practice* — the
concrete decisions it produced — so you can pattern-match your own situation
against it.

---

## I. What Ken is — the mission

### 1. Ken is a *software-engineering* language, not a programming language

Prior languages were tools for humans to communicate with humans — more craft
than engineering. Ken is **written by agents and read by humans**, and its job
is to **prove what can be proven and state what must be tested**, legibly for a
sufficiently-educated human. Every other principle is downstream of this one.

*In practice:* the human's scarce attention goes to **reviewing the
specification** (the `requires`/`ensures`/refinements), not re-reading generated
code; the code's correctness is the kernel's job (`64`). The four-way epistemic
status — **proved / tested / delegated / unknown** — exists so the document
never pretends to more certainty than it has (`20-verification/21 §5`,
`70-behavioral`).

---

## II. How to decide — the decision calculus

### 2. The agent-writes / human-reads asymmetry is a design axis

Writing is **cheap** (agents do it and collapse the effort); reading and
verification are **dear** (humans do them, and they are the bottleneck). So
**optimize the canonical, permanent form for reading and checking**, and push
cost onto the writing side.

*In practice:* syntax is **read-optimized** — rich notation matching CS/Math
convention, with a total ASCII transliteration agents need not prefer
(`30-surface/31 §1a`). Typeclass resolution is **canonical and explicit** rather
than convenient (`ADR 0008`): the cost of explicitness lands on writing, the
benefit (legible, stable proofs) on reading. When a choice trades writer
convenience for reader clarity, take it.

### 3. Decide on intrinsic merits, not effort (feasibility is the only hedge)

Person-time and "amount of work to build" are the **wrong axis**: a team of
agents collapses effort that human priors treat as decisive. Weigh the things
that are actually permanent — **correctness, soundness, a small auditable TCB,
legibility, fitness for purpose**. The only risk that justifies a hedge is
**feasibility / correctness**, never effort.

*In practice:* the reflective proved-adequacy SMT route is the *target* on
intrinsic merits (a permanent artifact, robust to solver drift), with
reconstruction kept only as a *feasibility* hedge — not an effort tradeoff
(`OQ-12`). When you catch yourself estimating "that's a lot of work," stop: ask
instead which option is *correct and permanent*, and hedge only genuine
feasibility doubt.

### 4. Choose the correct / natural / elegant / permanent over the expedient

"Time to market" and "less to implement" are not Ken values. Prefer the choice
that will still be right in ten years and reads as inevitable.

*In practice:* observational equality over cubical despite more upfront design
(`ADR 0005`); arbitrary-precision `Int` from day one over an f64 shortcut; the
permanent Rust kernel over a faster-to-ship larger TCB.

---

## III. Design invariants — the recurring commitments

### 5. Keep the TCB small, permanent, and auditable — the de Bruijn criterion

The kernel is the **only** trust root; everything else (elaborator, prover, SMT,
codegen, the agent) produces certificates the kernel **re-checks**. This is not
only soundness — it is **authorship-independence**, and therefore a *security*
property for the AI era: *a property holds in your kernel or it does not,
regardless of who or what produced the code and its proof.* The untrusted model
generates; the trusted kernel filters.

*In practice:* a bug or malice in any tool can cause a *failure to prove* or a
*rejected certificate* — **never a false `proved`** (`64 §2`). Consuming a
dependency means your kernel **re-checks** its proofs, not that you trust its
author (`63`). **Before adding anything to the kernel, the burden is on the
addition** to justify the TCB growth against this principle.

### 6. Reflect, don't extend

When something tempts you to grow the trusted core, the Ken move is almost
always to **deep-embed it as ordinary data and reason about it reflectively**
instead. The kernel stays small; the power lives in libraries the kernel checks.

*In practice — the same move, repeatedly:* observational `Eq`-as-data over
cubical primitives (`ADR 0005`); temporal logic as a `Temporal` datatype,
**not** kernel modalities (`72`); SMT discharge via a reflective certificate
checker, not a trusted solver (`OQ-12`); relational/2-safety as re-checked
product programs, with a relational *logic* only as a reflective embedding if
ever needed (`OQ-relational`). If your instinct is "add a primitive," first ask:
*can this be data the existing kernel reasons over?*

### 7. Subsume, don't proliferate — and know what Ken is *not*

Before adding a mechanism, check whether **already-decided machinery composes to
give it**. Most "do we need X?" questions resolve to "X is X already, viewed
right." And concerns that are not Ken's belong at the **right layer**, not
bolted into Ken.

*In practice:* assuring agent outputs = capabilities + IFC + the seam, no new
mechanism (`74`); multishot continuations subsumed by interaction-trees +
generators + search-as-data (`OQ-9`); constant-time = effects + IFC + a `@ct`
label, with the *timing guarantee* routed to Ward + the toolchain (`61 §5a`);
infinitude routed to the runtime loop + Ward, not coinductive values
(`OQ-coinduction`); the test-sampling *measure* lives in deployment-adjacent
config, not source (`OQ-sampling-policy`). Also: *better is the enemy of good* —
full dynamic IFC was declined for the minimal boundary mechanism that covers the
real cases (`61 §3`).

### 8. Be honest about the boundary — over-claiming is itself a failure

A verified language that pretends to guarantees it does not have is a **security
risk**. State assumptions, leakage models, and trusted-base deltas plainly; tag
every claim with its real epistemic status; **never promote `tested`/`delegated`
to `proved`**. Prefer **loud refusal over silent degradation**.

*In practice:* the four-way status is *visible in source and exported*
(`70-behavioral`); `trusted_base_delta` lists every
postulate/hole/FFI/declassify (`63 §2`); Ward's results return as a signed
attestation, never as a Ken proof (`OQ-classical-bridge`); constant-time is
proved *relative to a stated leakage model*, with the gap disclosed (`64 §4.2`);
the heap fails loudly at its limit, never corrupts (`44`). Agent-output
*quality* is `unknown`, explicitly outside the assurance boundary (`74 §3`).

### 9. One logic, two engines — and design Ken with its sibling as a whole

Ken proves the **static, propositional, total** fragment; **Ward** (ADR 0006)
discharges the **temporal / behavioral / runtime** fragment by model-checking,
testing, and monitoring. One assertion language, two engines, two trust domains.
Separation by **role** (security/compliance author policy; implementers are
bound) is the same idea applied to authority (`ADR 0007`).

*In practice:* the seam is a **generated, broadcast export** read by a family of
downstream engines (`71`), flowing **one way** (Ken → Ward) for legibility and
soundness (`OQ-classical-bridge`). When a concern is temporal, concurrent,
empirical, or quality-judgmental, it is Ward's — keep it out of Ken's core, but
design the handoff deliberately.

### 10. Predictability is a feature

The human reasoning about code or a proof needs a **predictable substrate**.
Favor the option that makes behavior and cost easy to predict over the one that
is cleverer or marginally faster.

*In practice:* strict call-by-value evaluation (a legible cost model, no
space-leak surprises) over laziness — which is *also* a precondition for the
timing reasoning security needs (`OQ-eval-order`, `61 §5a`); one canonical
instance per type (`ADR 0008`); a single mandated formatter, no style latitude
(`31 §1a`); overflow as an explicit obligation, never silent wrapping (`OQ-1a`).

### 11. Security is tier-1 and intrinsic, built on machinery Ken already has

Security is not a later layer. Information flow, capabilities, supply-chain
re-check, and policy-as-code are **first-class**, and they reuse the structures
Ken already has (the effect/label discipline, the de Bruijn criterion, the
content-addressed store) rather than bolting on a separate system.

*In practice:* IFC is the effect system indexing labels instead of capabilities
(`61`); the `@ct` constant-time discipline is that same IFC pointed at
leakage-relevant operations (`61 §5a`); even *syntax* carries a security
property — a confusable-resistant character set so a reviewer reads exactly what
the kernel checks (`31 §1a`, no homoglyph backdoor).

### 12. Bound the untrusted at its boundary; make the trust level a typed choice

An untrusted component (a foreign engine, a fast unproved algorithm) embedded in
Ken is **strictly better than the same component in an unverified language** —
not because its internals are verified, but because Ken verifies the two things
*bracketing* it: the **provenance** of its inputs (what may reach it — IFC
labels, capabilities, `@ct`) and the **use** of its results (where they may flow
— gated out of proof-relevant positions, carrying their inputs' taint). Safety
comes from bounding the boundary, not proving the picture. And where a capability
admits both a provable core and an unprovable-but-expressive extension, that is a
**feature, not a defect**: expose the two as distinct types that carry the
tradeoff on their face, default to the provable core, and let policy govern which
tier a trust-domain may use.

*In practice:* keep foreign code **opaque and proof-gated** — never assert a spec
axiom to make its result proof-usable (that voids the bracket and is usually
false); **earn** proof-relevance by certification or proof, never assert it
(`ADR 0009`). A wrong value from the tested-not-trusted evaluator is a wrong
*value*, never a false proof, because the kernel cannot consume it as one (`18a`,
the reachability precondition). Supply a capability at the lowest tier that
unblocks it, behind a stable interface, and migrate upward —
opaque-foreign → tested/certified → proved-native — as the value justifies;
native compilation (`45`, the X-series) is what lets the proved tier *also* be
fast. `Float` equality is `NATIVE`-but-non-proof while `Int`'s is provable — the
type tells you which tradeoff you accepted (`18a §5.4`).

---

## Working constraints (process, not philosophy)

- **Clean-room, and skeptical of inherited artifacts.** Design from first
  principles + the analysis digest + permissive references *understood, never
  copied*; AGPLv3 and copyleft material is off-limits except to the Spec
  enclave for behavior/approach only (`CLEAN-ROOM.md`, `CLAUDE.md`). Treat
  any inherited design candidate with skepticism — it may reflect uncritical
  wandering, not a considered choice.
- **Record decisions where they live.** Genuine forks →
  `spec/90-open-decisions.md`; architecturally significant ones → an ADR under
  `docs/adr/`; the deciding authority is the operator. Wrap markdown at 80
  columns, and **use Mermaid for diagrams/charts** (fenced ` ```mermaid `),
  never ASCII art (`CLAUDE.md`).

## How to use this document

When you face a choice the spec does not settle: identify which principles bear
on it, reason from them explicitly, and — if the choice is significant — record
the decision and cite the principle. If two principles seem to conflict, say so
and surface it; that tension is usually where a real fork lives, and it belongs
to the operator. Do **not** optimize for effort, speed-to-build, or writer
convenience at the expense of correctness, the small TCB, or the human reader.
