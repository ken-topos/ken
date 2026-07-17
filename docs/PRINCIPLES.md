# Ken design principles — the reasoning charter

> **Audience: every agent working on Ken — especially Steward, Spec, and
> Implementation.** These are the *implicit* values of the project, made
> explicit. They emerged while resolving the open-decisions register
> (`spec/90-open-decisions.md`, ADRs 0005–0008); decision after decision turned
> on the same handful of commitments. **When the spec dictates an answer, follow
> it. When it does not, reason from these.** They are priors for judgment, not a
> substitute for it.

---

> ### ⏳ Transient principle T — no backwards compatibility during initial development
>
> **This is a *time-bound* principle, not a permanent charter value.** It holds
> only while Ken is in initial development with **no real users and no consumers
> of its outputs**, and it will be revised or retired the moment either exists.
> Until this block is rewritten, treat it as binding.
>
> **Ken is in initial development. There is no backwards compatibility to preserve
> and no existing use to protect. Do not keep an old or superseded form in
> parallel with an improved one** — no `V1` retained "byte-identical" beside a
> `V2`, no second wire codec kept alongside its replacement, no legacy arm held
> "just in case." The *only* thing a parallel old form can buy is compatibility,
> and there is **no one to be compatible with**: no users of the language, no
> consumers of its outputs. So a preserved old form is pure duplication — extra
> code, extra TCB, extra audit surface — bought for a beneficiary that does not
> exist. When a form is improved, **replace it and delete the old one in the same
> change.**
>
> *In practice:* the effect-observation schema collapses to a **single**
> versionless form — one `EffectEvent` carrying role-labelled resource bindings
> (the single-resource case is the degenerate one), one observation/wire encoding,
> one obligation — rather than a `V1` schema preserved beside an additive `V2`
> (operator ruling, 2026-07-17). The additive "keep the prior WP's types
> byte-identical" tactic that keeps an individual merge small is legitimate
> *within* a work package, but the parallel forms it creates must be **subsumed
> before the arc closes**, never shipped as the resting state.
>
> *Relationship to the permanent principles:* this is #7 (subsume, don't
> proliferate) and #4 (correct over expedient) with their one standing
> exception — backwards compatibility — **removed for the duration**. Once Ken has
> users or output consumers, compatibility becomes a real value and this block is
> rewritten to say when parallel forms *are* warranted. Until then, "should we
> keep the old form too?" is already answered: **no.**

---

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

**Ken's intended domain is broad but bounded.** Its **lower bound is
systems-*adjacent*** — one notch above true systems programming; the
content-addressed managed heap with optional, semantics-invisible reclamation is
the *right* substrate for that, not a compromise (`44 §3`). Its **upper bound
reaches application, edge, web, and mobile** compilation targets. Across that
range Ken is a verified software-engineering language — not a bare-metal systems
language, and not tied to one runtime or target.

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

**The one case this does not cover — see #15.** When the fact is a guarantee
**the implementation itself makes** (an opaque primitive's length, structure, or
decidable equality), it is *not* data the kernel can reason over at any price,
and refusing the primitive does not avoid the trust — it **scatters** it, as an
`Axiom` in every consumer. Reflection is the default; #15 is the narrow, tested
exception, and its tell is that **the consumers are all writing the same
`Axiom`.**

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

### 13. Fix the defect at its layer — never compensate upward

Ken is a stack of layers, each trusting the one beneath it: kernel ⊂ elaborator
⊂ prover ⊂ standard library ⊂ surface program. **A defect in a lower layer is
fixed in that layer.** Compensating for it higher up — a library workaround, a
surface-syntax dance, a special case that routes around the broken path — leaves
the defect live for every other client, silently corrupts the lower layer's
contract, and grows incidental complexity that outlives the bug. The
higher-layer patch is the *expedient* (principle 4); worse, a masked lower defect
is a hidden lie about what the layer guarantees — over-claiming (principle 8) one
level down. The layered TCB (principle 5) exists precisely so a fix has an
unambiguous home; put it there.

*In practice:* when the kernel's `check` of a `let`-expression verified the body
against the wrong type, the fix belonged in the kernel's checker arm — not in a
parenthesization ritual in the library that happened to trip over it. A higher
layer may *route around* a lower defect only as a **disclosed, temporary** hedge,
with the root fix filed and tracked — never as the permanent answer, and never in
a way that hides the defect. The tell that you are about to violate this: a
workaround starts to feel permanent, or you reach for "just handle it here
instead." That feeling is the signal the root fix was skipped — stop and push the
fix down to where the defect lives.

### 14. Nothing required lives in a comment — express it in the language

A comment is **unchecked prose**: nothing verifies it, nothing re-checks it, and
it drifts silently from the code it annotates. So a *required* fact — a contract,
an invariant, a proof obligation, a trust boundary — must be expressed in the
**language proper**, where the elaborator and kernel check it, never in a comment
only a human might read. Making a comment mandatory is a two-fold failure: it
puts the load-bearing thing where nothing checks it (over-claiming, principle 8),
and — if the fact genuinely has no language-proper form — it hides a real gap in
the language behind prose instead of fixing it at its layer (principle 13). A
comment is therefore **never mandatory**; at most an optional pointer. This is
not "fewer comments": genuine narrative — proof strategy, naming rationale, why a
thing exists — is welcome, it simply is never the artifact anything *depends* on,
and in a literate `.ken.md` it belongs in the surrounding Markdown, not a `--`
inside the fence.

*In practice:* a contract/invariant is `requires`/`ensures` or a refinement type;
a proposition is `law`/`prop`/`lemma` discharged by `prove`; a trust boundary is
`Axiom` recorded in `trusted_base_delta` (`63 §2`) — the checked construct *is*
the record, and a comment restating it is redundant. When a required fact has no
language-proper home, that is a signal to **extend the language** (or file the
gap), never to enshrine the fact in a comment. The catalog style guide follows
this: proof-review information lives in the entry's Ken, and comments/prose are
reserved for the narrative that genuinely cannot be checked (`07 §8`).

### 15. A guarantee the implementation makes must be reasonable-from in Ken

Where the implementation **already guarantees** a fact — a `Bytes` value *has* a
length; a `String` *is* a sequence of `Char`s — the language must expose the
**proposition or structure that lets a program reason from it**. Where it does
not, every consumer who needs that fact must **postulate** it, and the cost is
not one assumption: it is **one per call site, forever**.

*In practice:* **prefer a fixed extension of the trusted base — one entry per
built-in — over an unbounded number of `Axiom`s added by consumers of Ken**
(operator ruling, 2026-07-14). Postulating does **not** avoid TCB growth: a
surface `Axiom` is `declare_postulate` → `Decl::Opaque` → **a real
`trusted_base()` entry**. So the two routes differ only in *shape*. One is
**bounded, reviewed once, and paid by us**; the other is **unbounded, paid by
everyone downstream, and never reviewed at all**. Take the bounded one.

**This is not a licence to add primitives — #6 still binds.** If the fact can be
**data the kernel reasons over, reflect it**. This principle governs only the
case where the guarantee is **the implementation's own**, and is therefore *not*
derivable inside Ken at any price: an opaque primitive's length, its structure,
its decidable equality. There the choice was never *primitive vs. reflection* —
it is **primitive vs. every consumer postulating the same fact independently**.
The tell that you are in this case, and not in #6's: **the consumers cannot
prove it, cannot reflect it, and are all writing the same `Axiom`.**

*The landed instance, and the one that was missing:* `String` is opaque, yet
`string_to_list_char : String → List Char` (with its inverse, `37 §2.3`) exposes
its structure as an inductive view — so programs **recurse structurally instead
of assuming**. `Bytes` had no such view: `bytes_length : Bytes → Int`, with `Int`
opaque and **no `Int → Nat`**, so a byte length could be *named* but never
*used*. Every consumer therefore carried a cached `Nat` plus an `Axiom` that the
cache agreed with the real length (CAT-5's `Source`, CC3's `ArgBytes`). **Four
consumers paid that tax before the asymmetry was recognized as a defect rather
than a fact of life.** The fix is the parallel view, not a fifth carrier.

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
