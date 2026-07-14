# The trust model, the TCB, and the honest limits

> Status: **Normative.** This chapter fixes Ken's trusted computing base, the
> security reading of the kernel (authorship-independence), the trusting-trust
> invariant, and the honest limits. The limits (§4) are the **most important**
> part of the chapter — a verified language that over-claims is itself a
> security risk — and are normative, first-class claims, not commentary. ADR
> 0001 (small permanent kernel), ADR 0004 (security tier-1, Decisions 2, 6, 7).
> Conformance: `../../conformance/security/trust-model/`.

Each numbered contract below is stated with its **trust level**: whether it is a
**landed kernel producer** (a real function whose output a consumer reads), a
**by-construction / structural** property (a mechanically-auditable invariant of
the trusted code or its API, *trusted-as-code* — TCB item 1 — but not proved by
the kernel about itself), or a **doc-posture** characterization. Nothing in this
chapter is a claim the kernel *proves*; the kernel is the trust root, so its own
properties are trusted-as-code, held by audit (§3), never by self-certification.

## 1. The trusted computing base (TCB), precisely

Ken's audited boundary has **exactly** three categories
(`../10-kernel/18 §5`). Proof soundness depends on the declarations/signatures
in all three; runtime value correctness additionally relies on item 2's
interpreter semantics:

1. **The kernel** — the small, permanent Rust core (type theory + conversion +
   proof checker), including the admission gates (positivity, W-style, SCT,
   quotient respect, `18 §4.3`). The gates are trusted-as-code but **re-run on
   every input** — nothing is admitted without passing — so they add **no
   per-program assumption**; they are part of item 1, never item 3.
2. **The primitive declarations and operation registrations** — the irreducible
   primitive surface (`../10-kernel/14 §5`), each registered via
   `declare_primitive` as a `Decl::Primitive` and enumerated for audit. The
   declaration and its type are trusted. Landed `PrimReduction::Op` execution
   is in `ken-interp`, not kernel conversion; that implementation is tested
   runtime semantics, not a proof-producing TCB path.
3. **The postulates** — every assumed axiom and `foreign`/FFI signature, each
   admitted via `declare_postulate` as a `Decl::Opaque` and listed in
   `trusted_base_delta` (`../20-verification/25 §3`).

**Nothing else is trusted:** not the elaborator, the prover, Z3/cvc5, the
surface compiler, the runtime, the IFC label discipline, or the package tooling.
They all produce artifacts (core terms, certificates) that the kernel re-checks.
Definitions and inductives are **re-checked, not trusted**, so they are excluded
from the trusted base.

The kernel's `trusted_base()` ledger includes item 2 because the primitive
declaration/signature is trusted. This does not claim that the kernel executes
its registered `Op`. A wrong interpreter reduction is a wrong value and a
semantic correctness defect; it cannot manufacture a false `proved`. Moving
registered operations into conversion would change that boundary and is
K3-deferred.

### 1.1 The enumeration contract (soundness) [landed producer]

The kernel MUST enumerate items (2) and (3) on demand. The landed producer is
`GlobalEnv::trusted_base() -> Vec<GlobalId>` (`crates/ken-kernel/src/env.rs`,
`18 §5`), which returns **exactly** the `Decl::Opaque` and `Decl::Primitive`
declarations in `Σ`, excluding the prelude `Top`/`Bottom`/`tt` constants
(fixed kernel vocabulary, `16 §1.3`, not user assumptions).

Every postulate entry is readable: its `Decl::Opaque` stores the audit label
supplied at the single `declare_postulate` choke point. A surface declaration
body uses its declaration name; an instance field uses exactly
`Class.HeadType.field`; a public standalone-expression API requires its caller
to supply a stable semantic audit owner; and another elaborator- or Rust-minted
postulate uses the producer's stable declared symbol. The public owner argument
is required and non-optional, so an ownerless call is unrepresentable. None of
these labels may be inferred from an expression, source position, module
fallback, allocation index, session counter, generated counter, gensym, or
generic sentinel. Multiple occurrences under one semantic owner may share its
label while retaining distinct `GlobalId`s; the contract is label derivation,
not global label uniqueness. The ledger
therefore remains stable when unrelated source is inserted, and it names
internal producers as well as surface `Axiom` expressions.

This completeness is enforced by the data and call shapes, not convention:
`Decl::Opaque` has a required non-optional name field, and
`declare_postulate` has a required name argument. There is no optional, default,
or builder path that can admit an unnamed opaque declaration.

> **Contract TB-Sound.** For any admitted program, `trusted_base()` (the
> artifact-relative projection is `trusted_base_delta`, `25 §3`) is **empty iff
> the artifact rests on no unchecked assumption** — fully verified and confined.
> A term elaborated with no postulate, `foreign`, or hole yields an **empty**
> delta; a term carrying one postulate yields a **non-empty** delta listing
> **exactly** that `GlobalId`, whose opaque declaration carries the required
> label. The consumer reads the emptiness as the "fully verified+confined"
> signal of §5.

There are no phantom entries: the enumeration is a filter over the real `Σ`, so
it cannot report an assumption the program does not carry.

The name is audit metadata only. The kernel may expose it through the ledger,
but conversion, typing, admission, positivity, universe checking, and
elimination MUST NOT inspect it or branch on it (`../10-kernel/11 §4`, `18
§4.2`). This kernel-inertness is the structural reason that making the trust
ledger readable does not enlarge the logical power of a postulate.

### 1.2 The completeness net (no hidden assumption) [by construction]

The security claim "nothing else is trusted, and the complete assumption set is
mechanically listable" fails silently if an assumption can enter `Σ` **without**
being enumerated. It does not, and the reason is structural, not a runtime
check:

> **Contract TB-Complete.** The **only** ways to introduce an unchecked
> assumption into `Σ` are `declare_postulate` (axioms; unproved prover goals,
> `23 §1.3`; typed holes, `24 §2`) and `declare_primitive` (`14 §5`); a
> `foreign`/FFI signature (`../30-surface/38 §3`) **desugars to
> `declare_postulate`** and carries no separate privilege. Each records a
> `Decl::Opaque` or `Decl::Primitive` — which is **exactly** the set
> `trusted_base()` enumerates. The choke-point through which assumptions enter
> `Σ` *coincides with* the filter that lists them, so **no assumption can
> hide**.

**Trust level.** This is a **by-construction invariant over the admission
surface** — mechanically auditable (the assumption-introducing entry points are
a closed, small set, each landing an `Opaque`/`Primitive`), and
*trusted-as-code* as part of TCB item 1. It is **not** a theorem the kernel
proves about itself.
Its force is that the conformance corpus can drive a **real** `foreign`/hole
admission and observe it surface — the omission net — rather than accepting a
hand-inserted delta. A build or characterization that adds a new assumption path
which bypasses `Opaque`/`Primitive` would break TB-Complete; that is the
invariant to guard.

## 2. The de Bruijn criterion is a *security* property

The standard reading of "the kernel re-checks every certificate" is soundness.
The *security* reading is **authorship-independence**:

> A property holds in *your* kernel or it does not — independent of who or what
> produced the code and its proof.

A bug, or active malice, in the prover, the elaborator, an SMT solver, or an AI
code generator can cause a **failure to prove** or a **rejected certificate** —
**never a false `proved`** (`../20-verification/23 §1`). This is precisely the
property the AI era requires: you may let an unreliable or adversarial model
*generate* code and proofs at scale, and admit the result into a trusted system
**only on the kernel's terms**. The model is the generator; the kernel is the
adversary that filters it.

### 2.1 The no-provenance-channel contract [structural]

> **Contract AI-Indep.** The kernel check judgment is `check(Σ, Γ, t, A)`
> (`crates/ken-kernel/src/check.rs`, `18 §5`) — its inputs are the environment,
> context, term, and type, and **nothing else**. There is **no** provenance,
> author, or trust-level parameter, so no "this came from a trusted source"
> framing can change the verdict. A certificate of a **false** proposition is
> **rejected** regardless of any authorship claim, and never enters
> `trusted_base()` as proved; a genuine proof is **accepted** regardless of
> origin. The discriminant is the kernel verdict, not a metadata flag.

**Trust level.** This is a **structural property of the trusted check path's
signature** — there is no provenance channel to exploit because there is no
provenance input. It is *trusted-as-code* (a property of the kernel API
surface), not kernel-proved; it is what makes "never a false `proved`" hold
**uniformly** across every generator, trusted or adversarial.

## 3. Auditing the one thing you must trust

Because the kernel *is* the trust root, two obligations follow:

- **Keep it small and audit it.** The kernel is deliberately minimal (ADR 0001)
  so that one team can review it; a **published, independent audit** of the Rust
  kernel is a tier-1 deliverable, not a nicety (strategy G5). The security claim
  "trust the kernel" is only as good as that audit. This chapter and its
  conformance corpus deliver the **machine-checkable substrate** of that posture
  (the enumerable TCB, §1; authorship-independence, §2; the invariant below);
  the **external, published audit report** for consumers is the G5 capstone that
  follows, and is out of scope here (§6).
- **Defend against trusting-trust.** A self-hosting compiler invites Thompson's
  "reflections on trusting trust" attack — a compromised toolchain that
  reproduces its own backdoor. Ken's structural defence is **already chosen**:
  the small Rust kernel stays **permanent** even after the elaborator/compiler
  self-host (ADR 0001, strategy S1/S2). So the self-hosted toolchain *always*
  has an **independent second checker** — the original Rust kernel re-checks
  what the self-hosted stack produces. This is **diverse double-compilation
  built into the architecture**: a backdoor would have to compromise *two
  independent checkers* identically.

### 3.1 Invariant TT — kernel dependency independence [structural]

> **Invariant TT.** The `ken-kernel` crate's build- and dependency-closure MUST
> contain **no Ken-generated artifact**: it must not link, embed, or
> build-depend on any self-hosted or Ken-emitted output. Mechanically checkable
> on `crates/ken-kernel/Cargo.toml`'s dependency graph, it **holds** for the
> landed self-contained kernel and **fails** for any kernel that took a
> dependency on a Ken-generated crate.

**Trust level.** A **structural/architectural invariant**, mechanically checked
on the dependency graph — *not* kernel-proved. It is the property that keeps the
second checker genuinely independent (the diverse double-compilation is only as
strong as this independence). Self-host (strategy G8) is out of the current
scope, but the invariant is stated **now** as a named security invariant so the
self-host epoch cannot silently break it: a conforming presentation states TT as
an invariant to hold, and a test asserts it on the current kernel crate.

## 4. The honest limits (what a language cannot fix) [normative]

A CISO must not over-rotate on "it's verified." Ken states its boundaries
plainly; pretending these away would itself be a security failure. **These
limits are normative, externally-legible claims**: a conforming presentation of
Ken MUST surface each of §4.1–§4.5, not bury or soften it. An over-claiming
presentation — one that drops or waters down a limit — is itself a security
defect, and the conformance corpus records the limits as characterizations to
preserve (§6).

### 4.1 Spec ≠ intent — the dominant residual risk
This is **the** headline residual risk, not a footnote. The kernel proves **code
matches its specification**. It says **nothing** about whether the specification
captures your **intent**. An adversary — or an AI — can ship a proof that code
satisfies a **weak or wrong** spec. Verification *relocates* the trust question
from "is the code right?" to "is the spec right?", and the latter is a **human
judgment** no checker can make.

- **What Ken can do:** make the spec a *first-class, machine-readable,
  reviewable* artifact (the `requires`/`ensures`/refinements are the
  security-reviewed surface, not the code body); support spec coverage metrics
  and spec **mutation testing** (does the proof still pass if the spec is
  weakened? then it is too weak); and, in the agentic loop, route the human's
  scarce attention to **reviewing the spec**, not re-reading generated code.
- **What Ken cannot do, and does not claim:** decide that a spec is the *right*
  spec. That is a **complementary engineering-discipline project** — the
  principal-engineer / software-architect / SWEBOK realm of turning requirements
  into specifications — and it sits *beside* Ken, not inside it. Ken makes the
  spec the artifact that discipline operates on; it does not replace the
  discipline.

### 4.2 Side channels and resource bounds
Functional + flow proofs cover *what is computed* and *where data flows*, not
*how long it takes* or *how much it costs*. **Constant-time** (timing side
channels in crypto) is handled by a **layered split** (decided, `61 §5a`): Ken
**statically** guarantees the *source-level precondition* — a `@ct`-labeled
value never steers a leakage-relevant operation (branch/index/var-time),
enforced by typing — but the **timing guarantee itself is
hardware/codegen-relative** (cache lines, `cmov`-vs-branch lowering) and lives
**below Ken**. So the running-time guarantee is **delegated to `Ward` + the
toolchain**: CT-preservation through compilation plus empirical timing
validation under an explicit **leakage model** on a **platform**, recorded in
the discharge attestation (`63 §5a`). Ken's static part is a *necessary
precondition*, honestly not the whole guarantee — the leakage model bounds the
strength of every CT claim. **Worst-case time/space bounds** (a dependency as a
DoS vector) remain a complexity discipline the totality checker does not provide
(termination ≠ cheapness, `../40-runtime/43`), **optional/research**.

### 4.3 The kernel, the FFI, and the runtime stay trusted
Proof covers the **pure core**. The kernel itself (§3), `foreign` C code
(`../30-surface/38 §3`, a listed postulate — it surfaces in `trusted_base()` by
TB-Complete), and the native runtime remain trust assumptions — minimised and
*listed*, but not *proven*. A `pure` annotation on a `foreign` is a **claim**,
not a check.

### 4.4 The social / registry layer
Namespace squatting, dependency confusion, key compromise, and registry
governance (`63 §6`) live **above** the language and need ecosystem policy. Ken
makes that layer *effective* (it has real attestations to police) but does not
replace it.

### 4.5 Regulated industries
In safety-critical sectors (DO-178C avionics, ISO 26262 automotive, IEC 62443
industrial, medical-device software), formal verification is moving from luxury
to **compliance expectation**, and copilot output explicitly **cannot** be
treated as self-certifying. Ken fits that trajectory — and can *emit* assurance
artifacts (proofs, deltas, provenance) those processes consume — but it **does
not eliminate process**. Ken is a powerful input to certification, not a
substitute for it.

## 5. What a consumer checks (the four-point summary)

Consuming a Ken artifact safely:

1. **Content hash** matches the lock — *identity* (`63 §3`). ✅ in design.
2. **Kernel re-check** of the proof bundle passes — *correctness*, on **your**
   kernel (`63 §1`) — the authorship-independent admission of §2 (AI-Indep).
   ✅ in design.
3. **`trusted_base_delta` audit** against policy — *assumptions*, read via the
   TB-Sound contract (§1.1): **empty = fully verified+confined**; non-empty =
   exactly what you inherit, incl. FFI and declassifications, complete by
   TB-Complete (§1.2). ✅ landed producer (`trusted_base()`).
4. **Provenance signature + SLSA** verify — *origin and build* (`63 §5`).
   ✅ in design; `OQ-provenance` decided (2026-06-27: keyless sigstore/cosign +
   in-toto/SLSA), implementation deferred.

## 6. What WS-K / tooling must deliver here

- **Delivered here (machine-checkable substrate).** The enumerable trusted base
  `trusted_base()` (TB-Sound, TB-Complete); the authorship-independent check
  path (AI-Indep); the kept-independent permanent Rust kernel as a named
  invariant (Invariant TT); and the **normative honest-limits** (§4) recorded as
  characterizations the conformance corpus preserves. Acceptance: the complete
  assumption set behind any artifact is mechanically listable and complete; the
  check verdict is independent of authorship; the kernel's dependency closure is
  independent of Ken-generated output; each honest limit is surfaced, not
  buried.
  Conformance: `../../conformance/security/trust-model/`.
- **Deferred (G5 governance capstone, flagged to the operator).** The
  **external, published, independent kernel-audit *report*** for consumers (§3):
  it is (a) T4-class public documentation (human-facing, not yet prioritized
  under the current re-scope), and (b) a **governance call** — external auditor
  vs. an internal independent reviewer, and the publication decision — that is
  the operator's, not an autonomous one. A lightweight agent-context **TCB
  inventory** (kernel modules + the `trusted_base()` surface) MAY accompany the
  conformance seed; the polished external report is **out of scope** here. The
  machine-checkable posture above is the substrate that report would attest to.
