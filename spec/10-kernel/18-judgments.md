# Judgments, the checking algorithm, and the kernel API

> Status: **K-api elaborated** (implementation-ready). Normative. Collects the
> judgment forms, the conversion ("switch") rule that ties typing to `17`, the
> **bidirectional** check/infer algorithm as defensive pseudocode, the kernel's
> **Rust API surface** (the trusted boundary other components call) with a
> per-entry contract, the trusted-base accounting, and the metatheory status.
> With `11`–`17` this completes the kernel contract (WS-K) — it is the
> *contract* the elaborator (V0), interpreter (X1), prover (V-series), and
> tooling code against, and the surface a published kernel audit (Sec4)
> scrutinizes.
>
> **Freeze status (read before treating §4 as the audited TCB boundary).** This
> chapter is *authored* against the **complete kernel** — the admission gates as
> the landed per-feature chapters define them (`14 §8`/`§8.4` positivity +
> W-style, `16 §5.1` quotient respect, `17 §4` SCT), which it **cites** rather
> than restates. Its API surface is correct-by-citation regardless of build
> timing. But the *freeze* — the claim "this is the audited, stable TCB boundary
> the code actually exposes" — is **contingent**: it holds only once the landed
> `ken-kernel` matches the cited gates, i.e. after **K1.5-build** (W-style
> admission + the `method_type` indexed-family eliminator fix,
> `dec_2vc6ytrbcbfc5`) **and K2c-series-2-build** (non-Ω quotient respect)
> merge. The K-api merge
> Decision is **gated on those two builds being green-and-merged** (§4.6); until
> then the on-`main` code still carries the pre-K1.5 blanket Π-bound-recursion
> reject and the pre-series-2 non-Ω-quotient reject. This is the **inverse of a
> stale-frame**: here the landed *spec* leads and the *code* trails an in-flight
> merge — the contract follows the spec (the authority), never the transient
> code state.

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
conversion machinery of `17` is invoked during checking. Ken is non-cumulative
(OQ-2 DECIDED; `10-kernel/12 §3`), so (Conv) uses definitional equality, not
subsumption; any required universe lift is inserted explicitly by the
elaborator. There is no subtype relation `Γ ⊢ A ≤ B` in the kernel.

## 3. The bidirectional algorithm

The kernel implements typing as two mutually-recursive, syntax-directed modes:

```
Γ ⊢ t ⇐ A        check: given Γ, t, and a known type A, verify t : A
Γ ⊢ t ⇒ A        infer: given Γ and t, produce the unique A with t : A
```

The split keeps the algorithm **deterministic** (the syntactic form of `t`, or
of the type `A` in checking mode, selects exactly one rule) and minimizes
annotations. The two modes are the public `infer`/`check` entries (§4). The
pseudocode below is **defensive** — it is written as the kernel computes,
including the failure branches, so it is implementable as-is. It is the *control
flow*; the per-head typing/level rules it dispatches to live in the cited
chapters (`12`–`16`) and are not restated here.

**Notation.** `whnf`, `conv`, `convType`, `convLevel` are the conversion
primitives of `17 §3` (the public names are `whnf`, `convert`, `convert_type`,
`level_eq` — §4). `ctx, A` extends the context with a fresh variable of type `A`
(de Bruijn index 0). `B[u/0]` substitutes `u` for index 0. `inferUniv(t)` is the
helper `A := infer(t); A' := whnf(A); require A' = Type ℓ or Ω_ℓ; return ℓ`
(used wherever a position must be a type), failing `UniverseInconsistency`
otherwise.

### 3.1 `infer` — the type-determining heads

`infer` dispatches on the head of `t`. The **inferable** heads are those whose
type is determined by `t` alone (a variable's binding, a former's formation
rule, an elimination's annotations). The **non-inferable** heads — `λ`, pair,
`refl`, a quotient class `[a]`, a truncation element — carry no type to read off
and **fail in infer mode** (the caller reaches them in checking mode, §3.2).

```
function infer(env, ctx, t):                       // Γ ⊢ t ⇒ A
  case head(t) of
    Var(i):           return ctx.lookup(i)         // or VarOutOfScope if i ∉ Γ
    Const(c, ℓ̄):      (Δℓ, A) := Σ.const_type(c)    // 11 §4
                      require |ℓ̄| = |Δℓ|            // else LevelArityMismatch
                      return A[ℓ̄ / Δℓ]
    Type(ℓ):          return Type(suc ℓ)           // 12 §1  (no Type:Type)
    Omega(ℓ):         return Type(suc ℓ)           // 12 §5  Ω_ℓ : Type (suc ℓ)
    Pi(A, B):         ℓ₁ := inferUniv(A)
                      ℓ₂ := inferUniv₍ctx,A₎(B)
                      return Type(max ℓ₁ ℓ₂)        // 13 §1  (predicative)
    Sigma(A, B):      ℓ₁ := inferUniv(A)
                      ℓ₂ := inferUniv₍ctx,A₎(B)
                      return Type(max ℓ₁ ℓ₂)        // 13 §2
    App(f, u):        F := whnf(infer(f))
                      require F = Pi(A, B)           // else NotAFunction{f}
                      check(u, A)
                      return B[u/0]
    Proj1(p):         P := whnf(infer(p)); require P = Sigma(A,B)  // NotASigma
                      return A
    Proj2(p):         P := whnf(infer(p)); require P = Sigma(A,B)
                      return B[Proj1(p) / 0]
    Ascript(t', A):   inferUniv(A); check(t', A); return A
    IndFormer/Constructor/Eq/Cast/Quot/Trunc:
                      // formation/intro/elim typing of the cited chapter:
                      // 14 (D, applied cₖ), 15/16 §2 (Eq formation),
                      // 16 §3 (cast), 16 §5 (A/R), 16 §6 (‖A‖). Returns the
                      // former's universe (Type(…)/Ω_…) or the result type.
                      // (refl is a *checking* form — see §3.2, not here.)
    Elim{M, m̄, ī, s} / QuotElim{M, f, r, s} / TruncElim:
                      // check motive/methods/indices/scrutinee against the
                      // eliminator schema (14 §3 / 16 §5.1 / 16 §6); the
                      // result is the motive applied to the indices + scrutinee.
                      // Quotient elim at a non-Ω motive checks `r` against the
                      // respect schema (16 §5.1) before admitting the elim.
    Lam | Pair | Refl | QuotClass | TruncElt:
                      fail "not inferable — supply a checking type"
```

`infer` returns the **unique** type up to conversion (Ken has no subtyping, so
there is no choice to make). Every `require` that fails returns a `KernelError`
(§4) naming the offending subterm.

### 3.2 `check` — the type-driven heads and the mode switch

`check` dispatches on the **type** for the heads that need one (this is where η
enters and where the non-inferable intro forms are handled); everything else
falls through to the **mode switch**.

```
function check(env, ctx, t, A):                    // Γ ⊢ t ⇐ A
  W := whnf(A)
  case (t, W) of
    (Lam(_, b), Pi(A₁, B)):                        // 13 §1; domain from the type
        check₍ctx,A₁₎(b, B)                         // η is consumed by conv, not here
        return Ok
    (Pair(a, b), Sigma(A₁, B)):                    // 13 §2
        check(a, A₁); check(b, B[a/0]); return Ok
    (Refl(a), Eq(A₁, x, y)):                        // 15 §2
        check(a, A₁); require conv(A₁, x, a) ∧ conv(A₁, y, a)
        return Ok                                  // Eq : Ω ⇒ content irrelevant
    (QuotClass(a), Quot(A₁, R)):  check(a, A₁); return Ok      // 16 §5
    (TruncElt(a), Trunc(A₁)):     check(a, A₁); return Ok      // 16 §6
    // intro forms whose target is an inductive/primitive are checked by their
    // formation rule against W (14, 14 §5).
    _otherwise:                                    // ── the mode switch ──
        A' := infer(t)                             // (Conv), algorithmic form
        if convType(A, A') then return Ok
        else fail TypeMismatch{ expected: A, found: A' }
```

The `_otherwise` arm is the **single place conversion is invoked during
checking** — the algorithmic form of (Conv) (§2). It infers the term's type and
asks `convType` (= `convert_type`, `17 §3.3`) whether the expected and inferred
types are definitionally equal. Because conversion is **total and decidable**
(`17 §5`, via the SCT gate `17 §4`), the switch always halts with a definite
yes/no — there is **no third "unknown"** outcome in the kernel. A checking-mode
proof of `Eq A a b` reduces, after the `Refl` rule above, to checking against
the proposition `Eq A a b` (which computes by `16 §2`); since `Eq : Ω`, proof
irrelevance (`16 §1`) means the proof *content* is never compared.

### 3.3 No unification, no guessing

The elaborator (`../30-surface/39-elaboration.md`) produces fully-explicit core
terms with enough ascription that `infer`/`check` never need to *guess*: there
is **no unification, no metavariable solving, no implicit insertion** in the
kernel. A non-inferable head reached in infer mode (§3.1) is an elaborator bug
surfacing as a `KernelError`, not a gap the kernel fills. This is a load-bearing
TCB-minimality property (§5): a kernel that "helpfully" solved a missing
annotation would have absorbed elaborator logic into the trusted base.

## 4. The kernel API (the trusted boundary)

Everything outside the kernel reaches it only through this surface. The API is
**total and decidable**: every call halts with a definite result. The signatures
below are the **landed `ken-kernel` surface** (grounded against the code, not an
idealized sketch); names and shapes are **normative**. Two conventions the
contract depends on: the kernel keys every declaration on a `GlobalId` it
allocates (source binding remains the elaborator's job; the postulate
declarator additionally requires the kernel-inert audit label pinned below),
and all term inputs are **by reference** (`&Term`); `env` is `&mut GlobalEnv`
for the declarators (they extend Σ) and `&GlobalEnv` everywhere else.

### 4.1 The stable surface

```rust
// ── Environment construction — each call re-checks; nothing trusted on input.
//    Every declarator returns the fresh GlobalId (Err ⇒ Σ left unchanged).
fn declare_def(env: &mut GlobalEnv, level_params: Vec<LevelVar>,
               ty: Term, body: Term) -> KernelResult<GlobalId>;
fn declare_recursive_group(
        env: &mut GlobalEnv, specs: Vec<(Vec<LevelVar>, Term)>,
        bodies_fn: impl FnOnce(&[GlobalId]) -> Vec<Term>)
        -> KernelResult<Vec<GlobalId>>;
fn declare_inductive(env: &mut GlobalEnv,
        build: impl FnOnce(GlobalId) -> InductiveSpec) -> KernelResult<GlobalId>;
fn declare_postulate(env: &mut GlobalEnv, name: String,
                     level_params: Vec<LevelVar>, ty: Term)
                     -> KernelResult<GlobalId>;
fn declare_primitive(env: &mut GlobalEnv, level_params: Vec<LevelVar>,
                     ty: Term, reduction: PrimReduction) -> KernelResult<GlobalId>;

// ── Core judgments — relative to a checked env + a context; terms by-ref.
fn infer(env: &GlobalEnv, ctx: &Context, t: &Term) -> KernelResult<Term>;
fn check(env: &GlobalEnv, ctx: &Context, t: &Term, ty: &Term) -> KernelResult<()>;
fn convert(env: &GlobalEnv, ctx: &Context, ty: &Term, a: &Term, b: &Term) -> bool;
fn convert_type(env: &GlobalEnv, ctx: &Context, a: &Term, b: &Term) -> bool;
fn level_eq(a: &Level, b: &Level) -> bool;            // 17 §3.6 convLevel
fn whnf(env: &GlobalEnv, ctx: &Context, t: &Term) -> Term;       // weak-head NF
fn normalize(env: &GlobalEnv, ctx: &Context, t: &Term) -> Term;  // full NF (NbE)

// ── Elaborator precondition — raw syntactic well-formedness (no env, no types).
fn raw_well_formed(ctx: &Context, t: &Term) -> KernelResult<()>;

// ── Trusted-base enumeration — a method on the env (§5).
impl GlobalEnv { fn trusted_base(&self) -> Vec<GlobalId>; }
```

The **input** to `declare_inductive` is the lightweight `InductiveSpec`
(`level_params, params, indices, level, constructors: Vec<CtorSpec>`); the
**stored** form callers read back via `env.inductive(id)` is `InductiveDecl`
(allocated ids, the built former type, per-constructor `recursive_positions`).
The `build` closure receives the family's fresh `GlobalId` so constructor
signatures can self-reference `D` (e.g. `suc : Nat → Nat`). `PrimReduction` is
`OpaqueType | Op { symbol }`. `KernelResult<T> = Result<T, KernelError>` (§4.4).

### 4.2 Per-entry contract

Every entry is **stable** (frozen — V0/X1/V-series build against it) unless
marked internal. "Pre" is what the caller must guarantee; "Guarantee" is what an
`Ok`/`true` result means; "Errors" are the `KernelError` variants (§4.4) the
entry may return.

| Entry | Pre | Guarantee on success | Errors |
|---|---|---|---|
| `declare_def` | `ty`, `body` raw-well-formed over `·` | `· ⊢ ty type`; `· ⊢ body ⇐ ty`; the def's group passes **SCT** (`17 §4`); `id` admitted **transparent** (δ-unfoldable). Pre-admitted opaque during checking so `body` may self-reference `id` | `TypeMismatch`, `UniverseInconsistency`, `NotTerminating` (SCT reject ⇒ `id` removed, Σ unchanged) |
| `declare_recursive_group` | one `(level_params, ty)` per member; `bodies_fn` returns one body per member, in order | each `ty` checked; all members pre-admitted opaque; each body checked; **SCT on the whole group**; accept ⇒ all transparent; **reject ⇒ the whole group rolled back** | as `declare_def`; `NotTerminating` rolls back every member |
| `declare_inductive` | `build(id)` yields a well-formed `InductiveSpec` self-referencing `id` | signatures checked; **strict positivity** (`14 §8`) and the **W-style admission boundary** (`14 §8.4`: W-style admitted, negative / non-`D`-free-domain rejected) hold; type former + constructors admitted; the **dependent eliminator** (Π-abstracted IH + W-ι, `14 §3.1`/`§7.7`) generated on use | `PositivityViolation`, `IllFormedDecl`, `LevelArityMismatch` |
| `declare_postulate` | `name` is a non-positional audit label; `ty` raw-well-formed over `·` | `· ⊢ ty type`; `id` admitted **opaque** with `name`; **recorded in the trusted base** (appears as a named entry in `trusted_base()`). A postulate of an empty type is admitted but **visible** as an assumption | `TypeMismatch`, `UniverseInconsistency` |
| `declare_primitive` | `ty` raw-well-formed; `reduction` the registered operation descriptor | `· ⊢ ty type`; `id` admitted opaque + descriptor **registered in the trusted-base ledger**. `Literal` records a value class; `Op` is opaque to landed conversion and names interpreter dispatch (§5) | `TypeMismatch`, `UniverseInconsistency` |
| `infer` | `ctx` well-formed; `t` raw-well-formed | returns the **unique** `A` with `ctx ⊢ t ⇒ A` (§3.1) | `VarOutOfScope`, `NotAFunction`, `NotASigma`, `LevelArityMismatch`, `TypeMismatch`; a non-inferable head ⇒ error |
| `check` | `ctx` well-formed; `t` raw-well-formed; **`ty` a well-formed type** | `ctx ⊢ t ⇐ ty` (§3.2); the single conversion call is the mode switch | `TypeMismatch` (the two non-converting types), plus any from `infer` |
| `convert` | `a`, `b` both check at `ty` | `true` ⇔ `ctx ⊢ a ≡ b : ty` (`17`); **total + decidable**. Threads `ty` for η + the Ω-PI shortcut (`16 §8.2`) | none — returns `bool`; the caller manufactures the error |
| `convert_type` | `a`, `b` are types | `true` ⇔ `ctx ⊢ a ≡ b type` (`17 §3.3`; types take no η). **This is the entry the (Conv) mode switch calls** | none — `bool` |
| `level_eq` | — | `true` ⇔ `a ≡ b` as levels (`12 §1`/`§6.1` semilattice normal form; `17 §3.6`). Total | none — `bool` |
| `whnf` | `t` raw-well-formed | **weak-head normal form**: head is not a redex. **Infallible** — an ι arity mismatch leaves the eliminator stuck/neutral, which is sound (`14 §7.6`). δ enabled at the head | none — total |
| `normalize` | `t` well-typed | **full normal form** (NbE): whnf then normalize under binders. Total on well-typed terms | none — total |
| `raw_well_formed` | — | `Ok` ⇒ `t`'s de Bruijn indices are in scope under `ctx` and it uses no reserved former — the **elaborator precondition** before `infer`/`check` | `VarOutOfScope`, `IllFormedDecl` |
| `GlobalEnv::trusted_base` | — | enumerates **exactly** the postulates + primitives (as `GlobalId`s), excluding the prelude (`Top`/`Bottom`/`tt`) and excluding definitions/inductives (re-checked, not trusted); every postulate id resolves to its audit label — §5 | none — total |

The required `String` is an audit label stored on the opaque declaration, not a
term and not part of definitional identity. `Decl::Opaque` has a required,
non-optional `name: String` field, and every `declare_postulate` caller
MUST supply the corresponding required argument; there is no default, optional,
or builder path for an unnamed postulate. This type-level requirement forces
surface, elaborator, and Rust-side producers through the same naming choke
point.

A surface `Axiom` in a top-level declaration uses that declaration's canonical
name. An `Axiom` in an instance field uses exactly
`Class.HeadType.field`. The caller of a public standalone-expression API
supplies a stable semantic audit owner through a required, non-optional
argument; an ownerless call is unrepresentable. Another non-surface elaborator
or Rust producer supplies its stable declared symbol at the same entry point.
No producer may derive a label from an expression, source position, module
fallback, `GlobalId`, allocation order, session counter, generated counter,
gensym, occurrence index, or generic sentinel. Multiple `Axiom` occurrences in
one semantic owner therefore legitimately share the same owner label while
retaining distinct `GlobalId`s; both entries remain visible in
`trusted_base()`. Labels identify semantic
owners, not individual occurrences. Inserting an unrelated declaration or field
cannot rename an existing postulate. The current corpus has at most one
canonical structure instance per class/head pair by coherence
(`../30-surface/33 §5.5`), but that observation is not a unique-label
invariant.

The label is deliberately kernel-inert (`11 §4`). Apart from storing it and
making it available to audit enumeration, the kernel MUST NOT read it:
conversion, typing, admission, positivity, universe checking, and elimination
may not branch on a postulate label. In particular, two declarations with
different labels are not distinguished *because* of those labels; their
ordinary `GlobalId`s already provide declaration identity.

**Internal (reachable but not the frozen contract):** the gate functions
`check_positivity`, `sct_check`, and the quotient respect/relation checks (these
fire *through* `declare_inductive`/`declare_def`/`infer`, §4.3); the
K2-reduction helpers in `obs`; and `GlobalEnv` bookkeeping (`add_decl`,
`fresh_id`, `remove_last`, `upgrade_to_transparent`, `const_type`,
`transparent_body`, `inductive`, `constructor`, `lookup`, `decls`). External
components do **not** call these; they are kernel-internal and may change
without a contract break.

### 4.3 The admission gates (cited, not restated)

Admission *semantics* are normatively owned by the per-feature chapters; §18 is
the API *surface* and **cites** them, so the contract can never disagree
with the chapter it points at. Each gate runs on **every** input — nothing is
admitted without passing.

| Gate | Fires at | Accept ⇔ | Cite | Reject error |
|---|---|---|---|---|
| Strict positivity | `declare_inductive` | every recursive occurrence of `D` is strictly positive (`Pol::Plus`) | `14 §8` | `PositivityViolation` |
| W-style admission (K1.5) | `declare_inductive` | a Π-bound recursive arg `(b:B) → D Δ_p t̄` has `D` only as **target**, `B` `D`-free; the **eliminator** generates the Π-abstracted IH + W-ι | `14 §2.1`/`§8.4` (gate) + `§3.1`/`§7.7` (elim/ι) | `PositivityViolation` |
| SCT (δ-termination) | `declare_def`, `declare_recursive_group` | every idempotent self-loop has ≥1 strict descent (`↓`) on the diagonal | `17 §4` | `NotTerminating` |
| Quotient respect (K2c-s2) | `infer`/`check` on `QuotElim` | Ω-target motive: respect-free; **Type-target motive: the respect proof checks against the `cong`/`cast` respect schema** | `16 §5.1` | `BadEliminator` |

Positivity and W-style fire **inside** `declare_inductive`; SCT inside the
definition declarators; quotient respect is checked when a `QuotElim` term is
**typed** (so it is gated at admission of the elimination, not a separate
declarator). This is why an "invoking" conformance test per gate (Acceptance §3)
drives the gate through its host entry, flipping accept↔reject on the gate
condition alone.

### 4.4 `KernelError` is a typed enum (honest minimality)

The error is a **typed enum**, not a single string blob — so the reason is
**variant-specific**, not one uniform "subterm + two types" payload:

- `TypeMismatch { expected, found }` — the two non-converting types (the (Conv)
  failure; the mode switch manufactures it from a `false` `convert_type`).
- `NotAFunction { head }`, `NotASigma { head }` — the failing **subterm** (the
  mis-eliminated head).
- `VarOutOfScope { index, depth }`, `UniverseInconsistency { expected, found }`,
  `LevelArityMismatch { expected, found }` — precise, structured.
- `PositivityViolation(..)`, `NotTerminating(..)`, `BadEliminator(..)`,
  `IllFormedDecl(..)` — the admission-gate rejections.

This is **minimal and precise** (the failing subterm and/or the two types), but
note `convert`/`convert_type` themselves return bare `bool` — there is no error
object at the conversion boundary; the *caller* (the `check` mode switch) builds
`TypeMismatch`. The kernel never returns a *partial* result or an `unknown`:
those are verification-layer concepts (`../20-verification/24-diagnostics.md`).
It answers typed/not-typed, equal/not-equal. Turning a `KernelError` into a
human/agent diagnostic is V4's job, not the kernel's.

### 4.5 Proof checking is `check` — there is no `check_proof`

The prover (`../20-verification/23-prover.md`) returns a certificate **term**;
checking it **is** typing — `check(env, ctx, proof, goal)`, where `goal` is the
proposition (`12 §5`). There is no separate `check_proof` entry: the de Bruijn
criterion is literally one call to `check`. A wrong certificate fails
`check`; it cannot make a false proposition inhabited — the soundness firewall
around the untrusted prover/Z3 (§5).

### 4.6 Freeze gating (the build-sequencing dependency)

§18 is **authored** now against the complete-kernel gates (§4.3, correct by
citation). The **freeze** — "§4 is the audited TCB boundary the code exposes" —
is **contingent** and is a **hard gate on the K-api merge Decision**, not prose:
the API-stabilization merge must land **after both**

1. **K1.5-build** — W-style admission *and* the `method_type` indexed-family
   eliminator fix (`dec_2vc6ytrbcbfc5`); until it lands the kernel both rejects
   W-style inductives (the pre-K1.5 blanket Π-bound gate is still active)
   **and** would mis-type W-style eliminators for indexed families, so
   "W-style is in the TCB boundary, soundly" is not yet true; and
2. **K2c-series-2-build** — non-Ω quotient respect (`16 §5.1`); until it lands
   the kernel rejects non-Ω `QuotElim`.

The K-api merge Decision is gated on these two builds being
**green-and-merged**, so the surface the contract claims is the surface the code
actually exposes the day K-api lands (cross-ref the banner's freeze-status
note).

## 5. The trusted base (what soundness actually rests on)

Ken's audited boundary has three categories. Kernel proof soundness rests on
the declarations/signatures in all three; runtime value correctness additionally
relies on the interpreter semantics named by (2):

1. **The kernel code** implementing §1–§4 and `11`–`17` (the Rust core) — this
   includes the admission gates (§4.3: positivity, W-style, SCT, quotient
   respect), which are *trusted-as-code* but **re-run on every input**: nothing
   is admitted without passing, so the gates add no per-program assumption.
2. **The primitive declarations and operation registrations** admitted via
   `declare_primitive` (`14 §5`) — opaque constants enumerated for audit. Their
   declared types are part of the proof-soundness TCB. In the landed system an
   `Op` symbol dispatches the interpreter's `prim_reduce`; that implementation
   is not executed by kernel conversion. Its specified partial function must be
   correct for runtime semantic correctness, but a wrong result is a wrong
   runtime value rather than an inhabitant of a false proposition.
3. **Any postulates** admitted via `declare_postulate` — each is an *assumed*
   axiom carrying the stable audit label required by §4.2; a postulate of an
   empty type would make the system inconsistent.

Nothing else is trusted for proof soundness: not the elaborator, the prover,
Z3, the surface compiler, or the runtime. The kernel MUST be able to
**enumerate (2) and (3)**
on request via the method `GlobalEnv::trusted_base() -> Vec<GlobalId>` (§4.1),
which returns **exactly** the registered primitives + admitted postulates —
excluding the prelude (`Top`/`Bottom`/`tt`) and excluding
definitions/inductives, which are re-checked rather than trusted. Every returned
postulate id resolves to the label stored by `declare_postulate`, so a reviewer
or agent sees names rather than an allocation-order-only list. This includes
postulates minted internally by the elaborator or its Rust support, not only
surface `Axiom` expressions. Idiomatic Ken adds **no** postulates; classical
axioms, if used, appear here and are visible (`16 §1.3` — Ω is
intuitionistic, excluded middle is not assumed).

The enumeration records the trusted primitive declarations/signatures while
current `Op` execution remains in the tested-not-trusted interpreter ring. It
does not imply that the kernel executes the registered operation. Promoting
those operations into conversion would enlarge the kernel TCB and is the
separate K3 decision.

The concrete enumeration of clause (2) — every native (`declare_primitive`)
operation, **adversarially** re-adjudicated against the surface chapters
(`35`/`37`/`38`), each with its verdict (`NATIVE` / `DEMOTE→derived` /
`RETIRE`), its correctness-AC, the class laws it forecloses to
**postulate-only**, and its
differential-oracle net — is the **primitive-operation registry**
(`18a-primitive-registry.md`, BUILTINS Phase 1). It is the auditable surface
behind clause (2)'s runtime semantic contract: `trusted_base()` enumerates
exactly the ops it ratifies `NATIVE`, plus the admitted postulates. Where a
native op meets an input outside its total domain the registry pins an
**obligation-emitting / error-surfacing** interpreter result — never a silent
wrong value (the partiality discipline, `18a §2`). This oracle checks runtime
values; it is not evidence of kernel conversion.

## 6. Metatheory status (honest accounting)

The kernel's soundness commitments (`README.md §5`) and their current status:

| Commitment | Status |
|---|---|
| No `Type:Type` / universe consistency | **By construction** (`12`); tested. |
| Subject reduction | **Argued** from the rules; extended to the K2c-s2 reducts (cast-at-inductive-index, J-at-dependent-motive, Type-target quotient elim — `16 §8.4`) and the W-style ι (`14 §9.4`); to be mechanized. |
| Confluence / unique normal forms | **Argued** (standard for this calculus). |
| Strong normalization of the core | **Argued** (β/ι/η/obs). W-style ι terminates by **finiteness of the inductive tree** (`14 §9.4`), not by stuckness; the hard metatheorem overall. |
| δ-termination → decidable checking | **By the SCT gate** (`17 §4`); the size-change principle (Lee/Jones/Ben-Amram 2001) bounds δ-unfolding (`17 §5`); tested. |
| Canonicity (closed terms compute) | **Required + tested** (`16 §9`, observational). The observational reductions are now **complete** (K2c series-1+2): the three formerly sound-but-stuck seams all compute (`16 §3.2`, `§4.1`, `§5.1`). |
| Decidable conversion | **Proven** for OTT (`TTobs`/`CICobs`, ADR 0005); Ken follows. **K2c delivers the operational decidability**: `convert` is total via the SCT gate (`17 §4`–§5), so the (Conv) mode switch (§3) always halts. |
| Consistency (no closed `· ⊢ p : ⊥`) | **Argued** from SN + canonicity. |

"Argued" means there is a standard proof for systems of this shape
(observational TT — `TTobs`/`CICobs` — + inductives + a terminating δ;
canonicity and decidable conversion are *proven* for OTT, ADR 0005) and Ken
intends to *follow* it, not that Ken has a machine-checked proof yet. A
mechanized kernel-soundness proof is a later goal (strategy G5 documents the
story; full mechanization is post-self-host, `../../docs/program/02-roadmap.md`
Phase 5). This table is the kernel's "known-risk register"; the conformance
corpus exercises each commitment behaviorally even where the metatheorem is not
yet mechanized.

**Level discipline (this chapter).** §18 introduces **no** level-computing rule
of its own: the formation universes are computed by the cited rules (`12`/`13`
predicative `max`, `15`/`16 §2` `Eq : Ω_ℓ`), and the only rule local to §18 —
(Conv) and its algorithmic mode switch — operates **at** a universe without
arithmetic on levels: it asks `convert_type` (and, for the universe heads,
`level_eq` = `convLevel`, `17 §3.6`) whether two types are definitionally equal.
Non-cumulativity (`12 §3`, OQ-2) means (Conv) compares levels by **equality, not
subsumption** — `Type 0 ⇐ Type 2` is rejected (no implicit lift). Universe
consistency is by construction (`12`); there is nothing here that the level
calculus of `12` does not already settle.

## 7. What the kernel checks here

A conforming kernel MUST implement the four judgments (§1) with the collected
rules (§2) including (Conv); the bidirectional infer/check algorithm (§3) with
the single conversion call at the mode switch and **no** unification/guessing
(§3.3); the API (§4) as a **total, decidable** surface matching the §4.2
per-entry contract, whose every declarator re-checks its input and passes the
§4.3 admission gates (positivity, W-style, SCT, quotient respect); a typed,
minimal `KernelError` (§4.4); proof checking as `check` with no separate
`check_proof` (§4.5); and the `trusted_base()` enumeration (§5). It MUST NOT
contain implicit insertion, error recovery, or proof search — those belong to
untrusted layers. Conformance: `../../conformance/kernel/judgments/` —
infer/check round-trips, the (Conv) switch (a term that checks only via a
conversion step), one **invoking** test per admission gate flipping
accept↔reject on the gate condition, a certificate-checking case (prover output
re-checked) and its wrong-certificate rejection, an ill-typed term rejected with
a minimal error, and a `trusted_base()` enumeration test. Per §4.6, the
W-style/positivity and non-Ω-quotient invoking cases pass against the **complete
kernel** (they ride K1.5-build and K2c-series-2-build).
