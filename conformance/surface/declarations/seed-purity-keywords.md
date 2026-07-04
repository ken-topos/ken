# SURF-1 (purity keywords `const`/`fn`/`proc`) conformance — seed cases

Format: `../../README.md`. These pin the **purity-keyword discipline** that
**SURF-1** delivers (`docs/program/wp/purity-keywords-effect-polymorphism.md`,
`spec/30-surface/33-declarations.md §1`, `spec/30-surface/36-effects.md`): the
single definition keyword `view` is retired for a **three-keyword split that
agrees, by a checked bidirectional rule, with a definition's static purity and
arity** —

- **`const`** — a **zero-param pure** value (`33 §1`'s pure `let`/nullary
  value);
- **`fn`** — a **pure function with ≥1 parameter** (statically, unconditionally
  pure — effect row is the **closed empty set `∅`**, no row variable);
- **`proc`** — **anything at least potentially impure/imperative** at any arity
  (a concrete non-empty row, an effect-polymorphic row *variable*, or a `space`
  op).

The keyword is a **reliable signal**: reading `fn` guarantees "unconditionally
pure mathematical function"; `proc` warns "at least potentially impure." Purity
is a **checked declaration at the definition site**, not a convention. These
cases pin the **non-row-polymorphic** slice (frame AC1/AC1a/AC4/AC5/AC6/AC7,
PK1–PK7) **and** the **row-polymorphic** slice (frame AC2/AC3, PK8–PK9) authored
against **Architect's D1 row-variable ruling** (`evt_53ybqtzjfv7yx`) — the
`[e]`/`[E | e]` surface is reconciled against spec-author's landed §36 `§1.5`
transcription at the merge gate.

**Trust posture.** The keyword split is a **surface grammar + elaborator +
effect-checker** discipline; effects are **outer-ring** (`36 §2`, `OQ-8`
DECIDED) and the **kernel is untouched** (frame AC5) — it re-checks only the
pure denotation (`ITree`/Π/Σ/inductive), which is keyword-agnostic. So a bug in
the purity check **cannot make the kernel unsound** (the emitted core term is
still a well-typed pure tree); **no case here is `(soundness)`** in the
kernel-trust-root sense. What the discipline *does* guarantee is a **static
reliable-signal property** the verification core (`../../20-verification/`) and
downstream readers rest on: a **`fn`/`const` cannot silently perform an
effect**, and its dual, a **`proc` cannot silently over-claim impurity**. Both
directions are load-bearing (`33 §1`, frame §2.3 "no silent disagreement"), so
each is pinned by a **discriminating** case whose verdict **flips** on the
targeted bug (right keyword accepts, wrong keyword rejects), holding the **body
fixed** so the flip is attributable to the keyword alone — never green-vs-green
(`discriminating-conformance-verdict-must-flip`, COORDINATION §7).

**Build-FORCING — the whole split is RED on `main` until built (do NOT hand-feed
green).** Grounding (`origin/main @ 24a414b`, verified at authoring): the
surface keyword today is **`view`** (`32-grammar` L16–17
`decl ::= "view" ident …`; `31-lexical §4` reserved-keyword list has `view`, not
`const`/`fn`/`proc`); the `const`/`fn`/`proc` grammar and the bidirectional
purity check are the SURF-1 deliverable (D2), not landed. So **every keyword
case below flips green only when the build lands the split** — authored as the
acceptance *target*, like `../effects/seed-effects.md`'s EFF6 direct-state
cases, **not** green against a pre-supplied verdict
([[conformance-hand-feeds-the-deliverable]]). The case **defines "done."**

**Tags.** `(oracle)` — confirmed at build time by the Spec enclave (safe: not in
the kernel TCB). The **keyword spellings `const`/`fn`/`proc` are FIXED**
(operator ruling, frame §2.4 — do **not** oracle-tag or propose alternates) and
the **classification rule** (static purity + arity, frame §2.1–2.3) is
**normative**; what stays `(oracle)` is the surrounding **effect surface
*spelling*** (`visits ρ`, `perform`, effect names `FS`/`Console`/`Clock`,
`becomes`, `old`/`ensures` — `36` is normative-for-model, `OQ-syntax` for
spelling), plus the enclave's own **bounded sub-decisions** (frame §5), tagged
at their case: **mismatch severity** (hard error vs lint — spec-author owns,
frame recommends hard error) and the **implicit-param arity edge** (does `const`
count implicit `{A}` — spec-author owns, frame recommends implicit-allowed).
`(property)` — an invariant over many inputs / a structural (TCB-surface,
migration) assertion, not a single trace. **Every case here is also
build-forcing** (red until D2 lands) per the note above; not repeated per
header.

**Row-polymorphic slice — authored against D1 (frame AC2/AC3, PK8–PK9).**
Architect's D1 ruling (`evt_53ybqtzjfv7yx`) pins the **row variable** as a bare
row `[e]` (`e` an implicit param, `39`) with an optional open-row tail `[E | e]`
(`RowType::Join(Concrete, Var)`), **required in the declared type** (§3.1:
effects recoverable from the type). PK8 (`proc` covers the polymorphic case +
the pure-instantiation round-trip) and PK9 (static closure at instantiation +
the fail-closed single-arm residual) are written against that ruling — the
`[e]`/`[E | e]` **surface spelling stays `(oracle)`** (reconciled against
spec-author's landed §36 `§1.5` transcription at the merge gate, the CAT-1
parallel-author-then-reconcile posture), but the **structure**
(variable-in-the-type, keyword `proc`, static closure via `apply_subst`) is
normative from the ruling. `traverse` is the first surface consumer and **gates
CAT-2/Traversable**.

**Flag to spec-author (independent-checker; not silently resolved).**

1. **`const` keyword collides with a landed def name.** `32-grammar` L224 uses
   **`const`** as an example view name — the K combinator
   `view const (A) (B) (x) (y) : A = x`. Making `const` a reserved keyword
   (`31-lexical §4`) makes that name un-spellable — a **D4 migration hazard**:
   the combinator must be renamed (e.g. `konst`/`always`) wherever it is a
   *definition*, distinct from the keyword. Grep `\bconst\b` as a def head
   across `packages/*`/`prelude`/`examples` in D4.
2. **The proc-must-earn-impurity vs over-declaration seam (frame §2.3 ↔
   `36 §1.4`).** `36 §1.4` **allows over-declaration** (`ρ_inf ⊆ ρ_decl`;
   declaring an unused effect is a legal interface upper bound). Frame §2.3 says
   a def whose effects are **provably closed-empty must be `fn`**. This bites a
   `proc` that **declares** a non-empty row as headroom but whose **body infers
   `∅`** (PK2b): does the declared-row headroom legitimize `proc`, or does
   provable purity force `fn` regardless? The clean case (`proc`, **no**
   declared row, pure body → should-be-`fn`) is unambiguous and pinned (PK2a);
   the over-declared edge is `(oracle)`, flagged for the §5 pin.

**Citations.** `33-declarations.md §1` (`view`/`let` → Π/λ; generic implicit
params; `let` = nullary view), `§6` (operators = ordinary defs, symbolic names);
`36-effects.md §1.1` (row lattice, latent arrow `A →[ρ] B`, pure row `∅`),
`§1.2` (transitive `infer_row`: `perform_E → {E}`, `g → row(g)`), `§1.4`
(declared-row `⊆` check, escape error + witness; "no `visits` ⇒ `ρ_decl = ∅`"),
`§2.4` (denotation `⟦·⟧`, pure collapse `ITree 𝟘 R ≅ R`), `§4.1`
(`space`/`becomes` → `State S`, one label per space), `§4.3` (bounded Hoare,
`old` scoped to the op), `§6` (a row-`∅` def is a mathematical function;
`ensures` value-level); `31-lexical §4` (keywords stay ASCII — Unicode is for
**operators**), `32-grammar §1` (`decl` production,
`effects ::= "visits" [ … ]`). Cross: `12 §2`/`§3` (predicative, non-cumulative
levels), frame §2 (fixed classification rule), §4 (acceptance), §5 (bounded
enclave sub-decisions).

Companion: this seed is **additive** over `../effects/seed-effects.md` (which
pins the keyword-**agnostic** row inference/escape under `view`) — SURF-1 layers
the **keyword↔(arity, purity)** contract on top.

---

## PK1 — `fn` is a reliable purity signal, forward direction (frame AC1)

A `fn` asserts the **closed-empty row `∅`** (`33 §1`: unconditionally pure). The
check runs the **body direction** (frame §2.3): if `infer_row(body) ⊄ ∅` — the
body performs or transitively infers **any** effect — the purity claim is false
and it is a **compile error**. Pinned **per effect source** (frame AC1: direct
`perform`, a called `proc`, a `space` op), each as a **two-arm** net: the
**identical body** under `proc` (with the matching declared row) **accepts**, so
the `fn` rejection is attributable to the keyword, not the body.

### surface/declarations/fn-direct-perform-rejected (oracle)
- spec: `33 §1` (`fn` = closed-empty row), `36 §1.2` (`perform_E → {E}`), `§1.4`
  (escape), frame §2.3/AC1
- given: an effect producer `greet` where `greet s ⤳ perform (Write s)`
  (`Console`, `36 §2.2`). Two arms, **identical body** `= greet x`:
  (a) `proc announce (x : String) : Unit visits [Console] = greet x`;
  (b) `fn  announce (x : String) : Unit               = greet x`.
- expect: (a) **accepts** — `proc` + declared `[Console]` matches
  `infer_row = [Console]` (`36 §1.4`). (b) **rejects** — `fn` asserts `∅`, but
  `infer_row = [Console] ⊄ ∅`; a **false-purity** static error naming effect
  **`Console`** and a **witness** (the `greet x` call / its `perform`, `§1.4`).
  Assert the **specific** error variant
  (`assert-specific-error-variant-not-is-err`), not a bare `is_err()`.
- why: the **direct-`perform` source** of the forward check. Verdict **flips**
  on the keyword (proc accepts, fn rejects), body held fixed. The targeted bug —
  a checker that skips the body-direction purity obligation for `fn` (treats
  `fn` like the old row-agnostic `view`) — **accepts (b)** ⇒ no flip ⇒ guards
  nothing; the pair pins it. Disconfirming: `greet` is well-formed in **both**
  arms, so arm (b) cannot reject for an unrelated (out-of-scope) reason — the
  reject is attributable to the purity claim.

### surface/declarations/fn-calls-proc-rejected (oracle)
- spec: `33 §1`, `36 §1.2` (`g → row(g)`, transitive closure), `§1.4`, frame
  §2.3/AC1
- given: a landed `proc read_config (p : String) : Config visits [FS] = …` (a
  boundary op, `36 §1`). Two arms, **identical body** `= read_config "/x"`:
  (a) `proc load () : Config visits [FS] = read_config "/x"`;
  (b) `fn  load () : Config             = read_config "/x"`.
- expect: (a) **accepts** — `infer_row = [FS]` (the callee's latent row released
  at the call, `36 §1.2`), matches declared `[FS]`. (b) **rejects** — the effect
  reaches `fn`'s `∅` **transitively** (not a syntactic `perform` in `load`'s own
  body); false-purity error naming **`FS`** with the `read_config` call as
  witness.
- why: the **called-`proc` (transitive)** source — the effect enters through the
  **call graph**, not a local `perform`, catching a checker that only scans for
  a syntactic `perform` in the immediate body. Verdict flips on the keyword; the
  `proc` accept-arm is the flip. Mechanism-consistent with the direct case (same
  "fn's `∅` must contain `ρ_inf`"), differing only in the effect's **source**.
  (`load` is nullary + effectful — a **`proc`**, not a `const`; see PK3.)

### surface/declarations/fn-space-op-rejected (oracle)
- spec: `33 §1`, `36 §4.1` (`space`/`becomes` → `State S`, one label per space),
  `§1.4`, frame §2.1 (a `space` op is `proc`), AC1
- given: a `space Counter { mut n : Int = 0 ; … }`; the increment op with
  **identical body** `= n becomes n + 1`:
  (a) `proc inc () : Unit visits [Counter] = n becomes n + 1`;
  (b) `fn  inc () : Unit                   = n becomes n + 1`.
- expect: (a) **accepts** — `becomes` desugars to `Get`-then-`Put` on the
  space's `State S` (`36 §4.1`), contributing the space label `[Counter]`,
  matched by the declared row. (b) **rejects** — a `space` op is impure by
  construction (frame §2.1: a `space`/imperative op is decisively `proc`); `fn`
  cannot carry `[Counter]`. (`becomes` outside a `space` is a separate `§7.3.4`
  error — here both arms are **inside** `Counter`, isolating the keyword.)
- why: the **`space`-op source** — the third distinct way impurity enters, and
  the one that is imperative rather than a `perform`/call. Frame §2.1 puts a
  `space` op **decisively** on the `proc` side; this pins that a `fn` can never
  host one. Verdict flips on the keyword. Companion to EFF4
  `space-becomes-threads-state` (keyword-agnostic there).

---

## PK2 — `fn`/`proc` reliable signal, reverse direction (frame AC1, §2.3)

The signal is reliable only if it **cannot lie in either direction**: a `proc`
must **earn** its "potentially impure" claim. A definition whose effects are
**provably the closed-empty row** must be `fn` (≥1 param) or `const` (zero
param) — a `proc` there is a **mismatch** (frame §2.3/§5). This is the piece
EFF1 does **not** cover: EFF1's `⊆` check allows over-declaration but never keys
on the *keyword*.

### surface/declarations/proc-pure-should-be-fn (oracle)
- spec: `33 §1`, `36 §1.4` ("no `visits` ⇒ `ρ_decl = ∅`"; `infer_row` of a pure
  body = `∅`), frame §2.3/§5/AC1
- given: a **pure** body, **no declared row**, ≥1 param, under two keywords:
  (a) `fn   add (x y : Int) : Int = x + y`;
  (b) `proc add (x y : Int) : Int = x + y`.
- expect: (a) **accepts** — `fn` + `infer_row = ∅` + ≥1 param (the required
  keyword for a pure ≥1-param def). (b) **flagged** — `proc` claims potential
  impurity, but the body is **provably pure** (`ρ_inf = ∅`, no row variable, no
  `space` op), so the claim is false: a `proc`-should-be-`fn` mismatch.
  **Severity `(oracle)`** — hard error vs lint is spec-author's §5 pin (frame
  **recommends hard error**). Pin the **concept** (flagged, not silently
  accepted); do **not** freeze error-vs-lint until §5 lands.
- why: the **reverse-direction** guard — the bidirectionality (frame §2.3 "no
  silent disagreement"). The targeted bug — a checker that runs **only** the
  forward direction (PK1: `fn`-cannot-perform) but **not** the reverse
  (`proc`-must-earn-impurity) — **accepts (b) silently** ⇒ no flip ⇒ the reverse
  guard is vacuous. The pair pins that **both** directions are enforced. Verdict
  flips on the keyword, body held fixed.

### surface/declarations/proc-overdeclared-headroom-oracle (oracle)
- spec: `36 §1.4` (over-declaration `ρ_inf ⊆ ρ_decl` **allowed**), frame §2.3/§5
- given: a `proc` that **declares** a non-empty row as interface headroom but
  whose **body infers `∅`**:
  `proc stable (x : Int) : Int visits [Console] = x + 1` — never performs
  `Console`.
- expect: **`(oracle)` — verdict deferred to spec-author's §5 pin.** Two
  coherent readings, unsettled: (i) **legit `proc`** — the **declared** row is
  non-empty and `36 §1.4` blesses declaring more than used (a stable interface
  reserving `Console` headroom); or (ii) **mismatch** — the keyword classifies
  on **provable static purity** (`ρ_inf = ∅`), so purity forces `fn`
  **regardless** of the headroom. Do **not** pin a verdict; this case **holds
  the seam open** until §5 resolves it (flag 2).
- why: an **independent-checker catch** — a genuine interaction between the
  frame's reverse check and `36 §1.4`'s over-declaration allowance, surfaced
  (not silently resolved) to the author via the leader
  ([[surface-the-seam-need-not-your-preferred-mechanism]]: state the seam, leave
  the resolution to the owner). Pins the **granularity** — PK2a locks the
  unambiguous no-declared-row case; this marks the deferred degree of freedom
  rather than over-freezing it.

---

## PK3 — `const` vs `fn` by arity (frame AC1a)

Among **pure** definitions the keyword is keyed on **arity**: a **zero-param**
pure def **must** be `const` (referential transparency ⇒ it *is* a constant); a
**≥1-param** pure def **must** be `fn`. The wrong keyword is flagged.

### surface/declarations/const-zero-param-required (oracle)
- spec: `33 §1` (`const` = pure zero-param value, subsumes `let`), frame
  §2.1/§2.5/AC1a
- given: a **pure zero-param** value under two keywords:
  (a) `const answer : Int = 40 + 2`;
  (b) `fn    answer : Int = 40 + 2`.
- expect: (a) **accepts** — zero explicit value params + provably-`∅` row ⇒
  `const` (the required keyword). (b) **flagged** — a zero-param `fn` should be
  `const` (frame §2.5: the honest signal for a nullary pure def; by referential
  transparency it always yields the same value). **Severity `(oracle)`** (§5, as
  PK2a). Pin the concept.
- why: the **zero-param end** of the arity split. Verdict flips on the keyword,
  body held fixed. The targeted bug — an arity-blind checker accepting any pure
  keyword — accepts **both** ⇒ no flip. Companion to PK3b (the ≥1-param end):
  the two ends make the `const`/`fn` boundary discriminating on arity both ways.

### surface/declarations/fn-one-param-required (oracle)
- spec: `33 §1` (`fn` = pure ≥1-param), frame §2.1/AC1a
- given: a **pure ≥1-param** function under two keywords:
  (a) `fn    triple (n : Int) : Int = n + n + n`;
  (b) `const triple (n : Int) : Int = n + n + n`.
- expect: (a) **accepts** — ≥1 explicit value param + provably-`∅` row ⇒ `fn`.
  (b) **rejects** — a `const` with a value parameter is not a zero-param value;
      a `const`-with-param arity error (a **hard** error, not a §5 lint —
      `const` is *defined* as zero-param, so a parameter is a category error at
      the keyword, independent of the §5 pure/impure severity choice).
- why: the **≥1-param end** of the arity split. Verdict flips on the keyword.
  Mechanism-consistent with PK3a: together they pin that among pure defs arity
  **exactly** selects `const` (0) vs `fn` (≥1) — a mis-selection either way is
  caught.

### surface/declarations/const-implicit-param-edge (oracle)
- spec: `33 §1` (generic implicit params `view id {A : Type} …`), frame §5
  (bounded sub-decision: does "zero parameter" count implicit type/level params)
- given: a polymorphic constant with an **implicit** type param and **no
  explicit value param**: `const nil {A : Type} : List A = Nil A`.
- expect: **`(oracle)` — verdict deferred to spec-author's §5 pin.** The bounded
  sub-decision: is `const` **zero explicit *value* params** (implicit type/level
  allowed ⇒ `nil` **is** a `const` — frame **recommends** this, a constant
  *family*), or truly **zero binders** (⇒ `nil` must be `fn`/other)? Ground on
  `39`'s implicit-param machinery. Pin the **shape** (a polymorphic constant
  family is the case at issue); do **not** freeze the count rule until §5 lands.
- why: the **implicit-param edge** the frame routes to spec-author (§5).
  Authoring it `(oracle)` on the concept — rather than guessing `const` or `fn`
  — matches the granularity discipline: pin the shape, tag the deferred degree
  of freedom, so the case cannot **falsely fail** a valid build once §5
  finalizes the count rule ([[spec-exact-granularity]] T1 half).

---

## PK4 — `fn`'s effect-free guarantee, for verification (frame AC4)

A pure-typed `fn`/`const` (row `∅`) denotes to `ITree 𝟘 R ≅ R` — the elaborator
**collapses** it to the plain term (`36 §2.4`) — so the verification core may
treat it as a **mathematical function** whose `ensures` are **value-level**
(`36 §6`, restated for `fn`). A `proc` (impure) denotes to a real `Vis`-bearing
tree and its `ensures` may be **state-relative** (`old`, post-state, `36 §4.3`).
The keyword thus determines the verification treatment.

### surface/declarations/fn-pure-ensures-value-level (oracle)
- spec: `36 §2.4` (pure collapse), `§6` (row-`∅` def = mathematical function,
  value-level `ensures`), `33 §1`, frame AC4
- given: a pure `fn` with a value-level postcondition:
  `fn succ (n : Int) : Int ensures result == n + 1 = n + 1`.
- expect: **accepts and the `ensures` discharges** — `succ` has row `∅`,
  collapses to the plain term (`ITree 𝟘 Int ≅ Int`, `§2.4`); the obligation
  `result == n + 1` is a **value** equation, discharged by computation/`refl`
  (`16 §2`), **no world-state**. The verification layer sees a mathematical
  function.
- why: AC4's **pure half** — the `∅`-row + collapse is the certificate the
  verification core rests on (mirrors EFF5 `pure-view-usable-in-pure-context`,
  re-keyed on the new `fn` keyword). Pairs with PK4b for the flip.

### surface/declarations/fn-no-old-in-ensures (oracle)
- spec: `36 §4.3` (`old(e)` = `e` in the **pre-state**; well-defined only when
  the denotation *names* a pre-state), `§6`, frame AC4
- given: the **same** `old(n)`-style postcondition under two keywords:
  (a) `proc inc () : Unit visits [Counter]` with `ensures n == old(n) + 1` and
      body `n becomes n + 1` (inside `space Counter`);
  (b) `fn bump (n : Int) : Int` with `ensures result == old(n) + 1` and body
      `n + 1`.
- expect: (a) **accepts and discharges** — `old(n)` names the pre-state of the
  `space` op's `State S` transformer, obligation computes to `refl` (EFF4
  `space-old-scoped-to-ensures`, `36 §4.3`). (b) **rejects** — a pure `fn` has
  **no pre-state** to name, so `old(_)` in its `ensures` is a category error
  (`OldOutsideStatefulOp`, kind `(oracle)`).
- why: the discriminating face of AC4 — the keyword decides whether a
  **state-relative** postcondition (`old`) is even well-formed. Verdict
  **flips** on the keyword, postcondition held fixed: the `proc` space-op
  accepts it, the pure `fn` rejects it. A checker admitting `old` in a `fn` has
  lost the "pure ⇒ mathematical function" guarantee the verification core
  depends on.

---

## PK5 — kernel-untouched (frame AC5)

### surface/declarations/keywords-kernel-untouched (property)
- spec: `36 §2` (effects/rows are outer-ring; the kernel has no effect notion),
  `33 §1` (`view`/`const`/`fn`/`proc` all elaborate to Π/λ defs), frame AC5/§2.6
- given: the delivered `const`/`fn`/`proc` grammar + the bidirectional purity
  checker.
- expect: **`git diff origin/main -- crates/ken-kernel/` stays empty** and
  `trusted_base()` is **byte-unchanged** — **no new `Term`/`Decl` variant**, no
  new `declare_primitive`/`declare_postulate`. The split is entirely **surface
  parse + elaborator + effect-checker**; a `const`/`fn`/`proc` def elaborates to
  the **same** Π/λ core term its `view` predecessor did (the keyword is erased
  before the kernel), and a pure def still collapses to the plain term
  (`36 §2.4`). The purity **check** is an elaboration-time diagnostic
  (`36 §7.3`), never a kernel rule.
- why: frame AC5 is **structural**
  ([[tested-not-trusted-posture-needs-reachability-precondition]]: a
  producer-grepped build obligation, not a runtime assertion). If a build path
  ever needs a kernel `Term`/`Decl` variant for a keyword, **that is the
  finding** (the split has mis-scoped out of the outer ring). Asserts the
  **absence** of a kernel delta — no value-flip. (`property`.)

---

## PK6 — migration is green and complete (frame AC6)

### surface/declarations/migration-view-fully-retired (property)
- spec: frame D4/AC6 (`view` → `const`/`fn`/`proc`, classified by the checker's
  own inference), `31-lexical §4` (`view` leaves the reserved set)
- given: the migrated corpus — `prelude`, `packages/*` (incl. CAT-1's
  `lawful-classes`/`lawful-functors`), `examples/rosetta/*`, doc snippets.
- expect: (a) **no `.ken` retains `view`** —
  `grep -rn '\bview\b' packages prelude examples` (as a **decl head**) returns
  empty; every def carries `const`/`fn`/`proc` **as classified by the checker's
  own purity inference** (mechanical + checked, not hand-judged — frame D4). (b)
  `cargo test --workspace` **green**. (c) the **rosetta** corpus still passes
  **16/0** (frame AC6). (d) the `const`-combinator collision (flag 1) is
  resolved — no def named `const` survives.
- why: AC6 — migration completeness as a **structural** check, not a per-file
  trace ([[two-arm-producer-needs-a-case-per-arm]]: the grep asserts the absence
  of a catch-all `view` remnant). Build-forcing: red until D4 lands; a
  **partial** migration (some `view` left, or a hand-misclassified `fn` that
  should be `proc`) fails (a) or (b). The checker-classified requirement is what
  makes it *checked*, not opinion.

---

## PK7 — Unicode surface parses identically (frame AC7, D3)

### surface/declarations/unicode-twin-identical (oracle)
- spec: frame D3/AC7 (Unicode surface, BL3), `31-lexical §4` (**keywords stay
  ASCII words**; Unicode is for **operators**/symbols — `→`, `λ`, `∀`, `Σ`, `Ω`,
  `⊑`), `36 §2.4` (elaboration target)
- given: a `.ken` module in **Unicode operator/symbol surface** and its
  **ASCII-digraph twin** (`->`, `\`, `forall`, …) — **identical keywords**
  `const`/`fn`/`proc` in both (keywords are not Unicode-ified, `31-lexical §4`).
- expect: the Unicode twin **elaborates to the byte-identical core term** as the
  ASCII twin (assert **term identity**, not merely "both elaborate"). Whether
  **both** spellings stay accepted (lexer accepts Unicode + ASCII alias, emits
  Unicode — frame **recommends**) **or** the corpus is **converted** to Unicode
  and only that is green is spec-author's **D3 `(oracle)`** decision — pin the
  **equivalence** (same core term), tag the accept-both-vs-convert policy.
- why: AC7 — the Unicode surface is **notation over the same grammar**, so it
  must not change what elaborates. Asserting **core-term identity** (not a
  verdict) makes it a structural check the D3 policy choice cannot make vacuous.
  Grounds BL3 against `31-lexical` (keywords ASCII, operators Unicode).

---

## PK8 — `proc` covers the polymorphic case, on D1 (frame AC2)

**Authored against Architect's D1 ruling** (`evt_53ybqtzjfv7yx`): the row
variable is a **bare row `[e]`** (`e` a lowercase ident bound as an **implicit
parameter**, like a type/level param, `39`), optionally an **open-row tail
`[E | e]`** (concrete head + poly tail → `RowType::Join(Concrete, Var)`); the
variable **must appear in the declared type** (§3.1 guarantee 1: effects
recoverable from the *type*, so D2's keyword check reads the poly row off the
signature). The `[e]`/`[E | e]` **surface spelling is `(oracle)`**
(proposal-level `OQ-syntax`, reconciled against spec-author's landed §36 `§1.5`
transcription at the merge gate — the same parallel-author-then-reconcile
posture as CAT-1); the **structure** (variable-in-the-type, keyword `proc`,
static closure via `apply_subst`) is **normative** from the ruling.

### surface/declarations/poly-def-is-proc-not-fn (oracle)
- spec: `33 §1`, `36 §1.1`/`§1.2` (latent arrow, `infer_row`), D1 ruling
  (`evt_53ybqtzjfv7yx`; `RowType::Var`), frame §2.1/§2.2/AC2
- given: an effect-polymorphic definition whose declared type carries a **row
  variable** `e`. Two arms, **identical signature**, keyword varied:

  ```
  (a) proc traverse {a b} {e} (f : a -> Eff [e] b) (xs : List a)
        : Eff [e] (List b)  visits [e]
  (b) fn   traverse {a b} {e} (f : a -> Eff [e] b) (xs : List a)
        : Eff [e] (List b)  visits [e]
  ```
- expect: (a) **accepts** — the declared row **contains a variable** `e`
  (`RowType::Var`, D1), which is **decisively `proc`** (frame §2.1: `proc` ⟺ "an
  effect-polymorphic row, contains a variable"). (b) **rejects** — `fn` asserts
  the **closed empty row, no row variable** (frame §2.1); a row *variable* in
  the signature is not `∅`, a false-closure claim (`FnHasRowVariable` /
  false-purity, kind `(oracle)`).
- why: AC2's **core** — the polymorphic case lives **decisively on the `proc`
  side**, the crux that makes the binary split **total**. Verdict **flips** on
  the keyword with the signature held fixed. The targeted bug — a checker that
  reads a row-variable signature as pure (`Var(e)` mistaken for `∅`) — **accepts
  (b)** ⇒ no flip. Disconfirming: the signature is well-formed in both arms
  (same `traverse`), so (b) rejects for the keyword, not a signature error.
  **Gates CAT-2/Traversable** (`traverse` is the first surface consumer of D1).

### surface/declarations/proc-stays-proc-at-pure-instantiation (oracle)
- spec: D1 ruling (`apply_subst(e := ρ)`, `RowType`), `36 §1.2` (λ builds a
  closure, performs nothing; `traverse` never `perform`s `e` itself — splices
  the callback via `bind`), `§2.4`, frame §2.2/§6/AC2
- given: the accepted `proc traverse` (PK8a) **instantiated at a pure callback**
  `pure_f : a -> Eff [] b` — i.e. `e := ∅` by `apply_subst` (D1) — in the call
  `traverse pure_f xs`.
- expect: **dual assertion.** (i) the **call**
  `traverse pure_f xs : Eff [] (List b)` is **statically pure** —
  `apply_subst(e := ∅)` resolves the instantiated row to the **closed-empty**
  set; assert the instantiated call's row **= `∅`** (structural), effect-free,
  runs pure. (ii) **YET** the **definition** `traverse` stays **`proc`** — the
  keyword classifies the abstraction's **guarantee** (frame §2.2), not this
  best-case instantiation; `fn traverse` is **still rejected** (PK8a) even
  though *this* instantiation is pure.
- why: the **exact pure-instantiation round-trip** frame §2.2/AC2 demands + the
  do-not-reopen §6 guard ("do **not** 'optimize' a polymorphic definition into
  `fn` because it *can* be pure"). Two discriminating faces: (i) a checker that
  fails to `apply_subst(e := ∅)` leaves the call's row polymorphic/non-`∅` —
  caught by the asserted `∅`; (ii) a checker that "optimizes" the
  pure-instantiable poly def to `fn` **accepts `fn traverse`** — the exact bug
  §6 forbids, caught by the definition staying `proc`. Effect-polymorphic **≠**
  pure: the guarantee, not the instantiation, sets the keyword.

---

## PK9 — static closure at every instantiation, on D1 (frame AC3)

Every concrete instantiation of a row-polymorphic `proc` resolves to a
**statically-closed** effect set — the D1 ruling makes this **structural**: a
`RowVar` is eliminated only by `apply_subst(e := concrete)` at a call, and "you
cannot run a variable" (a boundary/handler sees a concrete row), so no effect is
discovered at runtime.

### surface/declarations/mis-declared-caller-row-rejected (oracle)
- spec: D1 ruling (`apply_subst` then the `§1.4` escape check on the resolved
  row), `36 §1.4` (escape), frame §2.3/AC3
- given: a caller of `traverse` (PK8a) supplying an **effectful** callback
  `log_f : a -> Eff [Console] b` — so `e := [Console]` by `apply_subst`. Two
  caller declarations, **identical body** `= traverse log_f xs`:
  (a) `proc run_all (xs : List a) : ... visits [Console] = traverse log_f xs`;
  (b) `proc run_all (xs : List a) : ... visits []       = traverse log_f xs`.
- expect: (a) **accepts** — the instantiated
  `traverse log_f xs : Eff [Console] (List b)`; the caller's declared
  `[Console]` contains it (`[Console] ⊆ [Console]`, `§1.4` after `apply_subst`).
  (b) **rejects statically** — the instantiated `[Console]` escapes the caller's
  declared `∅`; a **static** `EffectEscapes(Console)` (kind `(oracle)`), **not**
  a runtime discovery.
- why: AC3 — every concrete instantiation is **statically closed** (frame
  §2.3/AC3, D1: `apply_subst` resolves `e` to a concrete row at the call, then
  the ordinary `§1.4` escape check fires). Verdict **flips** on the caller's
  declared row, body fixed. The targeted bug — a checker that fails to propagate
  `e := [Console]` into the caller's escape check — **admits (b)** ⇒ runtime
  effect discovery ⇒ no flip; the pair pins "no runtime effect discovery."

### surface/declarations/open-row-straddle-rejects-valid (property, oracle)
- spec: D1 ruling + verified myself — `RowType::is_subset_of`'s single-arm
  rule `x ⊆ Join(l,r) = (x⊆l) || (x⊆r)`
  (`crates/ken-elaborator/src/effects/row.rs`, `is_subset_of` impl L200–202, doc
  L183–187; re-derived at authoring, not transcribed), `36 §1.4`
- given: an open-row tail `[E | e]` (`Join(Concrete({E}), Var(e))`) and a
  **concrete** row that **straddles both arms** — part in the concrete head `E`,
  part that must be absorbed by the variable tail: checking `{E, F} ⊆ [E | e]`
  (semantically valid with `e := {F}`).
- expect: **rejected** — the landed single-arm rule needs **all** of `{E, F}` in
  **one** arm: `{E,F} ⊆ {E}` is false and `{E,F} ⊆ Var(e)` is false, so the
  straddle is rejected. This is a **known-completeness marker**: it **rejects a
  VALID program** (fail-closed), and is **NOT a soundness flip** — the
  single-arm rule **never over-accepts** (no effect silently escapes); the
  residual is **rejected-valid**, never accepted-invalid (Architect's D1 note).
- why: pins the fail-closed residual as a **documented completeness limit**, not
  a soundness hole
  ([[kernel-rejects-is-completeness-fix-is-where-soundness-converts]]:
  over-rejection is fail-closed/safe; the soundness risk is only if a *fix*
  loosens `is_subset_of` to over-accept). A future build that **tightens** the
  rule to accept the straddle is a **completeness improvement** (must still
  never over-accept); a regression that makes it **over-accept** is the real
  bug. Not a verdict-flip discriminator — a **one-directional** structural
  marker asserting the rejection **and** its completeness-not-soundness
  classification, so the fix-vector is unambiguous. (`property`; the `row.rs`
  line anchors are perishable — reconcile against the landed impl at the build.)

---

## Regression — SURF-1 is additive over `view`/effects surface

### surface/declarations/additive-over-effects-seed (property)
- spec: `../effects/seed-effects.md` (EFF1 keyword-agnostic row
  inference/escape), `36 §2.4` (pure collapse), frame §2.5 (`view` roles carry
  over)
- given: the EFF1 row-inference/escape invariants and the on-`main` surface
  corpus, after the `view` → `const`/`fn`/`proc` migration.
- expect: **unchanged behavior** — SURF-1 **renames and constrains** the
  definition keyword; it does **not** change row inference (`36 §1.2`), the
  escape check (`§1.4`), or pure-program elaboration. A former pure `view`
  becomes a `fn`/`const` and elaborates to the **identical** core term (`§2.4`);
  a former effectful `view` becomes a `proc` with the **same** inferred/declared
  row. The EFF1 cases (`eff-undeclared-escapes-rejected`,
  `eff-overdeclared-upper-bound-accepted`) still hold under the new keyword.
- why: SURF-1 is **additive** — it adds the keyword↔purity contract **over** the
  landed row machinery, not a replacement. Pins that introducing the split
  perturbs neither pure-program elaboration nor the effect-row discipline EFF1
  guards. (`property`: regression guard over the effects corpus + surface.)
