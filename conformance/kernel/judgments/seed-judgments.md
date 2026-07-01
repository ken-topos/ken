# Kernel judgments + API conformance — WP K-api seed

Format: `../../README.md`. These pin the **typing-judgment and kernel-API**
behavior that `18-judgments.md` makes implementation-ready: the four judgment
forms and the **(Conv) switch** (`18 §2`), the **bidirectional `infer`/`check`**
algorithm (`18 §3`), the **stable API surface + per-entry contract** (`18 §4.1`/
`§4.2`), the **admission gates** that fire through the API entries (`18 §4.3`),
the typed `KernelError` (`18 §4.4`), proof-checking as `check` (`18 §4.5`), and
the **trusted-base enumeration** (`18 §5`). This is the K-api WP corpus: it
**completes** the earlier K2c-series-1 judgments seed (Conv switch + SCT gate)
into the full kernel-boundary contract the build fleet codes against.

Cases tagged **(soundness)** encode a kernel soundness commitment
(`../../../spec/10-kernel/README.md §5`) and must never regress. Citations are
reconciled against the **landed** elaboration (`wp/K-api` `6a661d8`):
`18-judgments.md` §2 (Conv), §3 (bidirectional + no-guessing), §4 (API surface,
per-entry contract, admission-gate cites, typed error, `check_proof`=`check`,
freeze gating), §5 (trusted base), §6 (metatheory). Expected results are
determined by the on-`wp/K-api` spec (`12`–`18`); none required reading the
prototype.

**Signature note (real contract vs pedagogical shorthand).** `18 §4.1` is the
ground truth: every declarator is `declare_*(env: &mut GlobalEnv, level_params:
Vec<LevelVar>, …) -> KernelResult<GlobalId>` — **the kernel keys on the returned
`GlobalId`; no declarator takes a `name`** (naming is the elaborator's job).
`declare_inductive` takes a `build: FnOnce(GlobalId) -> InductiveSpec` closure
so constructors can self-reference `D`. Where a case below writes
`declare_def(env, "f", …)` the quoted name is **illustrative** (readability
only) — the behavior a case asserts is keyed on the returned `GlobalId`, never
on a name parameter.

**Build-sequencing note (`18 §4.6`).** Four gate-invoking cases below assert the
**complete-kernel** gate behavior and so go green only once the in-flight builds
land: the **W-style** admission cases ride **K1.5-build** (`dec_2vc6ytrbcbfc5`;
until it lands the kernel still runs the pre-K1.5 blanket Π-bound reject) and
the **non-Ω quotient respect** cases ride **K2c-series-2-build**. Each is tagged
`[K1.5-build]` or `[K2c-s2-build]`. This is the conformance mirror of the §4.6
freeze gate: the K-api merge Decision is itself gated on both builds being
green-and-merged, so the surface the corpus checks is the surface the code
exposes the day K-api lands.

---

## A. Admission gates fire through their API entry (one invoking test per gate)

`18 §4.3`: each admission gate runs on **every** input to its host entry, and an
"invoking" test drives the gate **through that entry**, flipping accept↔reject
on the gate condition alone. The exhaustive per-gate boundary lives in the
gate's own chapter seed (cited); these cases pin that the **API entry enforces
it**.

### A1 — SCT gate at `declare_def` (cite `17 §4`)

### kernel/judgments/declare-def-sct-admits (soundness)
- spec: `18 §4.2`/`§4.3` (`declare_def` runs SCT); `17 §4`
- given: `declare_def(env, "ack", Nat -> Nat -> Nat, <Ackermann body>)` — the
  SCT-accepted lexicographic definition from `../conversion/seed-conversion.md`
  (`conversion/sct-accept-lexicographic`)
- expect: **Ok(id)** — admitted **transparent** (δ-reducible); a subsequent
  `convert(env, ·, Nat, ack (suc^3 zero) (suc^3 zero), suc^61 zero)` returns
  `true`
- why: `declare_def` type-checks the body **then** runs `sct_check` (`18 §4.2`);
  on pass it admits transparent. Ties the API gate to the SCT criterion and to
  the δ-reduction it licenses. **Verdict-flips** with `declare-def-sct-rejects`.

### kernel/judgments/declare-def-sct-rejects (soundness)
- spec: `18 §4.2`/`§4.3`; `17 §4`
- given: `declare_def(env, "loop", Nat -> Nat, <loop x = loop x>)`
- expect: **Err(`NotTerminating`)** — admission **refused**; `env` is
  **unchanged** (the pre-admitted opaque `id` is removed, `18 §4.2` rollback);
  `loop` is not δ-reducible (never admitted)
- why: the kernel **never** admits uncertified transparent recursion (`17 §4`,
  `18 §4.2`). **Guard named:** the rejection is driven by `sct_check` returning
  `NotTerminating` on the idempotent self-loop with no strict descent (`17 §4`),
  run **after** type-checking the body — not by a type error. **Disconfirming
  check:** would `loop` also be refused if the SCT gate were *removed*? No — its
  body type-checks, so without SCT it would be wrongly admitted transparent and
  diverge under δ. The reject is **gate-gated**, not coincidental. Asserts the
  specific `NotTerminating` variant + the env rollback (structural), not bare
  `is_err`.

### kernel/judgments/declare-def-nullary-self-loop-rejects (soundness)
- spec: `18 §4.2` (`declare_def` runs SCT + `remove_last` rollback); `17 §4.1`
  (the **applied-only** precondition)
- given: `declare_def(env, "c", Nat, <c := c>)` — a **nullary** group-member
  self-reference (`c`'s body is the bare occurrence `c`, **unapplied**), from
  an `env₀` with no `c`
- expect: **`Err(KernelError::NotTerminating)`**, and **`env` identical to
  `env₀`** — the pre-admitted `Opaque c` is `remove_last`'d on the
  `.and_then(sct_check)` Err path (`18 §4.2`), so `c` is **fully absent** (not
  "present but opaque," not merely "non-δ-reducible"). Assert the **variant +
  env-unchanged**, **not** the message string.
- why: (soundness) the **exact gap** the applied-only guard closes. An
  **unapplied** group-member occurrence produces **no call edge**, so the old
  `edges.is_empty ⇒ accept` shortcut **over-accepted** `c := c` — admitting a
  transparent `c := c` whose δ-reduction is a non-terminating loop
  (`c ⇝ c ⇝ …`), a definitional cycle **inhabiting `Bottom`** (the trust-root
  hole). **Assert the observable, not the mechanism:** the guard is sound
  whether it ships as the synthetic `?`-everywhere self-loop (`17 §4.1` (1b))
  or an `occurs`-style early-reject in `collect_calls` — **both yield the
  identical observable** `Err(NotTerminating)` + rollback — so the case pins
  the **observable**, mechanism-agnostic (it drives whichever guard ships,
  never a hand-fed verdict, and never couples to one internal form).
  **Verified flip (anti-green-vs-green):** run against `wp/K2c-recursive-sct`
  **pre-guard** — `c := c` is **admitted** (`edges.is_empty ⇒ accept`, the
  bug) — and **post-guard** — rejected; the case genuinely **fails on the
  un-fixed kernel**, not for an incidental reason. **Distinct from
  `declare-def-sct-rejects`:** there the self-loop is **applied** (`loop x`,
  an edge, no descent); here it is **unapplied** (no edge) — the two arms
  cover both paths.

### kernel/judgments/declare-def-laundered-self-loop-rejects (soundness)
- spec: `18 §4.2`; `17 §4.1`
- given: an `env₀` in which `id : (Nat→Nat)→(Nat→Nat) := λx. x` is **already
  admitted transparent** (no group-member occurrence, admits with no edges;
  **not in `loop`'s group**), then
  `declare_def(env₀, "loop", Nat→Nat, <loop := id loop>)` — the group-member
  `loop` in **argument** position (unapplied): a **laundered** self-reference
  routed through the transparent passthrough `id`
- expect: **`Err(KernelError::NotTerminating)`**, and **`env` identical to
  `env₀`** (`loop` fully removed by `remove_last`; `id` untouched)
- why: (soundness) the **laundering** arm — the unapplied occurrence hides
  inside `id loop` (or any transparent passthrough / `map`), still **no call
  edge**, so the old shortcut over-accepted it
  (`id loop ⇝ loop ⇝ id loop ⇝ …`, a δ-loop inhabiting `Bottom`). Same guard,
  same **observable** assertion (`Err(NotTerminating)` + rollback,
  mechanism-agnostic), same **verified flip** (pre-guard admits
  `loop := id loop`; post-guard rejects). Pins that the applied-only
  precondition is on the **occurrence**, not the syntactic head — a group
  member unapplied **anywhere** (argument position, under a passthrough)
  forces the reject. The **recursion-through-transparent** case named in the
  finding; the second arm of the unapplied-gap net. (Fixture: `id` must be a
  genuine pre-admitted transparent identity **outside** `loop`'s group, else
  the laundering isn't real.)

### kernel/judgments/declare-def-eliminator-no-sct (soundness)
- spec: `17 §4` (SCT gates δ-recursion; eliminator recursion is already total);
  `18 §4.3`
- given: `declare_def(env, "double", Nat -> Nat, λ n. elim_Nat (λ _. Nat) zero
  (λ _ ih. suc (suc ih)) n)` — recursion via the inductive eliminator, **not** a
  self-call
- expect: **Ok** — admitted transparent; **no** SCT obligation arises
- why: SCT gates only **general** recursive δ definitions; recursion through an
  inductive eliminator is already structural and total (`14 §3`, `17 §4` scope).
  Discriminating: a checker that demanded SCT of every definition would reject
  this, and one that admitted a **non**-eliminator self-recursion *without* SCT
  would mis-accept `loop` — this case + `declare-def-sct-rejects` pin the gate
  to exactly general δ-recursion.

### A2 — Strict-positivity gate at `declare_inductive` (cite `14 §8`)

### kernel/judgments/declare-inductive-positivity-admits (soundness)
- spec: `18 §4.3`; `14 §8` (`Pos_D^+`)
- given: `declare_inductive(env, |d| <data Tree where leaf : Tree; node : Tree
  -> Tree -> Tree>)` — a strictly-positive recursive family
- expect: **Ok(id)** — type former + `leaf`/`node` admitted; `elim_Tree`
  generated
- why: every recursive occurrence of `Tree` is the **target** of `node`'s
  argument arrows, i.e. at `+` polarisation (`14 §8.2` `check-pos-arg(Tree, +,
  Tree) = true`). The positivity gate fires **inside** `declare_inductive` (`18
  §4.3`). **Verdict-flips** with `declare-inductive-positivity-rejects` on the
  polarity of the recursive occurrence alone.

### kernel/judgments/declare-inductive-positivity-rejects (soundness)
- spec: `18 §4.3`; `14 §8.2`/`§8.3` (worked example)
- given: `declare_inductive(env, |d| <data Bad where mk : (Bad -> Bool) ->
  Bad>)` — the constructor `mk` takes **one** argument of type `(Bad -> Bool)`
  and returns `Bad` (i.e. `mk : (x : (Bad -> Bool)) -> Bad`)
- expect: **Err(`PositivityViolation`)** — admission refused
- why: in the argument type `(Bad -> Bool) = (x : Bad) -> Bool`, the recursive
  `Bad` sits in the **domain**, a `-` position: `14 §8.2` `check-pos-arg(Bad, +,
  (x:Bad)->Bool) = check-pos-arg(Bad, -, Bad) ∧ … = false` — the kernel's own
  §8.3 worked example. **Guard named:** the reject is driven by the `Pos_D^-(D
  Δ_p t̄)` FAILS clause (`14 §8.1`), reached because the outer arrow flips `+ ↦
  -` over the domain. **Disconfirming check:** would this also reject if
  positivity were removed? No — it would admit a paradox. Gate-gated.
  **Disambiguation (load-bearing — this exact term is prose-ambiguous):** the
  argument type is `(Bad -> Bool)`, **not** `((Bad -> Bool) -> Bad)`. The nested
  reading `mk : ((Bad -> Bool) -> Bad) -> Bad` is **double-positive** (`Bad` at
  even arrow-depth, `- ∘ - = +`) and would be *accepted* — a different term, not
  this one. Asserts the specific `PositivityViolation` variant. The exhaustive
  positivity boundary (hidden-negative classes like `Pair (Bad3 -> Empty) Unit`,
  `14 §8.3`) is pinned in `../seed-k1.md` (`negative-bad-rejected`) — this case
  pins that **`declare_inductive` enforces it at the API entry**.

### A3 — W-style admission gate at `declare_inductive` (cite `14 §2.1`/`§8.4`)

### kernel/judgments/declare-inductive-wstyle-admits (soundness) [K1.5-build]
- spec: `18 §4.3`; `14 §2.1`/`§8.4` (gate) + `§3.1`/`§7.7` (elim/ι)
- given: `declare_inductive(env, |d| <data ITree E R where Ret : R -> ITree E R;
  Vis : (e : E.Op) -> (E.Resp e -> ITree E R) -> ITree E R>)`
- expect: **Ok(id)** — `ITree` admitted; `elim_ITree` generated with the
  Π-abstracted IH (`14 §3.1`) and W-ι (`14 §7.7`)
- why: `Vis`'s second argument `(E.Resp e -> ITree E R)` is a **W-style**
  (Π-bound) recursive occurrence — `ITree` appears **only as the target** of the
  branching arrow and the domain `E.Resp e` is `ITree`-free, so `14 §8.2`
  already accepts it (`§8.4`). Post-K1.5 the **separate** blanket Π-bound gate
  is retired, so the declaration is admitted. **Verdict-flips** under K1.5: this
  was *rejected* by the pre-K1.5 blanket gate, *accepted* now. Build-timing:
  until K1.5-build lands the kernel still runs the blanket reject (`18 §4.6`);
  the exhaustive W-style boundary is in `../inductive/seed-wstyle.md` (this pins
  the **API entry admits it**).

### kernel/judgments/declare-inductive-wstyle-rejects (soundness) [K1.5-build]
- spec: `18 §4.3`; `14 §8.2`/`§8.4`
- given: `declare_inductive(env, |d| <data Bad where mk : (Bad -> Bool) ->
  Bad>)` — argument type `(Bad -> Bool)`, as in A2
- expect: **Err(`PositivityViolation`)** — admission refused **even after** the
  blanket Π-bound gate is retired
- why: retiring the blanket gate (`14 §8.4`) leaves `§8.2` strict positivity as
  the **sole** structural test, and it **still rejects** this (`Bad` is `-` in
  the domain of `(Bad -> Bool)`, A2). **Guard named:** `Pos_D^-` failure, not
  the retired blanket gate. **Disconfirming check (the exact bug this
  targets):** would a kernel that retired the blanket gate **and** wrongly
  stopped re-running `§8.2` positivity admit this? Yes — so the reject **flips
  green→red precisely on the K1.5 over-relaxation bug** (gate removal that also
  drops positivity), which is the soundness risk K1.5 introduces. Same
  disambiguation as A2: argument is `(Bad -> Bool)`, not the nested
  double-positive form. Pairs with `declare-inductive-wstyle-admits` (accept
  ITree / reject negative). See `../inductive/seed-wstyle.md`
  `wstyle-branching-domain-not-d-free-rejected` (`(D->D)->D`) for the
  W-style-specific non-`D`-free-domain reject.

### A4 — Quotient-respect gate at `infer`/`check` on `QuotElim` (cite `16 §5.1`)

### kernel/judgments/quot-respect-admits (soundness) [K2c-s2-build]
- spec: `18 §4.3`; `16 §5.1`
- given: typing `elim_/ M f r [a] : M [a]` over `Bool / (λ _ _. Top)` with a
  **Type-target** motive `M := λ _. Bool` and a **respecting** map `f := λ _.
  true` whose respect proof `r : (x y : Bool) -> R x y -> Eq Bool (f x) (f y)`
  reduces to `Eq Bool true true ⇝ Top` — `check`/`infer` the `QuotElim`
- expect: **Ok** — the elimination type-checks (the respect proof checks against
  the `cong`/`cast` respect schema), and the i-reduction `elim_/ M f r [a] ⇝ f
  a` fires
- why: a **Type**-target quotient elim is admitted **iff** `r` checks against
  the respect schema (`16 §5.1`); `f := λ _. true` is constant so respect holds.
  **Verdict-flips** with `quot-respect-rejects` on respect-validity alone.

### kernel/judgments/quot-respect-rejects (soundness) [K2c-s2-build]
- spec: `18 §4.3`; `16 §5.1` (the closed-`Empty` respect probe)
- given: the **same** `Bool / (λ _ _. Top)`, `M := λ _. Bool`, but an
  **observing** map `f := λ x. x` — its respect obligation needs `r : (x y :
  Bool) -> Top -> Eq Bool x y`, i.e. `Eq Bool true false` for `x,y :=
  true,false`, which `⇝ Bottom` (uninhabited) — `check`/`infer` the `QuotElim`
- expect: **Err(`BadEliminator`)** — no respecting `r` exists, so the
  elimination is **refused at typing**
- why: the total relation `λ _ _. Top` equates every pair, so an observing `f`
  cannot respect it; admitting the elim would let `elim_/` observe `true` vs
  `false` through a quotient that equates them — a route to inhabiting `Empty`
  (`16 §5.1`). **Guard named:** the respect-schema `check` fails because the
  obligation `Eq Bool true false ⇝ Bottom` has no inhabitant. **Disconfirming
  check:** would the elim also be refused if the respect check were *skipped*
  (the K2c-s2 "raw-well-form non-Ω" hole)? No — it would be admitted and break
  soundness. Gate-gated. The exhaustive respect boundary is in
  `../conversion/seed-conversion.md`
  (`quotient-respect-schema-{accepts,rejects}`); this pins that
  **`infer`/`check` on `QuotElim` enforces it** (`18 §4.3`: the gate fires at
  elimination-typing, not a declarator).

### A5 — `declare_postulate` admits opaque and records in the trusted base

### kernel/judgments/declare-postulate-admits-and-records (soundness)
- spec: `18 §4.2` (`declare_postulate`); `§5` (trusted base)
- given: `declare_postulate(env, [], P)` for a closed type `P : Type 0`,
  yielding `id`; then `env.trusted_base()`
- expect: **Ok(id)** — admitted **opaque**; and `id ∈ env.trusted_base()`
- why: a postulate is an *assumed* axiom, so it is admitted opaque (never
  δ-unfolded) and **recorded** so a reviewer can see it (`18 §4.2`/`§5`).
  **Verdict-flips on membership:** the returned `GlobalId` **appears** in
  `trusted_base()` — a kernel that admitted the postulate but failed to record
  it would hide an unchecked assumption (a soundness-visibility bug). Bridges to
  the enumeration cases (group E), which pin the *exclusions*.

---

## B. The (Conv) mode switch — conversion integration (`18 §2`, `§3.2`)

(Conv) is the **single** place conversion (`17`) is called during checking — the
mode-switch fallback `t ⇐ A`: infer `t ⇒ A'`, then `convert_type(A, A')` (`18
§3.2`).

### kernel/judgments/conv-switch-delta (soundness)
- spec: `18 §3.2` (mode switch), `§2` (Conv); `17 §3`
- given: transparent `N : Type 0 := Nat`; open `n : Nat`; `check(n, N)`
- expect: **Ok** — infer `n ⇒ Nat`; the switch calls `convert_type(N, Nat)`,
  which δ-unfolds `N → Nat` and converts
- why: the (Conv) switch invokes `17` conversion including controlled δ; a
  transparent type synonym checks because conversion unfolds it. The single
  conversion call at the switch. **Verdict-flips** with `conv-switch-rejects`.

### kernel/judgments/conv-switch-eta (soundness)
- spec: `18 §3.2`, `§2`; `17 §2`
- given: `A : Type 0`; open `f : (x:A) -> A`; `check(λ (x:A). f x, (x:A) -> A)`
- expect: **Ok** — checking the λ descends, and the body switch converts `f x`
  against itself; equivalently `f ≡ λ x. f x` by Π-η
- why: conversion invoked at checking consumes η, so an η-expanded term checks
  against the un-expanded type. Ties `18 §3.2` checking to `17 §2`. (η is
  consumed by `convert`, **not** by a `check` rule — `18 §3.2`.)

### kernel/judgments/conv-switch-rejects (soundness)
- spec: `18 §3.2`, `§2`, `§4.4`
- given: open `n : Nat`; `check(n, Bool)`
- expect: **Err(`TypeMismatch{expected: Bool, found: Nat}`)** — the two
  non-converting types
- why: when inferred and expected types do not convert, the mode switch fails
  and **manufactures** `TypeMismatch` from the `false` `convert_type` (`18
  §4.4`: `convert_type` returns bare `bool`; the *caller* builds the error).
  **Guard named:** `convert_type(Bool, Nat) = false`. **Disconfirming check:**
  would `check(n, Bool)` also fail if the mode switch were broken to accept
  unconditionally? No — it would wrongly accept. Gate-gated. Asserts the
  specific `TypeMismatch` variant carrying `Nat`/`Bool`.

### kernel/judgments/conv-switch-non-cumulative (soundness)
- spec: `18 §2` (non-cumulative; no subtyping); `12 §3`
- given: `check(Type 0, Type 2)`
- expect: **Err** — infer `Type 0 ⇒ Type 1`; `convert_type(Type 2, Type 1)` is
  `false` (`Type 1 ≢ Type 2`)
- why: Ken is non-cumulative; (Conv) uses **definitional equality, not
  subsumption**, and there is no subtype relation in the kernel. A universe lift
  must be explicit (the elaborator inserts it). A checker that accepted `Type 0
  ⇐ Type 2` would be cumulative (OQ-2 DECIDED non-cumulative).
  **Level-discipline pin:** §18 introduces no level-computing rule — (Conv)
  operates *at* a universe and compares levels by `level_eq` (`12 §1`); this
  case guards the no-cumulativity invariant the level discipline rests on.

---

## C. Bidirectional algorithm — round-trips, non-inferable heads, minimal errors

### kernel/judgments/infer-check-roundtrip (soundness)
- spec: `18 §3.1`/`§3.2`
- given: open `A : Type 0`, `a : A`, `f : (x:A) -> A`; `infer(f a) ⇒ A0`, then
  `check(f a, A0)`
- expect: `infer` yields `A`; `check` at `A` **accepts** (round-trip)
- why: `infer` produces the **unique** type (no subtyping); checking the term
  against that type succeeds — the two syntax-directed modes agree (`18 §3`).
  Open terms, an application spine.

### kernel/judgments/infer-rejects-non-inferable-head (soundness)
- spec: `18 §3.1` (non-inferable heads fail in infer mode), `§3.2`
- given: a bare lambda `t := λ (x:A). x` with `A : Type 0`: (i) `infer(t)`; (ii)
  `check(t, (x:A) -> A)`
- expect: (i) **Err** — `infer` fails on the non-inferable head (λ carries no
  type to read off); (ii) **Ok** — the same λ **checks** against the supplied Π
  type
- why: `18 §3.1` lists `Lam | Pair | Refl | QuotClass | TruncElt` as
  **non-inferable** — they fail in infer mode and are reached only in checking
  mode (`§3.2`). **Guard named:** the explicit non-inferable arm of `infer`'s
  head dispatch. **Verdict-flips on mode** (infer rejects / check accepts the
  *same* term) — a verdict-independent structural pin on the bidirectional
  split. **Disconfirming check:** would `infer(λ…)` also fail if the kernel
  "helpfully" guessed the domain? No — it would *succeed*, absorbing elaborator
  logic into the TCB (`18 §3.3`). So the failure is guard-gated, and a kernel
  that passed (i) would have lost TCB-minimality.

### kernel/judgments/no-unification (soundness)
- spec: `18 §3.3` (kernel receives fully-explicit core terms; no metavariables)
- given: a core term carrying a genuine metavariable / missing annotation that
  would require the kernel to **solve** it to type the term
- expect: **Err** — the kernel does not invent the annotation or solve the
  metavariable
- why: the kernel performs **no** unification, metavariable solving, or implicit
  insertion (`18 §3.3`, `§7`); fully-explicit core terms are the elaborator's
  responsibility. A kernel that solved the gap would have absorbed elaborator
  logic into the TCB. Pins trusted-boundary minimality (complements
  `infer-rejects-non-inferable-head`, which pins the concrete non-inferable
  heads).

### kernel/judgments/ill-typed-type-mismatch (soundness)
- spec: `18 §4.2` (`infer`/`check` errors), `§4.4` (`TypeMismatch`)
- given: open `f : (x:Nat) -> Nat`; the application `f true` (`true : Bool`)
- expect: **Err(`TypeMismatch{expected: Nat, found: Bool}`)** — naming the
  argument `true` and the two non-converting types
- why: `App(f, u)` checks the argument against the domain (`18 §3.1`); `Bool ≢
  Nat` at the argument position. The error is **minimal and precise** — the
  failing subterm and the two types — and the kernel does no recovery / proof
  search / unification. Asserts the specific variant + payload (`18 §4.4`), not
  bare `is_err`.

### kernel/judgments/ill-typed-scope-error (soundness)
- spec: `18 §3.1` (`Var(i)` lookup), `§4.4` (`VarOutOfScope`)
- given: a term referencing de Bruijn index `i` with `i ∉ Γ` under the given
  `ctx` (e.g. `infer` of `Var(3)` in a context of depth 2)
- expect: **Err(`VarOutOfScope{index, depth}`)**
- why: `infer`'s `Var(i)` arm returns `VarOutOfScope` when `i` is not bound (`18
  §3.1`). A distinct rejection **class** from `TypeMismatch` — the scope failure
  is structural, caught before any conversion. **Guard named:** `ctx.lookup(i)`
  miss. **Disconfirming check:** an **in-scope** variable infers its binding
  type with no error — the verdict flips on scope alone. Asserts the
  `VarOutOfScope` variant (a different variant guards a different bug — `18
  §4.4` honesty).

### kernel/judgments/ill-typed-universe-error (soundness)
- spec: `18 §3.1` (`inferUniv`), `§4.4` (`UniverseInconsistency`)
- given: a Π-formation `(x : zero) -> Bool` whose **domain** position holds a
  term `zero : Nat` that is **not** a type; `infer((x : zero) -> Bool)`
- expect: **Err(`UniverseInconsistency{…}`)** — the domain does not classify as
  `Type ℓ`/`Ω_ℓ`
- why: `infer`'s `Pi(A, B)` arm calls `inferUniv(A)`, which requires `whnf(infer
  A) = Type ℓ` or `Ω_ℓ` and fails `UniverseInconsistency` otherwise (`18 §3.1`
  notation). A third distinct rejection class. **Guard named:** the `inferUniv`
  universe requirement. **Disconfirming check:** a genuine type domain (e.g. `(x
  : Nat) -> Bool`) infers `Type 0` with no error — flips on the domain-is-a-type
  condition. Asserts the `UniverseInconsistency` variant. The three classes
  (`TypeMismatch` / `VarOutOfScope` / `UniverseInconsistency`) pin the §4.4
  claim that the error is **variant-specific**, not one uniform payload.

---

## D. Proof checking is `check` — there is no `check_proof` (`18 §4.5`)

`18 §4.5`: the prover returns a certificate **term**; checking it **is** typing
— `check(env, ctx, proof, goal)`. There is **no** separate `check_proof` entry
(the de Bruijn criterion is one `check` call). The cases below call `check`.

### kernel/judgments/certificate-recheck-valid (soundness)
- spec: `18 §4.5` (`check_proof` ≡ `check`)
- given: proposition `G := Eq Nat (add 1 1) 2` and certificate term `proof :=
  refl 2`; `check(env, ·, proof, G)`
- expect: **Ok** — the kernel re-derives the type: `refl 2 : Eq Nat 2 2`, and
  `Eq Nat (add 1 1) 2` converts to `Eq Nat 2 2` because `add 1 1` prim-reduces
  to `2`
- why: a proof IS a term; checking it IS typing (`18 §4.5`). The kernel
  re-checks the prover's certificate — nothing the prover claims is trusted
  until the kernel re-derives it (the de Bruijn criterion). **Verdict-flips**
  with the wrong-certificate case.

### kernel/judgments/certificate-recheck-rejects (soundness)
- spec: `18 §4.5`; `§5` (trusted base); `16 §2.2`
- given: a **false** proposition `G' := Eq Nat 1 2` and a plausible wrong
  certificate `c := refl 1`; `check(env, ·, c, G')`
- expect: **Err** — infer `refl 1 ⇒ Eq Nat 1 1`; `convert_type(Eq Nat 1 2, Eq
  Nat 1 1)` is `false` (distinct closed numerals reduce `Eq Nat 1 2` to
  `Bottom`, uninhabited)
- why: a wrong certificate cannot make a false proposition inhabited. The
  kernel's re-check is the **soundness firewall** around the untrusted prover/Z3
  — no false `proved` (`18 §4.5`/`§5`). **Guard named:** the (Conv) mode-switch
  `convert_type` returning `false`. **Disconfirming check:** would `check`
  accept `c` if it *trusted* the prover's claimed type instead of re-deriving
  it? Yes — which is exactly the firewall this pins. Gate-gated on the re-check.

---

## E. Trusted-base enumeration (`18 §5`)

### kernel/judgments/trusted-base-enumerate (soundness)
- spec: `18 §5` (`GlobalEnv::trusted_base()`)
- given: an `env` with a `declare_postulate` (axiom `P`), a `declare_primitive`
  (`add`), several `declare_def`s, and a `declare_inductive` (`Nat`); call
  `env.trusted_base()`
- expect: **enumerates exactly** `{P, add}` (their `GlobalId`s) — and **not**
  the definitions, **not** the inductive `Nat`, **not** the prelude
  (`Top`/`Bottom`)
- why: soundness rests on exactly the kernel code + registered primitives +
  admitted postulates (`18 §5`); the kernel MUST enumerate (2)+(3) so a reviewer
  sees every unchecked assumption. **Structural verdict-flip on set
  membership:** postulates + primitives **in**, defs/inductives/prelude **out**.
  A bug that included a `declare_def` would over-report the TCB; one that
  dropped `P` would hide an axiom; one that included `Top`/`Bottom` would
  mis-count the re-checked prelude as trusted — each flips this set.
  Verdict-independent of any type-checking outcome.

### kernel/judgments/trusted-base-idiomatic-empty (soundness)
- spec: `18 §5`
- given: an `env` built only from `declare_def` and `declare_inductive` over the
  standard primitives — **no** postulates
- expect: `trusted_base()` lists only the registered primitives; **no
  postulates**, and still **excludes** the prelude `Top`/`Bottom`
- why: a program that adds no axioms depends on nothing beyond the kernel +
  primitives. Idiomatic Ken adds no postulates; any classical axiom, if used,
  appears here and is visible (`12 §5.2`). Pins that inductives and definitions
  are **re-checked, not trusted**, so they never enter the trusted base.

---

## F. Regression — K1/K2/K2c-series-1 judgments unchanged

### kernel/judgments/k1-k2-judgments-still-green (soundness)
- spec: `../../../spec/10-kernel/README.md §5`
- given: all K1/K2 check/infer cases already pinned (`../seed-k1.md`,
  `../seed-kernel.md`, `../observational/seed-observational.md`) and the
  series-1 Conv-switch + SCT cases above
- expect: **all pass** — K-api freezes the API surface and collects the judgment
  forms; it does not change the typing relation K1/K2/K2c fixed
- why: the bidirectional algorithm and the API surface K1/K2 established must
  not regress; K-api only **completes the contract** (per-entry pre/post/error,
  the admission-gate cites, the typed error, the trusted-base enumeration) over
  the unchanged typing relation. Regression gate for the kernel boundary freeze.
