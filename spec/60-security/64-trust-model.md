# The trust model, the TCB, and the honest limits

> Status: **DRAFT v0**. Normative for the trusted-base definition and the
> security reading of the kernel; the limits section is the **most important**
> part of this chapter — a verified language that over-claims is itself a
> security risk. ADR 0004 Decisions 2, 6, 7.

## 1. The trusted computing base (TCB), precisely

Soundness — and therefore every security guarantee that rests on a proof —
depends on **exactly** three things (`../10-kernel/18 §5`):

1. **The kernel** — the small, permanent Rust core (type theory + conversion +
   proof checker).
2. **The primitive reductions** — the audited operations on literals
   (`../10-kernel/14 §5`).
3. **The postulates** — every assumed axiom and `foreign`/FFI signature, each
   listed in `trusted_base_delta` (`../20-verification/25 §3`).

**Nothing else is trusted:** not the elaborator, the prover, Z3/cvc5, the
surface compiler, the runtime, the IFC label discipline, or the package tooling.
They all produce artifacts (core terms, certificates) that the kernel re-checks.
The kernel can **enumerate (2) and (3)** on demand, so the *complete* set of
unchecked assumptions behind any artifact is mechanically inspectable — the
foundation of the supply-chain audit (`63`).

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

## 3. Auditing the one thing you must trust

Because the kernel *is* the trust root, two obligations follow:

- **Keep it small and audit it.** The kernel is deliberately minimal (ADR 0001)
  so that one team can review it; a **published, independent audit** of the Rust
  kernel is a tier-1 deliverable, not a nicety (strategy G5). The security claim
  "trust the kernel" is only as good as that audit.
- **Defend against trusting-trust.** A self-hosting compiler invites Thompson's
  "reflections on trusting trust" attack — a compromised toolchain that
  reproduces its own backdoor. Ken's structural defence is **already chosen**:
  the small Rust kernel stays **permanent** even after the elaborator/compiler
  self-host (ADR 0001, strategy S1/S2). So the self-hosted toolchain *always*
  has an **independent second checker** — the original Rust kernel re-checks
  what the self-hosted stack produces. This is **diverse double-compilation
  built into the architecture**: a backdoor would have to compromise *two
  independent checkers* identically. Maintaining that independence (the Rust
  kernel never depends on Ken-generated artifacts) is a stated security
  invariant, not an accident of bootstrapping.

## 4. The honest limits (what a language cannot fix)

A CISO must not over-rotate on "it's verified." Ken states its boundaries
plainly; pretending these away would itself be a security failure.

### 4.1 Spec ≠ intent — the dominant residual risk
The kernel proves **code matches its specification**. It says **nothing** about
whether the specification captures your **intent**. An adversary — or an AI —
can ship a proof that code satisfies a **weak or wrong** spec. Verification
*relocates* the trust question from "is the code right?" to "is the spec
right?", and the latter is a **human judgment** no checker can make.

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
(`../30-surface/38 §3`, a listed postulate), and the native runtime remain trust
assumptions — minimised and *listed*, but not *proven*. A `pure` annotation on a
`foreign` is a **claim**, not a check.

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
   kernel (`63 §1`). ✅ in design.
3. **`trusted_base_delta` audit** against policy — *assumptions* (empty = fully
   verified+confined; non-empty = exactly what you inherit, incl. FFI and
   declassifications). ✅ in design.
4. **Provenance signature + SLSA** verify — *origin and build* (`63 §5`). ⬜
   `OQ-provenance` — the one piece still to add.

## 6. What WS-K / tooling must deliver here

The enumerable trusted base (`trusted_base()`); the kept-independent permanent
Rust kernel (the trusting-trust invariant, §3); the published kernel-audit
posture; and the honest-limits documentation as a first-class, externally-facing
artifact (not buried). Acceptance: the complete assumption set behind any
artifact is mechanically listable; the self-hosted toolchain's output is
re-checkable by the independent Rust kernel. Conformance + posture:
`../../conformance/security/trust-model/` and the public security documentation
(strategy T4).
