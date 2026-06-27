# Judgments, the checking algorithm, and the kernel API

> Status: **DRAFT v0**. Normative. Collects the judgment forms, the conversion
> ("switch") rule that ties typing to `17`, the **bidirectional** check/infer
> algorithm, the kernel's **Rust API surface** (the trusted boundary other
> components call), the trusted-base accounting, and the metatheory status. With
> `11`–`17` this completes the kernel contract (WS-K).

## 1. Judgment forms

The kernel decides four judgments, all relative to a global environment `Σ` (`11
§4`):

```
⊢ Γ ctx              Γ is a well-formed context
Γ ⊢ A type           A is a type      (≡ Γ ⊢ A : Type ℓ for some ℓ)
Γ ⊢ t : A            t has type A
Γ ⊢ a ≡ b : A        a and b are definitionally equal at A   (17)
```

Context well-formedness threads through the binder rules: `⊢ · ctx`, and `⊢ (Γ,
x : A) ctx` when `⊢ Γ ctx` and `Γ ⊢ A type`. There are no interval/cofibration
entries (ADR 0005); the context is term variables only.

## 2. The rules, collected

The introduction/elimination/formation rules are stated in the per-feature
chapters and are the typing relation's clauses:

| Connective | Rules in |
|---|---|
| Universes `Type ℓ`, levels, Ω | `12` |
| Π (functions) | `13 §1` |
| Σ (pairs) | `13 §2` |
| Inductives `D`, `cₖ`, `elim_D` | `14` |
| `Eq`, `refl`, `J` | `15`, `16 §2` |
| `cast`, Ω + proof irrelevance, quotients `A/R`, truncation `‖A‖` | `16` |
| Primitives | `14 §5` |

One rule lives here because it ties typing to conversion (`17`) — the
**switch**:

```
  Γ ⊢ t : A      Γ ⊢ A ≡ B : Type ℓ
  ─────────────────────────────────────  (Conv)
  Γ ⊢ t : B
```

A term keeps its type up to definitional equality. (Conv) is where the whole
conversion machinery of `17` is invoked during checking. Under cumulativity
(OQ-2) this generalizes to a subsumption `Γ ⊢ A ≤ B`; the DRAFT uses equality.

## 3. The bidirectional algorithm

The kernel implements typing as two mutually-recursive, syntax-directed modes:

```
Γ ⊢ t ⇐ A        check: given Γ, t, and a known type A, verify t : A
Γ ⊢ t ⇒ A        infer: given Γ and t, produce the unique A with t : A
```

This split keeps the algorithm deterministic and minimizes annotations.

**Inferring terms** (the head is type-determining):
- variable `x` → its type from Γ; constant `c` → its type from Σ.
- application `f u`: infer `f ⇒ (x:A)→B`; check `u ⇐ A`; result `B[u/x]`.
- projection `p.1`/`p.2`: infer `p ⇒ (x:A)×B`; results `A`, `B[p.1/x]`.
- `elim_D M m̄ … s`, `cast A B e t`, quotient-elim `elim_/`: infer from the
  eliminated/operated subject and the annotations.
- ascription `(t : A)`: check `A ⇐ Type ℓ`, then `t ⇐ A`, result `A`.
- `Type ℓ`, `Ω`, `(x:A)→B`, `(x:A)×B`, `Eq …`, `A / R`, `D …`: infer their
  universe (formation rules), result `Type (…)` or `Ω`.

**Checking terms** (the type drives the rule; this is where η enters):
- `λ (x:A). t ⇐ (x:A')→B`: check `A ≡ A'`, then `t ⇐ B` under `x:A`.
- `(a,b) ⇐ (x:A)×B`: check `a ⇐ A`, then `b ⇐ B[a/x]`.
- `refl a ⇐ Eq A a a`; any proof of `Eq A a b` is checked against the
  proposition `Eq A a b` (which computes by `16 §2`) — and since `Eq : Ω`, proof
  irrelevance (`16 §1`) means the *content* is not compared.
- `[a] ⇐ A / R`; constructor applications and quotient classes: checked against
  their target type (`14`, `16 §5`).
- **mode switch (fallback):** any other `t ⇐ A` infers `t ⇒ A'` and checks `A ≡
  A'` via conversion (`17`) — this is the algorithmic form of (Conv) and the
  single place conversion is called during checking.

The elaborator (`../30-surface/39-elaboration.md`) produces fully-explicit core
terms with enough ascription that infer/check never need to *guess* (no
unification in the kernel); any remaining annotation gaps are an elaborator bug,
not a kernel responsibility.

## 4. The kernel API (the trusted boundary)

Everything outside the kernel reaches it only through this surface. The API is
**total and decidable**: every call halts with a definite result. Signatures are
illustrative Rust (`ken-kernel`); names are normative, types are indicative.

```rust
// Environment construction (each call re-checks; nothing is trusted on input).
fn declare_def(env, name, ty: Term, body: Term) -> Result<(), KernelError>;
   //  checks `ty type`, `body ⇐ ty`, runs SCT (17 §4); admits transparent.
fn declare_postulate(env, name, ty: Term) -> Result<(), KernelError>;
   //  checks `· ⊢ ty type`; admits OPAQUE; records it in the trusted base (§5).
fn declare_inductive(env, decl: InductiveDecl) -> Result<(), KernelError>;
   //  checks signatures, STRICT POSITIVITY (14 §2); generates cₖ and elim_D.
fn declare_primitive(env, name, ty: Term, red: PrimReduction) -> Result<(), …>;
   //  checks `· ⊢ ty type`; registers the reduction in the trusted base (§5).

// Core judgments (relative to a checked env + a context).
fn infer(env, ctx, t: Term)            -> Result<Term /*type*/, KernelError>;
fn check(env, ctx, t: Term, ty: Term)  -> Result<(), KernelError>;
fn convert(env, ctx, ty: Term, a, b: Term) -> bool;   // definitional eq (17)
fn whnf(env, ctx, t: Term)      -> Term;              // weak-head normal form
fn normalize(env, ctx, t: Term) -> Term;              // full normal form (NbE)

// Prover / proof checker use this (a proof IS a term; checking IS typing).
fn check_proof(env, ctx, proof: Term, goal: Term) -> Result<(), KernelError>;
   //  ≡ check(env, ctx, proof, goal); goal is the proposition (12 §5).
```

- **`check_proof` is just `check`.** The prover
  (`../20-verification/23-prover.md`) returns a certificate *term*; the kernel
  re-derives its type — the de Bruijn criterion in one line. A wrong certificate
  fails `check`; it cannot make a false proposition inhabited.
- The API never returns a *partial* result or an `unknown`: those are
  verification-layer concepts (`../20-verification/24-diagnostics.md`). The
  kernel answers typed/not-typed, equal/not-equal.
- `KernelError` carries a **minimal, precise** reason (the failing subterm and
  the two types that did not convert). Turning that into a human/agent
  diagnostic is V4's job, not the kernel's.

## 5. The trusted base (what soundness actually rests on)

Soundness of any Ken program rests on exactly:

1. **The kernel code** implementing §1–§4 and `11`–`17` (the Rust core).
2. **The primitive reductions** registered via `declare_primitive` (`14 §5`) —
   each must be a correct partial function on literals.
3. **Any postulates** admitted via `declare_postulate` — each is an *assumed*
   axiom; a postulate of an empty type would make the system inconsistent.

Nothing else is trusted: not the elaborator, the prover, Z3, the surface
compiler, or the runtime. The kernel MUST be able to **enumerate (2) and (3)**
on request (e.g. `env.trusted_base()`), so a reviewer or an agent can see the
complete set of unchecked assumptions a given program depends on. Idiomatic Ken
adds **no** postulates; classical axioms, if used, appear here and are visible
(`12 §5.2`).

## 6. Metatheory status (honest accounting)

The kernel's soundness commitments (`README.md §5`) and their current status:

| Commitment | Status (DRAFT) |
|---|---|
| No `Type:Type` / universe consistency | **By construction** (`12`); tested. |
| Subject reduction | **Argued** from the rules; to be mechanized. |
| Confluence / unique normal forms | **Argued** (standard for this calculus). |
| Strong normalization of the core | **Argued** (β/ι/η/obs); the hard metatheorem. |
| δ-termination → decidable checking | **By the SCT gate** (`17 §4`); tested. |
| Canonicity (closed terms compute) | **Required + tested** (`16 §9`, observational). |
| Decidable conversion | **Proven** for OTT (`TTobs`/`CICobs`, ADR 0005); Ken follows. |
| Consistency (no closed `· ⊢ p : ⊥`) | **Argued** from SN + canonicity. |

"Argued" means there is a standard proof for systems of this shape
(observational TT — `TTobs`/`CICobs` — + inductives + a terminating δ;
canonicity and decidable conversion are *proven* for OTT, ADR 0005) and Ken
intends to *follow* it, not that Ken has a machine-checked proof yet. A
mechanized kernel-soundness proof is a later goal (strategy G5 documents the
story; full mechanization is post-self-host, `02 §5`). This table is the
kernel's "known-risk register"; the conformance corpus exercises each commitment
behaviorally even where the metatheorem is not yet mechanized.

## 7. What the kernel checks here

A conforming kernel MUST implement the four judgments (§1) with the collected
rules (§2) including (Conv); the bidirectional infer/check algorithm (§3) with
the single conversion call at the mode switch; the API (§4) as a **total,
decidable** surface whose every constructor entry re-checks its input and gates
with positivity/SCT; and the trusted-base enumeration (§5). It MUST NOT contain
unification, implicit insertion, error recovery, or proof search — those belong
to untrusted layers. Conformance: `../../conformance/kernel/judgments/` —
infer/check round-trips, the (Conv) switch, a certificate-checking case (prover
output re-checked), rejection of an ill-typed term with a minimal error, and a
`trusted_base()` enumeration test.
