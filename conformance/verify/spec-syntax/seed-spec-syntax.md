# Verification spec syntax (V1) conformance — seed cases

Format: `../../README.md`. These pin **WS-V V1**: the spec-annotation syntax
(`requires`/`ensures`/refinements/`prove`/`law`), its **elaboration to core**,
and the **verification status model** — the contract elaborated in
`spec/20-verification/21-spec-syntax.md` (landed `wp/V1`). Expected results are
grounded in the **landed** `21`, the V0 elaborator (`ken-elaborator`,
`spec/30-surface/39-elaboration.md §5`), and the `18 §4` certificate API
(`check`)
— not the WP frame's prose (the perishable-frame discipline). The prototype
(`yon`) is not mounted; none of these required it.

**The layer is ★★ (untrusted), so the cases live where the kernel cannot
backstop.** Everything V1 emits is re-checked by the kernel (`18 §4`); a bug is
a
**wrong verdict or a poor diagnostic, never unsoundness**. The cases that matter
are therefore the elaboration **shape** (the emitted core), the **honesty
guard**
(an `unknown` never reads `proved`), and the **scope guards** (`old`, `result`,
Ω-typing) — exactly what a wrong-but-well-typed elaboration could get wrong
while
still kernel-checking.

**Two encoding facts these cases rest on (`21 §2`/§6.3, Architect-confirmed):**

- **Carrier-plus-obligation, not `Σ(B,ψ)`.** An `ensures`/refinement lowers to
  its **carrier** value plus a **separate obligation** (a typed hole =
  postulate, `24 §2`), never a proof-carrying `Σ` value. So the emitted core
  body stays the bare carrier, contracts are erasable, and the encoding is
  **independent of the `Σ`-sort erratum** (`sort_sigma → Ω` iff *both*
  components Ω — a refinement's relevant carrier keeps it at `Type`;
  Architect-confirmed, **in-flight on `wp/V1-sigma-sort`**, not this base —
  `13 §4`/§5 + `../../kernel/pi-sigma/`). `requires φ → Π` proof-arg is sound
  because `sort_pi(Ω, Type) = Type` (codomain-keyed, landed `16 §1.1`).
- **The honesty discriminator is kernel-side.** `unknown ≢ proved` is decidable
  from `GlobalEnv::trusted_base()` (`18 §5`) — a hole **is** a postulate
  (appears), a checked certificate adds **nothing** (absent) — **not** a V-layer
  flag, so the untrusted layer cannot forge `proved` (`21 §5.4`).

**Deferred-tag note (`21 §5.5`, tagged at elaboration time).** V1 delivers the
status **model** and concrete grammar for
`requires`/`ensures`/`{x:A|φ}`/`prove`/
`law` only; the **disposition-tag clause spelling** (`tested`/`assume`/`test`/
`delegated`) stays **reserved** (`OQ-syntax`, downstream of
`../70-behavioral/`).
Cases that would exercise that spelling are tagged **`[deferred — §5.5]`** and
assert the model, never un-landed grammar.

Cases tagged **(soundness)** encode the honesty/scope commitments the whole
model
rests on (`21 §5.4`) and must never regress.

---

## A. Syntax, AST, and elaboration to core (`21 §6.1`–§6.3)

### verify/spec-syntax/requires-elaborates-to-pi-proof-arg
- spec: `21 §6.3` (elabView), `§6.1`/§6.2 (grammar/AST); `16 §1.1` (`sort_pi`)
- given: `view divide (n : Int) (d : Int) : Int requires d ≠ 0 = n / d`
- expect: **accepts**; the emitted core **type** is
  `(n:Int) → (d:Int) → (_ : d ≠ 0) → Int` — the precondition is a **Π
  proof-argument** and the function type stays at **`Type`** (not Ω); the core
  **body** is `λ n. λ d. λ p. (n / d)` — the bare carrier, no proof paired in.
- why: §6.3 lowers `requires φ` to a Π proof-arg, `check`ed at Ω then assumed in
  the body. `sort_pi(Ω, Type) = Type` (`16 §1.1`, codomain-keyed) — an Ω
  **domain** does not collapse the function to a proposition. Structural
  (emitted core), not just accept: a bug dropping the proof-arg, or mis-keying
  `sort_pi` on the **domain** (→ Ω), changes the emitted type — green-vs-red on
  the core shape.

### verify/spec-syntax/ensures-emits-obligation-not-sigma
- spec: `21 §6.3` (postcondition → obligation), `§2` (carrier-plus-obligation),
  `§6.5`
- given: `view abs (n : Int) : Int ensures result ≥ 0 = if n < 0 then -n else n`
- expect: **accepts**; the emitted core **body** has type **`Int`** — the bare
  carrier, **not** `(r:Int) × (r ≥ 0)`; a **single** obligation hole
  `?h : (abs n) ≥ 0` is emitted (goal = `ψ[body/result]`, §6.3, `result`
  replaced by the body).
- why: §6.3/§2 use the **carrier-plus-obligation** encoding — the postcondition
  is a separate obligation (`24 §2`), never paired into the value, so contracts
  are erasable and the encoding is **independent of the `Σ`-sort caveat** (the
  Architect-confirmed erratum in-flight on `wp/V1-sigma-sort` — V1 never forms
  that `Σ`). Structural: a bug reifying `Σ(Int, ≥0)` flips the emitted-body type
  from `Int` to a `Σ`; a bug dropping the obligation empties the hole set.
  Distinct from the prover-level accept (`../seed-verify.md`
  `verify/proved-postcondition`) — this pins the **elaboration shape**.

### verify/spec-syntax/refinement-lowers-to-carrier
- spec: `21 §6.3` (elabType refinement + introduction), `§2`, `§8` (level)
- given: `type Pos = { n : Int | n > 0 }`; the introduction `(5 : Pos)`
- expect: the core type of `Pos` is **`Int`** (the carrier — `φ` not reified),
  at **exactly `Type 0`** (no level bump); the introduction `(5 : Pos)` emits
  obligation `?h : 5 > 0` and yields the core value `5 : Int`.
- why: §6.3 `elabType({x:A|φ})` returns the carrier `A'` (predicate
  elaborator-tracked, not a core type); `A ≤ {x:A|φ}` generates the obligation
  on introduction (no kernel coercion). §8: the refinement stays at the
  carrier's `Type ℓ_A` — the load-bearing choice that V1 **never forms a core
  `Σ` over an Ω predicate**. Structural: emitted type IS `Int` at `Type 0` + one
  obligation; a reification bug emits a `Σ` and may bump the level.

### verify/spec-syntax/non-omega-predicate-surface-error
- spec: `21 §4` (every spec prop `: Ω`), `§6.3` (check at Ω before any
  obligation)
- given: `view f (n : Int) : Int requires (n + 1) = n` — the clause body
  `n + 1 : Int`, **not** a proposition
- expect: **rejected** as a **surface type error** at elaboration
  (`TypeMismatch`: expected `Ω`, found `Int`), **before** any obligation is
  formed; **not** a verification failure, **not** silently admitted.
- why: §6.3 `check`s every clause body at Ω; a non-Ω body is rejected at
  elaboration (`§4`, "a load-bearing guard"). **Absence-assertion gate** — guard
  named: the explicit `check(Γ, φ, Ω)` in `elabView`/`elabType`. **Disconfirming
  check:** would a non-Ω `requires` also reject under the bug this targets
  (omitting the Ω-check)? **No** — that bug would *accept* it and form an
  obligation over a non-proposition. **Verdict-flips:** `requires (n > 0)`
  (`: Ω`) accepts; `requires (n + 1)` rejects — pins the Ω-gate, not a
  coincidental reject.

### verify/spec-syntax/result-scope
- spec: `21 §4` (`result` only in `ensures`), `§6.3`
- given: (a) `view g (n:Int) : Int ensures result ≥ 0 = …`; (b)
  `view h (n:Int) : Int requires result ≥ 0 = …`
- expect: (a) **accepts** — `result` resolves to the return value (`: Int`) in
  the `ensures` scope; (b) **rejected** — `result` in a `requires` is an
  **unbound-name / scope error** (`result` not in scope outside `ensures`).
- why: §4 binds `result` only in `ensures` clauses. **Verdict-flips on scope:**
  the same identifier resolves in (a), is unbound in (b). **Absence-assertion:**
  guard = the binder scope (resolution adds `result` only when elaborating an
  `ensures`). Disconfirming: a bug scoping `result` globally would *accept* (b)
  — so the reject is guard-gated, not coincidental.

---

## B. `old`-capture scope guard (`21 §6.4`) — the flip pair

### verify/spec-syntax/old-resolves-in-space-op-ensures
- spec: `21 §6.4` (elabSpaceEnsures), `36 §4.3`; `16 §2` (`refl`)
- given: a `space` op `inc` over a cell `n : Int` with `ensures n == old(n) + 1`
- expect: **accepts**; `old(n)` resolves to the **pre-state** `s_pre.n`
  (`§6.4`), the bare `n` to the post-state `s_post.n`; the obligation
  `(s_pre with .n := s_pre.n + 1).n == s_pre.n + 1` computes by record-β
  (`13 §3`) to `s_pre.n + 1 == s_pre.n + 1`, discharged by `refl`.
- why: §6.4 binds `s_pre, result, s_post` in a `space`-op `ensures`; `old(e)`
  denotes `⟦e⟧` at `s_pre`. Structural: `old(n)` resolves to the `s_pre`
  projection (not `s_post`). Worked example from `36 §4.3`. Pairs with the
  reject below — the flip is the §6.4 scope guard.

### verify/spec-syntax/old-out-of-scope-rejects (soundness)
- spec: `21 §6.4` (scope guard), `§4`, `36 §7.3`
- given: a **pure**
  `view k (x : Int) : Int ensures result == old(x) + 1 = x + 1`
- expect: **rejected** — `old(x)` in a pure `view`'s `ensures` is a **scope
  error** at elaboration (no `State` effect ⇒ no distinct pre-state `s_pre` to
  bind), before the kernel.
- why: §6.4 admits `old` **only** when the enclosing declaration is a `space`
  operation — the guard is the **kind of the enclosing declaration**, asserted
  explicitly. **Verdict-flips** with the case above: identical `old(…)` syntax,
  **space-op resolves / pure-view rejects**. **Absence-assertion gate** — guard
  named: enclosing-decl-kind = `space` op (the one place `s_pre ≢ s_post`).
  **Disconfirming check:** would `old(x)` in a pure view reject under the bug
  this targets (dropping the scope guard)? **No** — that bug would bind `old(x)`
  to `s_post ≡ s_pre` and **accept** it (silently meaningless) — so the reject
  is guard-gated, not coincidental.

---

## C. The verification status model + honesty guard (`21 §5`)

### verify/spec-syntax/proved-status-cert-checks-not-in-trusted-base (soundness)
- spec: `21 §5.1` (verdict `proved`), `§5.4` (discriminator); `18 §4.5`/§5
- given: an `ensures` obligation `?h : φ` discharged by a certificate `p` with
  `Γ ⊢ p : φ` (a closed core proof term)
- expect: verdict **`proved`** — `check(env, Γ, p, φ)` **accepts**, **and** the
  goal `φ` does **not** appear in `GlobalEnv::trusted_base()` (the postulate is
  retired on discharge).
- why: §5.1/§5.4 — a `proved` certificate is a closed core term `check`
  validates; it adds **nothing** to the trusted base (`18 §5`). The `proved`
  side of the kernel-side discriminator. (The cert is given, not
  prover-generated — V1 fixes the **status model**; the prover is V3.)

### verify/spec-syntax/bogus-cert-not-proved (soundness)
- spec: `21 §5.1`, `§5.4`; `18 §4.5` (de Bruijn re-check); `../seed-verify.md`
  `verify/wrong-proof-rejected`
- given: the same goal `φ` and a **wrong** certificate `c` with `Γ ⊬ c : φ`
- expect: `check(env, Γ, c, φ)` **rejects** ⇒ verdict is **not `proved`** — the
  obligation stays an open hole (`unknown`), `φ` **remains** in
  `trusted_base()`.
- why: the kernel re-check is the soundness firewall around the untrusted prover
  (`18 §4`) — a wrong certificate cannot make `φ` `proved`. **Verdict-flips**
  with the case above: valid cert → `proved` + `φ ∉ trusted_base()`; bogus cert
  → not-proved + `φ ∈ trusted_base()`. The de Bruijn criterion at the
  status-model level.

### verify/spec-syntax/unknown-hole-distinct-from-proved (soundness)
- spec: `21 §5.4` (the honesty guard), `§5.1` (`unknown` = postulate); `18 §5`;
  `24 §2`
- given: a `view` with an `ensures φ` left **undischarged** — admitted with a
  typed hole `?h : φ`; the program type-checks and runs
- expect: the goal `φ` **appears** in `GlobalEnv::trusted_base()` (the hole
  **is** a postulate of `φ`), whereas a `proved` claim's goal does **not**. So
  `unknown ≢ proved` is decidable **kernel-side**: a claim is `proved` **iff**
  its certificate `check`s **and** no postulate carrying its goal sits in
  `trusted_base()`.
- why: §5.4 — the discriminator is **structural and kernel-side**, not a V-layer
  flag, so a bug in the untrusted layer **cannot forge** `proved`.
  **Absence-assertion gate** — guard named: **`trusted_base()` membership**
  (`18 §5` enumerates exactly postulates + primitives). **Disconfirming check:**
  would an `unknown` claim read `proved` under the bug this targets (conflating
  the statuses)? **Only** if the discriminator were a self-reported flag;
  because it is the kernel's own `trusted_base()` membership, the hole-postulate
  is **structurally present** — green-vs-red on membership, never
  green-vs-green. The load-bearing ★★ honesty property: an `incomplete` never
  masquerades as `proved`. (Pins `21 §9` acceptance #2; grounds the same
  `trusted_base()` enumerated by `../../kernel/judgments/seed-judgments.md`
  `judgments/trusted-base-enumerate`.)

### verify/spec-syntax/disproved-distinct-from-unknown
- spec: `21 §5.1` (`disproved` = verification error), `§5.3` (projection),
  `24 §3`
- given: `view f (n : Int) : Int ensures result > 0 = n` — false for `n ≤ 0`,
  with a **given** countermodel at the `n ≤ 0` world (the prover is V3 — the
  countermodel is supplied here, as the certificate is in the
  `proved`/bogus-cert cases)
- expect: verdict **`disproved`** (carried evidence: the countermodel) — a hard
  **verification error** (`24 §3`) with **no epistemic status** (it is fixed,
  not exported); **distinct** from `unknown`: a `disproved` claim is an error to
  fix, an `unknown` claim leaves the program **running** with a hole.
- why: §5.3 — the projection maps `disproved` → *none* (a refuted claim is never
  shipped) and `unknown` → `unknown` (the program runs). The verdict trichotomy
  (`proved`/`disproved`/`unknown`) is **not** collapsible: `disproved ≠ unknown`
  (both "not proved", but one is an error, the other a running hole) —
  discriminates on the trichotomy, not merely "not proved".

---

## D. Epistemic projection + deferred disposition tags (`21 §5.2`/§5.3/§5.5)

### verify/spec-syntax/epistemic-projection-distinct
- spec: `21 §5.2`/§5.3 (the projection), `§5.5` (deferred tag grammar)
- given: the four epistemic statuses of `21 §5.2`, each by carried evidence
- expect (model, landed): the projection (`§5.3`) pins them **distinct** —
  prove+`proved` → **proved** (certificate, kernel-checked, `∉ trusted_base()`);
  prove+`unknown` → **unknown** (typed hole = postulate, `∈ trusted_base()`);
  `test`/`assume` → **tested** (runtime/test + generator obligation); `delegate`
  → **delegated** (temporal-logic export, model-checking obligation). The
  verifier **never returns** `tested` or `delegated` — they are author-chosen
  annotations that **bypass the prover** (`§5.3`).
- expect (deferred): the `proved` vs `unknown` distinction is pinned
  **structurally** here and now (via `trusted_base()`, group C); the
  **`tested`/`delegated` clause spelling** is **`[deferred — §5.5, OQ-syntax]`**
  (`assume`/`test` reserved, grammar downstream of `../70-behavioral/`) — this
  case asserts the **model**, not un-landed grammar.
- why: §5.3/§5.5 — V1 delivers the status **model** (four meanings + projection
  + honesty guard) and concrete grammar only for the proof-disposition forms;
  the disposition-tag spelling is reserved. Tagging the deferred sub-part keeps
  the case from asserting grammar that does not yet exist.

---

## E. The V1→V2 interface (`21 §7`)

### verify/spec-syntax/obligation-hole-set-exposed-to-v2
- spec: `21 §7` (the interface), `§6.5` (obligation-hole encoding)
- given: `view f (n : Int) (m : {k:Int|k≠0}) : Int` with `requires n > 0`, two
  postconditions `ensures result ≥ n` and `ensures result ≥ 0`, and body `n + m`
- expect: the elaborated form exposes exactly the four-part V1→V2 interface: (1)
  the kernel-checkable core term (precondition Π proof-arg, carrier result,
  refined-param `m` lowered to `Int`); (2) the **obligation-hole set** — exactly
  **2** holes here, one per `ensures`, each `⟨id, Γ ⊢ φ, provenance⟩` (a refined
  **parameter** is **not** a definition-site hole — its `m ≠ 0` is a
  Γ-hypothesis per part (3), and the obligation to *establish* it is the
  **caller's**, at the call site, `22 §2.3`/§3); (3) each hole's `Γ` carries the
  **at-introduction hypotheses** — the precondition `n > 0` and the
  refined-param fact `m ≠ 0`; (4) provenance (source span + responsible clause)
  per hole.
- why: §7 — V1 hands V2 a kernel-checked elaborated form with obligation sites
  marked; it does **not** generate VCs or walk the body (that is V2).
  Structural: the obligation-hole **count** and each hole's `Γ`-contents are the
  asserted output; a bug dropping a hole or losing a precondition from `Γ` flips
  the structure. Pins `21 §9` acceptance #4 (V2-ready).

---

## F. Goals — `prove` / `law` (`21 §3`, §6.3)

### verify/spec-syntax/prove-goal-obligation-and-postulate-binding (soundness)
- spec: `21 §3` (`prove`), `§6.3` (elabProve), `§5.4`; `11 §4`
- given: `prove add_comm : (a b : Int) → a + b == b + a`, before and after
  discharge
- expect: **before discharge** — a standalone obligation hole `?h : φ` is
  emitted; `add_comm` is bound as a **postulate** of `φ` (usable as a proof
  term), and `φ` **appears** in `trusted_base()` (status `unknown`). **After
  discharge** by a certificate `p` with `Γ ⊢ p : φ` that `check`s — the
  postulate is **retired**, `add_comm ↦ p`, status `proved`, `φ` **leaves**
  `trusted_base()`.
- why: §6.3/§5.4 — `prove` is the degenerate (no-`view`) obligation; proving is
  hole-filling, and the honesty guard applies identically (a goal is a visible
  postulate until its certificate checks). Verdict/structural flip on
  **`trusted_base()` membership** across discharge.

### verify/spec-syntax/law-all-omega-fields-is-proposition
- spec: `21 §3` (`law`), `§8`; `16 §1.3` (conjunction in Ω)
- given: `law Monoid (M) { assoc : … ; unit_l : … ; unit_r : … }` with each
  field a proposition (`: Ω`)
- expect: **accepts**; each field `check`s at Ω, and the all-Ω bundle is a
  **conjunction** landing in **Ω** (the bundle is itself a proposition) — the
  **sound** both-components-Ω case (landed `16 §1.3`).
- why: §8 — a `law` of all-Ω fields is `Σ`-of-Ω-into-Ω, which lands in Ω
  **correctly** (both halves proof-irrelevant, `16 §1.3`).
  **Internal-consistency tie:** this is the *legitimate* Ω-landing `Σ`, in
  deliberate contrast to a **refinement** `{x:A|φ}` (relevant first component)
  which must stay at `Type` (the in-flight `Σ`-sort erratum, `wp/V1-sigma-sort`)
  — together they pin **both directions** of the both-components-keyed
  `sort_sigma`, and must not contradict (a refinement is **not** a proposition;
  a law **is**). The refinement direction is pinned by `../../kernel/pi-sigma/`
  (the erratum's conformance third), not this file.

---

## G. Regression — V0 unchanged (`21 §6.2`)

### verify/spec-syntax/v0-unchanged-for-non-spec-programs (soundness)
- spec: `21 §6.2` (no regression), `§9` acceptance #5; `39 §5` (V0)
- given: a non-spec program, e.g. `view id (A : Type) (x : A) : A = x` — no
  `requires`/`ensures`/refinement/`prove`/`law`
- expect: parses to **exactly** the V0 AST (the `ViewDecl`'s
  `requires`/`ensures` lists empty, no `TRefine`/`ProveDecl`/`LawDecl`),
  elaborates identically to V0, and the kernel-checked core term is
  **unchanged** from V0.
- why: §6.2 — V1 adds spec-carrying fields and new variants, but a spec-free
  program exercises **none** of them; its elaboration must be byte-identical to
  V0 (`39 §5`). Regression guard: any V1 change that altered non-spec
  elaboration flips this. Mirrors `../../kernel/judgments/seed-judgments.md`
  `judgments/k1-k2-judgments-still-green`.

---

## Coverage map (acceptance, `21 §9`)

- **#1 syntax + elaboration to core** — `requires-elaborates-to-pi-proof-arg`,
  `ensures-emits-obligation-not-sigma`, `refinement-lowers-to-carrier`,
  `non-omega-predicate-surface-error`, `result-scope`.
- **#2 four-way status honest** —
  `proved-status-cert-checks-not-in-trusted-base`, `bogus-cert-not-proved`,
  `unknown-hole-distinct-from-proved`, `disproved-distinct-from-unknown`,
  `epistemic-projection-distinct`.
- **#3 `old` semantics** — `old-resolves-in-space-op-ensures` /
  `old-out-of-scope-rejects` (the flip pair).
- **#4 V2-ready** — `obligation-hole-set-exposed-to-v2`.
- **#5 no regression** — `v0-unchanged-for-non-spec-programs`.
- **goals** — `prove-goal-obligation-and-postulate-binding`,
  `law-all-omega-fields-is-proposition`.

Build-sequencing: V1 extends the **landed** V0 elaborator and reuses kernel
`Term`
constructors V0 never emits (`Pi`/`Sigma`/`Pair`/`Proj`/`Omega`/`Eq`, none
`[K2]`-reserved) and the `18 §4`/§5 certificate API. The `Σ`-sort caveat (§2) is
**not** on V1's path (carrier-plus-obligation never forms `Σ` over an Ω
predicate);
the `Σ`-sort discriminating guard rides the dedicated erratum branch
`wp/V1-sigma-sort` (spec `13 §4`/§5 + the `../../kernel/pi-sigma/` conformance +
kernel-leader's `sort_sigma` split, landing together per the Architect's 3-piece
gate), not this file.
