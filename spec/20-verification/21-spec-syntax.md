# Specification syntax

> Status: **V1 elaborated** (implementation-ready). Normative for the forms,
> their meaning, their grammar/AST, their **elaboration to core**, and the
> **verification status model**; concrete surface spelling cross-refs
> `../30-surface/`. Contract for WS-V **V1** (the first WP of the verification
> spine V1→V2→V3). **★★ (untrusted):** everything this layer emits is re-checked
> by the kernel (`../10-kernel/18 §4`); a bug here is a wrong verdict or a poor
> diagnostic, **never** unsoundness. How a programmer or agent attaches a
> *correctness specification* to code — the surface the whole verification loop
> hangs off. This is the L1→L2 bridge at the surface level.

A **specification** is one or more **propositions** (`../10-kernel/12 §5`,
elements of Ω) attached to a definition, asserting how it must behave. Ken
offers three layered ways to state one, from lightest to most expressive (§1–3),
discharged through one smooth **gradient** — declarative contract → automatic
proof → typed hole → tactic/term (`23`), never a cliff between "automatic" and
"hand-written."

**The defining discipline (`OQ-spec`, DECIDED).** Because Ken is a *software
engineering* language — written by agents, read by humans — every claim carries
a **visible, exportable epistemic status**: **proved** (kernel-discharged),
**tested** (assumed, with a runtime/test obligation), **delegated** (a
temporal/behavioral property Ken can *state* but not close as a static
proposition — exported downstream), or **unknown** (an open typed hole). The
source distinguishes proof from test from model from gap *on its face* (§5); the
delegated/tested set is emitted as the **assumption boundary** consumed by the
behavioral sibling (`../70-behavioral/`, ADR 0006). This is the surface form of
"prove what can be proven and state what must be tested."

**What V1 elaborates (this chapter's scope).** The three spec forms (§1–§3) with
**concrete grammar and AST** (§6.1–§6.2) extending the V0 surface
(`../30-surface/39-elaboration.md §5`); their **elaboration to core** as
defensive pseudocode (§6.3–§6.5); the `old`-capture rule (§6.4); the
**verification status model** — the per-obligation *verdict* and the per-claim
*epistemic status*, with each verdict's carried evidence and the honesty guard
(§5); and the **V1→V2 interface** the obligation generator consumes (§7). The
concrete clause spelling for the **disposition tags** `tested`/`assume`/`test`
stays **reserved** (`OQ-syntax`, deferred — §5.5): V1 fixes what those statuses
*mean* and how they project to the assumption boundary, not their surface
grammar. The *prover* (`23`), *obligation extraction* (`22`), and *diagnostics*
(`24`) are downstream WPs.

## 1. Function contracts — `requires` / `ensures`

The everyday form: pre- and post-conditions on a function.

```
view divide (n : Int) (d : Int) : Int
  requires  d ≠ 0
  ensures   result * d + (n % d) == n
= n / d
```

- **`requires φ`** — a **precondition**: a proposition `φ : Ω` over the
  function's parameters. Callers must establish it; inside the body it may be
  assumed.
- **`ensures ψ`** — a **postcondition**: a proposition `ψ : Ω` over the
  parameters *and* the special binder **`result`** (the return value). The body
  must establish it.
- Multiple `requires`/`ensures` clauses conjoin. Clauses may mention earlier
  parameters (they are checked in the telescope `../10-kernel/13 §3`).
- Contracts are **erasable**: they generate obligations (`22`) and assumptions
  but no runtime code by default (runtime-checked contracts are an opt-in, §5).

The clause grammar (the `view`-declaration addendum; the full grammar is §6.1):

```
contract ::= requires-clause* ensures-clause*
requires-clause ::= "requires" prop        -- a precondition proposition (: Ω)
ensures-clause  ::= "ensures"  prop         -- a postcondition (: Ω, may use `result`)
prop ::= expr                               -- any expression that checks at Ω (§4)
```

Semantically, a contract **denotes** a **refined function type**
(`../30-surface/39-elaboration.md`): `divide` above denotes the dependent type

```
(n : Int) (d : Int) → { d ≠ 0 } → (r : Int) × (r * d + (n % d) == n)
```

i.e. preconditions are extra (proof) arguments and the postcondition pairs the
result with a proof. This is the **denotational** reading — the meaning of the
contract as a type. The **operational** V1 elaboration does **not** reify the
postcondition as a proof-carrying `Σ` *value*: the result stays at the carrier
`Int` and the postcondition becomes a separate **obligation** (a typed hole
discharged by a kernel-checked certificate, §6.3). The reasons — erasability and
a kernel `Σ`-sort caveat — are in §2 and §6.3. The programmer writes
`requires`/`ensures`; the encoding is hidden.

## 2. Refinement types — `{ x : A | φ x }`

A **refinement type** is the comprehension subobject (`../10-kernel/12 §5`,
`../30-surface/34-data-match.md`): the type of `x : A` *for which `φ x` holds*.

```
def Pos = { n : Int | n > 0 }
view head (xs : { l : List A | l ≠ nil }) : A = …      -- non-empty by type
```

- `{ x : A | φ x }` requires `φ x : Ω`. Its inhabitants are, by the
  comprehension reading, pairs `(x, proof-of-φx)` with the proof component a
  **mere proposition** (`16 §1.2`), so refinements carry *no runtime payload*
  and coerce silently to `A`.
- Refinements are the **route to L2 at the surface**: pushing a property into a
  type makes the checker enforce it at every use, with obligations generated
  where a plain `A` is used as `{x:A|φ}` (`22`).
- They compose with Π/Σ: function arguments, results, and record fields may all
  be refined; `requires`/`ensures` (§1) is the contract spelling of refined
  argument/result types.

**The V1 core encoding (operational) — carrier-plus-obligation, normative.** A
refinement `{x:A|φ}` elaborates **to its carrier `A`**; the predicate `φ` is
tracked by the (untrusted) elaborator, not reified as a core type, and every
*introduction* of a value at the refinement emits the obligation `φ a` (§6.3,
`22 §2.1`). The naive reification `{x:A|φ} = Σ(A, φ)` with `φ : Ω` is **not**
used in V1, for two reasons:

1. **Erasability.** The proof component is in Ω (proof-irrelevant,
   computationally irrelevant), so the value behaves as a bare `A` at runtime;
   keeping the carrier *as* `A` and the proof *as* an obligation realizes this
   directly (`24 §2`).
2. **A `Σ`-sort caveat (grounding catch, Architect-confirmed).** The kernel as
   landed classifies `Σ(A : Type ℓ, φ : Ω_ℓ')` as a **proposition** in Ω (the
   formation sort was keyed on the codomain only, `check.rs sort_pi_sigma`,
   sound for Π-into-prop but over-admitting a Σ with a *relevant* first
   component). A refinement reified that way is collapsed by Ω-proof-irrelevance
   (`16 §1.2`): `(3, p) ≡ (5, q) : {n:Int|n>0}` definitionally — the carrier is
   lost, and via a transport motive it closes to a proof of `Empty`. The
   Architect **confirmed** this as a reachable trust-root over-equating hole; a
   priority kernel erratum splits the rule (`sort_sigma → Ω` iff *both*
   components are Ω; Π stays codomain-keyed), with the matching spec erratum in
   `../10-kernel/13 §4`/`§5`. The carrier-plus-obligation encoding here is
   **independent** of that fix (it never forms a core `Σ` over an Ω predicate).
   Even once the kernel admits a relevant subset-`Σ` at `Type (max ℓ_A ℓ_φ)`,
   V1 keeps the obligation encoding as the **erasure-faithful** operational
   form; the proof-carrying `Σ` reification remains the *denotational* reading.

The kernel has **no subtyping** for refinements: `{x:A|φ} ≤ A` is free (here the
identity, since the core type *is* `A`), and `A ≤ {x:A|φ}` generates the
obligation `φ` — checked by emitting that obligation, never by a kernel coercion
(§6.3).

## 3. Propositional goals — `prove` / `law`

A **goal** is a standalone proposition to be discharged — a lemma, an invariant,
or an algebraic law — not attached to a single function's body.

```
prove  add_comm : (a b : Int) → a + b == b + a
law    Monoid (M) { assoc : … ; unit_l : … ; unit_r : … }   -- a property bundle
```

- **`prove name : φ`** registers `φ : Ω` as an obligation; on success `name`
  becomes a usable proof term of `φ` in the environment (`../10-kernel/11 §4`).
  Until discharged, `name` is an open obligation — a typed hole / visible
  postulate (status `unknown`, §5).
- **`law`** bundles related propositions (the algebraic-law form, stated via
  `law`/`verify`); proving a `law` for a type makes the bundle available as a
  record of proofs, usable by constraint resolution (typeclasses-as-subobjects,
  `../30-surface/33-declarations.md`). A `law` whose fields are all propositions
  is a conjunction of props — the sound `Σ`-of-Ω-into-Ω case (`16 §1.3`), so the
  bundle is itself a proposition.
- **Named proof claims** (`prop` / `lemma` / attached `proof`) are the same
  proof lane with different namespacing. `prop` declares a proposition family,
  `lemma` declares a standalone theorem in the ordinary module namespace, and
  `proof` attaches a theorem to a resolved subject path. All three are Ω-typed
  proof claims; none adds a new kernel declaration class or a trusted proof
  table.
- Goals are where Ken is used as a *proof assistant*, and where the REPL's
  "Little Prover" loop (`../30-surface/`, strategy T2) lives.

The clause grammar (the declaration addendum; full grammar §6.1):

```
goal-decl ::= "prove" ident ":" prop                  -- a named goal proposition
            | "law"   ConId "(" ident ")" "{" law-field (";" law-field)* "}"
law-field ::= ident ":" prop                          -- a named bundled proposition
prop-decl ::= "prop" ConId tyvar* binder* ":" type prop_block?
lemma-decl ::= "lemma" ident binder* ":" type "=" expr
proof-decl ::= "proof" ident "for" path binder* ":" type "=" expr
```

## 4. What the binders mean (precise)

| Binder / form | Scope | Type |
|---|---|---|
| parameters `x : A` | the whole contract + body | as declared |
| `result` | `ensures` clauses only | the function's return type |
| `old(e)` *(scoped to `space` ops)* | `ensures` of a `space` operation | value of `e` in the pre-state |
| `φ` in `requires`/`ensures`/`{·|φ}`/`prove` | as above | **must be `: Ω`** |

- Every specification proposition MUST type-check at `Ω` (`12 §5`, `16 §1.1`) in
  its scope; a `requires`/`ensures` whose body is not a proposition is a
  **surface type error** (caught at elaboration, §6.3), not a verification
  failure. This is a load-bearing guard: the elaborator `check`s each clause
  body at Ω and rejects a non-Ω body *before* any obligation is formed.
- A `prop` family result, `lemma` theorem, or attached `proof` theorem also
  MUST type-check at `Ω` in its scope. The attached form is still just a proof
  term; its canonical export name is `subject::proof_name`, and there is no
  separate proof-table lane.
- **`result`** is in scope only in `ensures` clauses; referencing it in a
  `requires` or a refinement predicate is a scope error.
- **`old(e)`** (referring to a pre-state value in a postcondition) is meaningful
  only for **`space` operations** (`../30-surface/36-effects.md §4.3`); for pure
  `view`s the pre/post states coincide. **`OQ-Space` DECIDED:** `old(e)` is
  admitted, **scoped to a `space` operation's `ensures`** (a cell's pre-call
  value), well-defined because a space's denotation is a state-transformer
  `S → R × S` that *names* the pre-state (§6.4). There is **no global
  `\old`/heap** and **no separation logic** — a space's cells are non-aliased,
  so reasoning is bounded per-space Hoare. An `old` outside a `space`-op
  `ensures` is a scope error (§6.4). For explicitly-threaded state you simply
  name the pre/post values.

## 5. The verification status model

A spec'd program returns, per claim, an **actionable status** — the feature that
makes Ken "Ken." Two distinct classifications are in play, and conflating them
is the chapter's central hazard, so they are separated here:

- the **verdict** (§5.1) — the *operational* outcome of attempting one
  obligation, with its **carried evidence**. Per-obligation, produced by `22`/
  `23` and rendered by `24`/`25`. The kernel/Heyting **trichotomy**:
  `proved` / `disproved` / `unknown`.
- the **epistemic status** (§5.2) — the *export-facing* label a claim carries
  (`OQ-spec` DECIDED). Per-claim, visible in the source and in the assumption
  boundary. The **four-way**: `proved` / `tested` / `delegated` / `unknown`.

§5.3 is the projection between them; §5.4 is the honesty guard (an `unknown`
must never read as `proved`); §5.5 is the V1 scope ruling on disposition-tag
syntax.

### 5.1 The verdict (per-obligation, operational)

Attempting an obligation `Γ ⊢ φ` (`22 §1`) yields one of **three** verdicts —
the surface rendering of the kernel trichotomy (`16 §1`, `24 §3`), the spine
of the protocol's verdict (`25`) — each carrying the evidence that makes it
actionable and (for `proved`) re-checkable:

| Verdict | Meaning | Carried evidence | Kernel re-check |
|---|---|---|---|
| `proved` | the obligation is discharged | a **certificate**: a core proof term `p` with `Γ ⊢ p : φ` | `check(env, Γ, p, φ)` accepts (`18 §4.5`) — the de Bruijn criterion |
| `disproved` | the proposition is refuted | a **countermodel**: a finite Kripke model forcing `¬φ` at some world (`24 §1`) | where the prover yields a proof of `¬φ`, `check(env, Γ, p, ¬φ)` certifies it; else the countermodel is a prover-asserted refutation (untrusted, but a concrete falsifying witness) |
| `unknown` | undecided / not discharged | a **typed hole** `?h : φ` in `Γ`, admitted as a **postulate** of `φ` (`24 §2`) | none — the hole is *assumed*; it appears in `trusted_base()` (§5.4) |

- A **`proved`** verdict adds **nothing** to the trusted base: the certificate
  is a closed core term `check` validates, and a wrong certificate fails `check`
  — it cannot make a false proposition inhabited (`18 §5`). This is the
  soundness firewall around the untrusted prover.
- A **`disproved`** verdict is a hard **verification error** (`24 §3`, the
  `S_{¬φ}` region: *fix the code or the spec*). It is **never** an exported
  guarantee — you do not ship a known-false claim — so it has *no* epistemic
  status (§5.3); it surfaces as a diagnostic (`24`).
- An **`unknown`** verdict leaves the program **running**: the hole is a visible
  postulate and evaluation propagates the runtime third value `unknown`
  (`24 §2`, `../40-runtime/42-evaluation.md`). Verification is *incremental*.

### 5.2 The epistemic status (per-claim, export-facing — `OQ-spec` DECIDED)

Every specification claim carries one of four **statuses**, each visible in the
source and (for the latter three) carried in the **assumption-boundary export**
(`../70-behavioral/`). This four-way distinction is the heart of `OQ-spec` and
the feature that makes Ken a *software engineering* language rather than a
programming language: a reader sees, per claim, whether it is *proved*, merely
*tested*, *delegated* to behavioral checking, or still *open*.

- **`proved`** — the obligation (`22`) was discharged and the kernel re-checked
  the certificate (`23`, `../10-kernel/18 §4`). The default for a contract that
  goes through. No annotation; it simply holds.
- **`tested`** — a property that **cannot (yet) be proven** but is **asserted
  with a runtime/test obligation**: an `assume`/`test`-tagged clause (the
  keywords are reserved, `../30-surface/31 §4`; the exact clause grammar is
  `OQ-syntax`, §5.5) that lowers `requires`/`ensures` to a runtime assertion
  (boundaries, FFI, untrusted input) *and* registers a test/generator
  obligation. It is **visible** — a reader knows this guarantee rests on tests,
  not proof — and it is **exported** as part of the assumption boundary (the
  refinement predicate becomes a generator/oracle spec, `../70-behavioral/`,
  §2/Layer 2).
- **`delegated`** — a **temporal/behavioral** property (liveness, fairness,
  ordering, eventual consistency, an interleaving safety property) that is **not
  a static proposition over a pure function** and so cannot be closed in the
  kernel. Ken can *state* it — as ordinary **deeply-embedded temporal-logic
  data** (an inductive `Temporal`/μ-calculus value, *not* a kernel modality, so
  the TCB is untouched, `../70-behavioral/`) — and **exports** it to the
  behavioral sibling for model-checking and runtime monitoring. Stated here,
  discharged there. *(This is "the fourth" status the verification spine adds
  beyond the proof trichotomy: it carries a model-checking/monitoring
  obligation, not a kernel proof obligation.)*
- **`unknown`** — the obligation is *not* discharged and no test/delegation is
  given: the definition is admitted with a **typed hole** and the program
  **still runs**, the result carrying `unknown` where the unproven property is
  observed (`24-diagnostics.md §2`). Verification is *incremental*, not
  all-or-nothing (Hazel-style); a hole is the honest "not done yet."

By default `proved` specs are static-only (erased); `tested` adds runtime code
by construction; `delegated` adds none to Ken (it is exported); `unknown` adds
none. The **assumption boundary** (`../70-behavioral/`) is precisely the
`tested` + `delegated` + open-`assume` set — what Ken could not guarantee
statically, handed to the sibling as the exact specification of what to model,
test, and monitor.

### 5.3 How the verdict and the status relate (the projection)

The epistemic status of a claim is the claim's **disposition** (how the author
chose to establish it — prove it, test it, or delegate it) resolved with the
**verdict** of attempting it:

| Disposition | Verdict (§5.1) | Epistemic status (§5.2) | Carried evidence |
|---|---|---|---|
| prove (default) | `proved` | **`proved`** | certificate (kernel-checked) |
| prove (default) | `unknown` | **`unknown`** | typed hole = postulate |
| prove (default) | `disproved` | *(none — a verification error, `24`)* | countermodel |
| `test`/`assume` | — *(not statically attempted)* | **`tested`** | runtime/test + generator obligation |
| `delegate` (temporal) | — *(not a static proposition)* | **`delegated`** | temporal-logic export to `../70-behavioral/` |

So the frame's "four-way status" is the **epistemic status** (four labels,
DECIDED); the operational **verdict** is the trichotomy (three outcomes). A
`disproved` verdict has no exported status because a refuted claim is fixed, not
shipped; the `tested`/`delegated` statuses sit beside the proof axis entirely,
carrying downstream (test/monitor) obligations rather than a kernel verdict.

### 5.4 The honesty guard (`unknown`/`tested`/`delegated` never read `proved`)

The load-bearing property of the whole model (§the framing in
`docs/wp/V1-spec-syntax.md`): a partially-verified claim must **never**
masquerade as a proved one. The discriminator is **kernel-side, not a
V-layer flag** — so a bug in the (untrusted) verification layer cannot forge a
`proved`:

- A `proved` claim's certificate is a closed core term that `check` accepts; it
  introduces **no postulate** of the goal, so the goal does **not** appear in
  `GlobalEnv::trusted_base()` (`18 §4`, `§5`: `trusted_base` enumerates exactly
  the postulates and primitives).
- An `unknown` claim's typed hole **is** a postulate of the goal (`24 §2`), so
  the goal **does** appear in `trusted_base()`. Likewise a `tested`/`delegated`
  claim is admitted as a visible assumption (its obligation is downstream, not
  kernel-discharged).

Therefore the verdict is decidable from the kernel's own state, not from a label
the layer self-reports: **a claim is `proved` iff its certificate `check`s *and*
no postulate carrying its goal sits in `trusted_base()`.** The *presence* of
such a postulate is the guard that the obligation is *assumed, not proved* —
guard-gated (postulate membership), not coincidental. "Shipping a verified
artifact" means **zero spec-induced postulates** in `trusted_base()` (or an
explicit, recorded acceptance of the listed ones). This is the absence-assertion
the conformance corpus must pin: an `unknown` case is distinguished from a
`proved` case by `trusted_base()` membership + certificate-presence — a
*structural* discriminator that flips, not a status string compared for
equality.

### 5.5 Scope ruling — disposition-tag syntax is deferred

V1 delivers the **status model** above (all four statuses' meaning, the verdict
trichotomy, the projection, the honesty guard) and **concrete grammar** for the
proof-disposition forms `requires`/`ensures`/`{x:A|φ}`/`prove`/`law` (§6.1).
The **disposition-tag clause spelling** — how `tested`/`assume`/`test` and a
`delegated` temporal clause attach to a declaration — stays **reserved**
(`assume`/`test` are reserved keywords, `31 §4`; the clause grammar is
`OQ-syntax`). It is deferred because the `tested`/`delegated` paths depend on
the behavioral sibling (`../70-behavioral/`) and the test/generator framework
(`../50-stdlib/`), which are downstream of V1. Conformance for V1 therefore
exercises the *status model* (verdict-distinct, the honesty guard) and tags any
`tested`/`assume`-spelling case **deferred** rather than asserting un-landed
grammar.

## 6. Surface syntax, AST, and elaboration to core

V1 **extends** the V0 surface (`../30-surface/39-elaboration.md §5`) — its
lexer, AST, parser, resolver, and bidirectional elaborator — with the spec
forms, and **reuses** the kernel term constructors V0 never emits (`Pi`,
`Sigma`, `Pair`, `Proj`, `Omega`, `Eq`; all already in the kernel `Term`, none
`[K2]`-reserved). V1 **introduces** the verdict/obligation/status vocabulary
(none exists in the codebase today). Like all of V0, the output is
**kernel-re-checked**: a bug yields a rejected program or an open hole, never an
unsound acceptance (`39 §1`).

### 6.1 Grammar and lexer addendum

**Lexer (`31 §4`, the V0 lexer of `31 §8`).** V1 reserves the keywords
`requires ensures prove law` (already in the §4 keyword table) and the
refinement brackets `{ } |` (the spec brace, `31 §2` punct). `assume test` stay
reserved but their clause grammar is deferred (§5.5). `old` is a **contextual**
keyword — recognized only inside a `space`-op `ensures` (§6.4) — not globally
reserved.

**Grammar (the `32` brace form, extending `39 §5.2`).** The spec forms attach to
declarations and types:

```
view-decl ::= "view" ident binder+ (":" type)? contract? "=" expr
contract  ::= requires-clause* ensures-clause*
requires-clause ::= "requires" prop
ensures-clause  ::= "ensures"  prop

type      ::= … (V0 type forms) …
            | "{" ident ":" type "|" prop "}"       -- refinement {x : A | φ}

goal-decl ::= "prove" ident ":" prop
            | "law" ConId "(" ident ")" "{" law-field (";" law-field)* "}"
law-field ::= ident ":" prop

prop-decl  ::= "prop" ConId tyvar* binder* ":" type prop_block?
lemma-decl ::= "lemma" ident binder* ":" type "=" expr
proof-decl ::= "proof" ident "for" path binder* ":" type "=" expr

prop      ::= expr   -- an ordinary expression; elaboration checks it at Ω (§6.3)
```

A `prop` is syntactically just an expression (no separate proposition grammar);
its **Ω-typing** is enforced at elaboration, not parse time (§6.3). `result` and
`old` are ordinary identifiers at parse time; resolution (§6.3/§6.4) gives them
their binder meaning in `ensures` scope. `prop`, `lemma`, and attached `proof`
all elaborate as proof claims in the same Ω-checked lane.

### 6.2 Surface AST extension

Grounded against the landed V0 AST (`crates/ken-elaborator/src/ast.rs`), V1 adds
spec-carrying fields and one type variant (and the resolver/`RType` mirrors):

```
Decl  ::= ViewDecl name (binder list) (Type option) (Expr list) (Expr list) Expr
                                       -- params, result, REQUIRES, ENSURES, body
        | LetDecl  name (Type option) Expr
        | ProveDecl name Prop          -- new: prove name : φ
        | LawDecl   name name (LawField list)   -- new: law Name (M) { … }
        | PropDecl  name (tyvar list) (binder list) Type (PropIntro list)
        | LemmaDecl name (binder list) Type Expr
        | ProofDecl name Path (binder list) Type Expr
LawField ::= name Prop
PropIntro ::= name Type

Type  ::= …                            -- V0 forms (TPi, TArr, TUniv, TCon, TVar)
        | TRefine name Type Prop span  -- new: { x : A | φ }

Prop  ::= Expr                         -- a proposition is an expression (checked at Ω)
```

The `requires`/`ensures` lists hang on `ViewDecl` (the existing
function-declaration node gains two `Expr list` fields); the refinement gains a
`Type` variant; `prove`/`law` are new top-level `Decl`s. Nothing changes in the
V0 term-only nodes — non-spec programs parse to exactly the V0 AST (acceptance
§5, no regression).

### 6.3 Elaboration to core (the algorithm)

Elaboration extends the bidirectional V0 walk (`39 §5.4`: `infer`/`check`/
`elabType` over a kernel `Context`). The spec forms lower as follows; the
pseudocode is **defensive** — every position that *must* be a proposition is
explicitly `check`ed at Ω (a non-Ω body is a surface error, never silently
admitted), and every obligation site explicitly emits a typed hole.

**Function contract** — `requires`/`ensures` on a `view`. Preconditions become
Π proof-arguments (assumed in the body, discharged at call sites); the
postcondition becomes an obligation over `result`:

```
elabView(Σ, ⟨ view f (Δ) : B requires φ̄ ensures ψ̄ = body ⟩) → (coreDef, obls):
  Γ := extendTelescope(·, Δ)                  -- params in scope (39 §5.4)
  -- preconditions → Π proof-args, assumed in the body (22 §3)
  for φᵢ in φ̄:
    φᵢ' := check(Γ, φᵢ, Ω)                     -- MUST check at Ω (12 §5); else SURFACE ERROR
    Γ   := extend(Γ, φᵢ')                       -- pᵢ : φᵢ now an assumption
  B'  := elabType(Γ, B)                         -- the carrier result type (a Type)
  b   := check(Γ, body, B')                     -- the body at the carrier
  -- postconditions → obligations over result := b (NOT paired into b)
  obls := ∅
  for ψⱼ in ψ̄:
    ψⱼ' := check(extend(Γ, result : B'), ψⱼ, Ω) -- MUST check at Ω; result : B' in scope
    goal := subst(ψⱼ', b / result)              -- ψⱼ[body/result]  (22 §2.2)
    hⱼ  := freshHole()
    emit ⟨hⱼ, Γ ⊢ goal, prov(ψⱼ)⟩              -- a typed hole = postulate (24 §2, §6.5)
    obls := obls ∪ {hⱼ}
  coreTy := Π(Δ). Π(φ̄). B'                      -- (Δ) → (φ̄) → B   (refined type, denotational)
  coreTm := λ(Δ). λ(p̄). b                       -- λΔ. λp̄. body
  return (declare_def-checked coreTm : coreTy, obls)
```

- The precondition `Π(φ̄)` is **sound**: `Π(p : φ : Ω). Rest` keys its formation
  sort on the **codomain** `Rest` (`16 §1.1`, the landed `sort_pi_sigma`), so an
  Ω domain does **not** collapse the function type — it stays a `Type`. Proof
  args are at Ω (erased at runtime; discharged at the call as `φ[args]`,
  `22 §2.3`).
- The postcondition proof is **not** paired into `b` (no `Σ(B,ψ)` value, §2);
  it is the obligation `hⱼ`. `b` remains the bare carrier value, so contracts
  are erasable and the encoding is independent of the `Σ`-sort caveat.

**Refinement type** — `{x:A|φ}` lowers to its carrier; uses emit obligations:

```
elabType(Γ, {x : A | φ}) → Term:
  A' := elabType(Γ, A)                          -- the carrier (Type ℓ_A)
  _  := check(extend(Γ, A'), φ, Ω)              -- predicate MUST check at Ω under x : A; else SURFACE ERROR
  recordRefinement(A', φ)                        -- elaborator-side fact: A'-values here carry φ
  return A'                                      -- CORE TYPE IS THE CARRIER (φ not reified, §2)

check(Γ, a, {x:A|φ}):                            -- introduction: a : A used where {x:A|φ} expected
  a' := check(Γ, a, A)                           -- value at the carrier
  emit ⟨freshHole(), Γ ⊢ subst(φ, a'/x), prov⟩  -- obligation φ[a]  (22 §2.1)
  return a'                                       -- {x:A|φ} ≤ A is free (here, identity)
```

A **refined parameter** `(x : {y:A|φ})` lowers its type to `A` and contributes
`φ[x]` as an assumption in `Γ` for downstream obligations (`22 §3`).

**Goals** — `prove`/`law` lower to standalone obligations:

```
elabProve(Σ, ⟨ prove name : φ ⟩):
  φ' := check(Γ_binders, φ, Ω)                   -- the goal proposition; MUST check at Ω
  h  := freshHole();  emit ⟨h, Γ_binders ⊢ φ', prov⟩   -- standalone obligation (22, degenerate)
  bind name ↦ postulate(φ')                       -- name : φ usable as a proof term (11 §4)
  -- on discharge: certificate p with Γ ⊢ p : φ' is check'd; name ↦ p (postulate retired, §5.4)

elabLaw(Σ, ⟨ law Name (M) { fᵢ : φᵢ } ⟩):
  for fᵢ: φᵢ' := check(Γ_M, φᵢ, Ω)               -- each field a proposition; MUST check at Ω
  emit one obligation per field; the proved bundle is a record of proofs (33 §5)
```

`lemma` and attached `proof` are the same Ω-checked proof path with different
namespacing: `lemma` binds a standalone theorem in the module namespace, while
`proof` binds the theorem under `subject::proof_name`. `prop` family helpers
are checked at Ω in the family namespace and are ordinary proof terms, not a
separate kernel class.

### 6.4 `old`-capture for `space` operations

A `space` operation denotes a **state-transformer** `⟦f⟧ : S → R × S`
(`36 §4.2`); its `ensures` is a predicate relating the **pre-state**, the
`result`, and the **post-state** (`36 §4.3`). Elaboration of an `ensures ψ` on a
`space`-op binds three things in `ψ`'s scope:

```
elabSpaceEnsures(Γ, f, ψ):                       -- f : a space op, ⟦f⟧ : S → R × S
  -- bind s_pre : S, result : R, s_post : S; (result, s_post) = ⟦body⟧(s_pre)
  Γ' := extend(Γ, s_pre : S, result : R, s_post : S)
  resolve in ψ:
    old(e)       ↦ ⟦e⟧ at s_pre                   -- pre-state value (36 §4.3)
    bare cell cᵢ ↦ proj_i(s_post)                 -- post-state value
  ψ' := check(Γ', ψ, Ω)                           -- MUST check at Ω
  goal := ψ' with (result, s_post) := ⟦body⟧(s_pre)
  emit ⟨freshHole(), Γ, s_pre ⊢ goal, prov(ψ)⟩
```

**The scope guard (discriminating, not coincidental).** `old(e)` is admitted
**only** when the enclosing declaration is a `space` operation — the one place a
distinct pre-state `s_pre` exists. In a pure `view`'s `ensures` there is no
`State` effect, `s_pre ≡ s_post`, and there is no pre-state to bind: `old(e)` is
a **scope error**, rejected at elaboration before the kernel (`36 §7.3`). The
guard is the *kind of the enclosing declaration*, asserted explicitly — so the
conformance verdict flips on it (`old(c)` in a `space`-op `ensures` resolves to
`proj_i(s_pre)`; `old(x)` in a pure-`view` `ensures` is rejected), never passing
vacuously. Worked example (`36 §4.3`): `inc`'s `ensures n == old(n) + 1` denotes
to the obligation `(s_pre with .n := s_pre.n + 1).n == s_pre.n + 1`, which
computes by record-β/η (`13 §3`) to `s_pre.n + 1 == s_pre.n + 1`, discharged by
`refl` (`16 §2`).

### 6.5 The obligation-hole encoding (the `22` input)

Each contract/refinement/goal point emits an **obligation** — a triple
`⟨id, Γ ⊢ φ, provenance⟩` (`22 §1`) realized as a **typed hole** `?id : φ` in
`Γ`, admitted as a **postulate** of `φ` (`24 §2`). This is the single
representation that unifies "obligation," "typed hole," and "visible postulate":

- The program **still type-checks and runs** with open holes — each is a
  postulate in the trusted base (§5.4), so the system is *honest* about what is
  assumed.
- **Discharging** a hole means a certificate `p` with `Γ ⊢ p : φ` that the
  kernel `check`s (`18 §4.5`); the postulate is then retired and the claim turns
  `proved` (§5.4). Proving is hole-filling.
- The holes are **precisely located** (provenance) and independent — provable in
  any order / in parallel (`22 §5`).

The interaction of the spec forms is uniform through this encoding: specs
elaborate to the carrier core (Σ, Π, Ω, `Eq`) plus an obligation set; refinement
"subtyping" is the obligation `φ` on the introduction direction; and a spec
proposition may itself use earlier `prove`d lemmas, `law`s, and refinements —
specs compose by composing their obligations.

## 7. The V1→V2 interface

V1 produces, per definition, exactly what obligation generation (`22`, V2)
consumes. The interface is four things:

1. **The elaborated core term** — kernel-checkable, with the contract encodings
   of §6.3: precondition `Π` proof-arguments, carrier result and refined-
   parameter types, and the bare body. V0 re-checks it (`18 §4`); a spec program
   with a type error has no core image and is rejected (`39 §3`).
2. **The obligation-hole set** — the ordered set of `⟨id, Γ ⊢ φ, provenance⟩`
   (§6.5), one per `ensures`/refinement-introduction/`prove`/`lemma`/`proof`/
   `law`-field site, each a typed hole `?id : φ` admitted as a postulate.
3. **The at-introduction hypotheses** — each hole's `Γ` already carries the
   facts in scope where the obligation *arose*: preconditions and refined-
   parameter predicates (`22 §3`). V2 **extends** each `Γ` with path-sensitive
   facts (let-equations, case-split constructor equations, body-as-motive
   induction hypotheses — `22 §3`/`§4`); V1 provides the seed context, V2 the
   accumulation.
4. **The provenance** — source span + responsible clause per hole, for the
   diagnostics (`24`) and the protocol (`25`).

V1 does **not** generate verification conditions or walk the body for
path-sensitivity (that is V2's extractor) and does **not** discharge anything
(that is V3's prover). It hands V2 a spec-annotated, kernel-checked elaborated
form with the obligation sites marked. This is the V1→V2 contract acceptance
ties to (`docs/wp/V1-spec-syntax.md` acceptance §4).

## 8. Level-discipline reconcile

Per the standing directive, every formation in this chapter that produces or
manipulates a universe level is given its explicit level computation and
reconciled against `10-kernel/12-universes.md` and `16 §1.1` — *reconciled*, not
merely cited.

- **A spec proposition lands in Ω.** Every `requires`/`ensures`/refinement-
  predicate/`prove`/`law`-field body MUST check at `Ω_ℓ` for some `ℓ`
  (`12 §5`, `16 §1.1`: `Ω_ℓ : Type (suc ℓ)`, predicative, non-cumulative) in its
  scope; `Eq A a b : Ω_ℓ` for `A : Type ℓ` (`16 §2.1`). A body at `Type ℓ`
  rather than `Ω_ℓ` is a `TypeMismatch` at elaboration (§6.3), **not** a silent
  coercion — Ken has no `Type → Ω` inclusion (`16 §1.3`: only genuine
  sub-singletons enter Ω, and even those by explicit prelude declaration).
- **Precondition `Π` proof-argument.** `Π(p : φ : Ω_ℓ). Rest` with
  `Rest : Type ℓ'` has formation sort keyed on the **codomain** (`16 §1.1`;
  landed `sort_pi_sigma`): result `Type (max ℓ ℓ')`. The Ω **domain** does not
  lower the sort to Ω — so threading preconditions preserves the function type's
  `Type`-hood. This is the sound half of the codomain-keying (a Π *into* a
  relevant type stays relevant).
- **Refinement carrier level.** `{x:A|φ}` elaborates to the carrier
  `A : Type ℓ_A` (§6.3) — the predicate is elaborator-tracked, not reified — so
  the refinement type sits at **exactly `Type ℓ_A`**, no bump. This is the level
  reconcile's load-bearing choice: by keeping the carrier the core type, V1
  **never forms a core `Σ` over an Ω predicate**, so it does not depend on the
  kernel's `Σ`-over-Ω sort (the §2 caveat, now an Architect-confirmed erratum).
  The
  same-level refinement is the standard subset-type discipline (a subset stays
  at its carrier's universe).
- **Postcondition certificate.** `ψ[b/result] : Ω_ℓ`; its proof `p : ψ` is at Ω
  (irrelevant, erasable). `check(env, Γ, p, ψ)` is an ordinary kernel check; no
  level appears on the function beyond the carrier's.
- **`prove`/`law`.** `prove name : φ` gives `name : φ : Ω_ℓ`. A `law` of all-Ω
  fields is a conjunction — the sound `Σ`-of-Ω-into-Ω case (`16 §1.3`,
  `sort_pi_sigma` with **both** components Ω) — so the bundle is itself a
  proposition; this is the *correct* use of codomain-keying (both sides props).
- **No new universes or formers.** V1 introduces no universe or proposition
  former — it reuses Ω (`16 §1`) and the derived connectives (`16 §1.3`). So the
  reconcile reduces to: every spec body lands in Ω at its scope's level, and the
  contract encoding preserves the carrier's `Type` level. Consistent with `12`'s
  predicative, non-cumulative regime — no implicit lifts; a level-mismatched
  proposition is a `TypeMismatch`, not a coercion.
- **Named proof claims stay in the same Ω lane.** `prop`, `lemma`, and attached
  `proof` bodies all check at `Ω`; the attached form is still an ordinary proof
  term attached to a subject path, not metadata, not a proof search table, and
  not a new kernel declaration kind.

## 9. What WS-V must deliver here (V1)

The spec syntax (`requires`/`ensures`, `{x:A|φ}`, `prove`/`law`) with concrete
grammar (§6.1) and AST (§6.2); its **Ω-typing** of every proposition (§4); its
**elaboration to core** as the carrier-plus-obligation encoding (§6.3), the
`old`-capture rule for `space` ops (§6.4), and the obligation-hole form (§6.5);
the **verification status model** — the per-obligation verdict
trichotomy and the per-claim four-way epistemic status, the projection, and the
honesty guard (§5); and the **V1→V2 interface** (§7). The disposition-tag clause
spelling (`tested`/`assume`/`test`) stays reserved (§5.5).

Acceptance ties to **G2**: a real function with an `ensures` whose correct proof
is accepted (verdict `proved`, certificate kernel-`check`ed) and whose wrong
proof is rejected (verdict `not proved` — `disproved` or `unknown`, the
verdict-flip); a refinement introduction emits its obligation; an `incomplete`
claim is distinguishable from `proved` by `trusted_base()` membership (§5.4);
`old` resolves in a `space`-op `ensures` and is rejected out of scope (§6.4);
and V0's behavior is unchanged for non-spec programs (§6.2). Conformance:
`../../conformance/verify/spec-syntax/`.
