# Obligation generation (V2) conformance — seed cases

Format: `../../README.md`. These pin **WS-V V2** — the verification-condition
extractor: turning a V1-spec'd program into the **obligation set** (each a
triple
`⟨id, Γ ⊢ φ, provenance⟩`). Grounded in the **landed** `22-obligations.md`
(`wp/V2`), V1's `21 §6`/§7 (the carrier-plus-obligation form V2 consumes), the
`18 §4`/§5 cert API, the kernel eliminator (`14 §3`) and `match → elim_D`
(`39 §2.6`), and first principles. The prototype is not mounted; none of these
required it. (Partial-primitive grounding `35 §3`/`43 §2` confirmed landed —
not a forward reference.)

**The layer is ★★ (untrusted) — but read the backstop precisely (Architect).** A
V2 bug never breaks **kernel** soundness: the kernel re-checks every *supplied*
certificate (`18 §4`), so a **spurious** obligation is over-conservatism (a
false
`unknown`) and a *bad* cert is kernel-rejected. But a **missed** obligation — a
burden the extractor **never emits** — is **not** caught downstream: the V1 §5.4
honesty guard (`trusted_base()`) catches **generated-but-undischarged** holes,
not
sites that were never turned into holes, so a never-emitted burden reads as
`proved` though unproven. **Completeness-of-extraction is therefore the
*verification*-soundness linchpin, backstopped by nothing but the absent-clause
scan (§2.5)** — "all obligations discharged ⇒ correct" is only as strong as the
guarantee that **no burden was silently skipped**. So these cases pin
**completeness of extraction** (every burden site → its obligation; the
absent-clause scan audits that no burden is silently skipped *and* no trivial
clause over-skipped — the load-bearing safeguard, not a nicety) and **honest
provenance** (each obligation traces to its source clause + has a stable id).

**Decoupled from the Σ-sort erratum (`22 §1.1`).** V2 reads V1's
**bare-carrier + separate-obligation** form — never a proof-carrying `Σ(B,ψ)`
value (V1 emits none, `21 §2`/§6.3) — so it never forms or depends on a core `Σ`
over an Ω predicate. The `sort_sigma` erratum (`13 §4`, on `wp/V1-sigma-sort`)
is
off V2's path.

**Two-sided completeness (the spine of these cases).** The extractor must
**emit**
at every burden site (§2.1–§2.4) **and** must not emit where there is no burden
—
but each no-emit position is **explicitly guarded** (§2.5), so a *missing* guard
(a silently-dropped clause) is detectable. Every no-emit case below names its
**guard** and passes the **disconfirming check** ("would an obligation wrongly
appear / vanish here under the targeted bug?").

Cases tagged **(soundness)** encode the completeness/honesty commitments the
verdict model rests on (`22 §2.5`, `21 §5.4`) and must never regress.

---

## A. Extraction completeness — the four sources (`22 §2`)

### verify/obligations/refinement-introduction-emits-phi
- spec: `22 §2.1`; `21 §2` (carrier encoding)
- given: `type Pos = { n : Int | n > 0 }`; the introduction `(5 : Pos)` (a value
  `5 : Int` used where `Pos` is expected)
- expect: emits **one** obligation `⟨id, Γ ⊢ 5 > 0, prov⟩` — the goal is
  `φ[a/x] = (n > 0)[5/n] = 5 > 0`, at Ω; provenance points to the introduction
  site.
- why: §2.1 — the introduction direction `A ≤ {x:A|φ}` emits `φ[a/x]`.
  Structural: the goal is the **substituted** `5 > 0` (not the un-substituted
  `n > 0`, not nothing). A bug emitting `n > 0` (no subst), or none, flips the
  structure. The reverse forgetful direction emits nothing
  (`forgetful-coercion-emits-nothing`).

### verify/obligations/postcondition-emits-substituted-goal
- spec: `22 §2.2`; `21 §6.3`
- given: `view inc (n : Int) : Int ensures result > n = n + 1` — a
  **straight-line** body
- expect: emits **one** obligation `⟨id, Γ,(n:Int) ⊢ (n + 1) > n, prov⟩` —
  `ψ[b/result]` with `result` replaced by the straight-line body `b = n + 1`; at
  Ω; `Γ` carries the parameter telescope.
- why: §2.2 — for a **straight-line** body the postcondition is the single
  substituted goal `ψ[b/result]` (the refined-result-type motive degenerates to
  one). Structural: the goal mentions the **body** (`result` substituted), not a
  free `result`; a bug leaving `result` free, or omitting the obligation, flips
  it. **A branchy/recursive body does *not* yield a single over-the-body
  obligation** — it pushes the motive through **per path / per constructor**
  (`§3`/§4; cases `conditional-branch-adds-boolean-equation`,
  `recursive-fn-per-ctor-obligation-with-ih`, the flagship), which carry the
  path-conditions and the induction hypothesis. This illustration uses a
  straight-line body deliberately: a single over-the-body obligation carries
  **no** IH and cannot verify a recursive function (§2.2/§4) — the
  internal-consistency alignment with `C`/`D1`.

### verify/obligations/precondition-obligation-at-call-not-in-body (soundness)
- spec: `22 §2.3`, `§2.5.2`
- given: `view safe_div (n : Int) (d : Int) : Int requires d ≠ 0 = n / d`, and a
  caller `view use (x : Int) : Int = safe_div x 2`
- expect: the precondition `d ≠ 0` yields an obligation **at the call** in `use`
  — `⟨id, Γ_call ⊢ 2 ≠ 0, prov(call)⟩` (the caller meets it); **inside**
  `safe_div`'s body it yields **no** obligation — `d ≠ 0` is an **assumption**
  in `Γ`.
- why: §2.3 — the precondition is the **caller's** burden, discharged at the
  call; the callee assumes it. **Verdict/structural flip on placement:** the
  obligation appears at the call (`2 ≠ 0`) and is **absent** (present as a
  Γ-assumption) in the body. **Absence-assertion (no body obligation)** — guard:
  a precondition enters `Γ` at the body top (§3), never the function's own goal.
  **Disconfirming:** would the body also carry no `d ≠ 0` obligation under the
  bug that re-obligates the precondition? **No** — that bug emits a **spurious**
  body goal the callee cannot discharge (it has no proof its own argument is
  nonzero). The asymmetry **is** the contract.

### verify/obligations/partial-primitive-emits-nonzero-obligation
- spec: `22 §2.4`; `35 §3` (Int div/mod by zero = obligation); `43 §2`
- given: an unrefined division `n / d` on `Int` with a **possibly-zero** `d` (no
  `{d|d≠0}` refinement, no `d ≠ 0` in scope)
- expect: emits a **non-zero** side-condition obligation
  `⟨id, Γ ⊢ d ≠ 0, prov(op)⟩` at the operation site; a divisor **already**
  refined `{d|d≠0}` (or with `d ≠ 0 ∈ Γ`) emits **no** new obligation
  (discharged by the refinement / assumption).
- why: §2.4 — a partial primitive (`/`, `%` on `Int`) emits its side condition
  (`35 §3`: "possibly-zero is an obligation, not a silent trap";
  cross-referenced from `43 §2`). **Verdict-flips:** possibly-zero divisor →
  non-zero obligation; refined/assumed-nonzero divisor → no new obligation.
  (`Int` is arbitrary-precision, so the partial case is **div-by-zero**, not
  overflow; fixed-width overflow is the analogous side-condition for sized
  types.)

### verify/obligations/prove-and-law-emit-one-obligation-per-goal
- spec: `22 §2.4` (degenerate `prove`/`law`); `21 §3`
- given: (a) `prove add_comm : (a b : Int) → a + b == b + a`; (b)
  `law Monoid (M) { assoc : … ; unit_l : … ; unit_r : … }`
- expect: (a) **one** obligation `⟨id, Γ_binders ⊢ a + b == b + a, prov⟩` (no
  body — the degenerate case); (b) **one obligation per field** (3 here).
- why: §2.4 — `prove` is the degenerate (bodyless) obligation; `law` emits one
  per field. Structural: the obligation **count** (1 / 3) + each prop. A bug
  merging the `law` fields into one obligation, or dropping the bodyless
  `prove`, flips the count.

---

## B. The absent-clause scan — guarded no-emit + the counter-rule (`22 §2.5`)

### verify/obligations/refined-param-is-hypothesis-not-obligation (soundness)
- spec: `22 §2.5.1`, `§3`
- given: `view head (xs : { l : List A | l ≠ nil }) : A = …` — a refined
  **parameter**
- expect: the refined parameter emits **no** definition-site obligation; it
  contributes `(_ : xs ≠ nil)` to `Γ` (a hypothesis the body may use); the
  obligation to establish `xs ≠ nil` is the **caller's** at each call (§2.3).
- why: §2.5.1 — a refined parameter is a **binder**, not a use/introduction.
  **Absence-assertion gate** — guard: the position is a binder (→ Γ-hypothesis),
  not an introduction. **Disconfirming:** would `head` carry no `xs ≠ nil`
  obligation under the bug that treats the binder as an introduction? **No** —
  that bug emits a **spurious** obligation `head` cannot discharge (it has no
  proof its own argument is non-nil). Structural flip: correct → `xs ≠ nil ∈ Γ`,
  no def-site obligation; bug → spurious def-site obligation. (The V1→V2
  distinction; `../spec-syntax/seed-spec-syntax.md`
  `obligation-hole-set-exposed-to-v2` pins the same on the V1 side.)

### verify/obligations/body-requires-assumed-not-reobligated
- spec: `22 §2.5.2`, `§3`
- given: inside `safe_div`'s body (above), the precondition `requires d ≠ 0`
- expect: `d ≠ 0` is **assumed** — it enters `Γ` at the top of the body — and is
  **not** re-emitted as an obligation of `safe_div`.
- why: §2.5.2 — a precondition, once inside the body, is an assumption (the
  caller owes it, §2.3). **Absence-assertion** — guard: a precondition enters
  `Γ` at the body top, never the function's own goal. **Disconfirming:** would
  the body lack a `d ≠ 0` obligation under the bug that re-obligates it? **No**
  — that bug emits a spurious unprovable body goal. Pairs with
  `precondition-obligation-at-call-not-in-body` (the two halves of one
  asymmetry).

### verify/obligations/present-cert-yields-zero-new-obligations (soundness)
- spec: `22 §2.5.3`; `21 §5.4` (discharged hole leaves `trusted_base()`);
  `18 §4.5`
- given: `prove p : φ := <a valid certificate term>` — a `prove` whose
  discharging term is already supplied and `check`s
- expect: **zero new** obligations — the certificate **is** the discharge; the
  hole is **retired** (not re-emitted), and `φ` does **not** appear in
  `trusted_base()`.
- why: §2.5.3 — a site whose certificate is already present yields no open
  obligation (proving = hole-filling, and this hole is filled).
  **Absence-assertion** — guard: a discharged hole is not in the open set
  (`21 §5.4` — the goal leaves `trusted_base()`). **Disconfirming:** would an
  **un**discharged `prove φ` also yield zero obligations? **No** — without a
  cert it emits the open obligation and `φ` **stays** in `trusted_base()` (the
  `prove-and-law` case). Zero-vs-one obligations flips on cert-presence. (Ties
  the honesty guard `21 §5.4` to extraction.)

### verify/obligations/forgetful-coercion-emits-nothing
- spec: `22 §2.5.4`, `§2.1`
- given: a value `p : Pos` (`= {n:Int|n>0}`) used where plain `Int` is expected
  — the forgetful direction `{x:A|φ} ≤ A`
- expect: **no** obligation — the coercion forgets the proof and is the
  **identity** on the carrier `Int` (V1's encoding).
- why: §2.5.4/§2.1 — `{x:A|φ} ≤ A` is **free**. **Absence-assertion** — guard:
  the `≤`-direction is carrier-**forgetting**, not refinement-**introducing**.
  **Disconfirming:** would the **introduction** direction (`Int` used as `Pos`)
  also emit nothing? **No** — that direction emits `φ[a]`
  (`refinement-introduction-emits-phi`). The two directions flip: introduce →
  obligation; forget → none.

### verify/obligations/trivial-clause-still-emits-obligation (soundness)
- spec: `22 §2.5` (the completeness counter-rule); acceptance `§8` / frame `§1`
- given: (a) `view f (n : Int) : Int ensures result == result = n` (a
  **trivially-true** postcondition); (b)
  `view g (n : Int) : Int ensures result ≥ 0 = n * n` (a **real-burden**
  postcondition) — both **straight-line** bodies (one obligation each, isolating
  the trivial-vs-real axis from path-sensitivity)
- expect: **both** emit a postcondition obligation — (a)
  `⟨id, Γ ⊢ b_f == b_f, prov⟩` (provable, discharged trivially by `refl`); (b)
  `⟨id, Γ ⊢ b_g ≥ 0, prov⟩` (a real burden). **Neither** yields *no* obligation.
- why: §2.5 counter-rule — emission is keyed on the **clause's presence**, never
  a triviality heuristic; a trivially-true clause yields its **provable**
  obligation, not no obligation. **The absent-clause discriminating property:**
  a real-burden and a trivial clause both **emit** distinct obligations.
  **Disconfirming:** would (a) emit no obligation under a "skip the
  obviously-true ones" optimization? **Yes** — and that *is* the bug: a clause
  mis-judged trivial is silently dropped, reading as "verified." Green
  (obligation emitted, discharged by `refl`) vs red (no obligation — a missed
  check), on emission keyed to clause presence. The load-bearing completeness
  audit.

### verify/obligations/exhaustive-traversal-no-silent-skip (soundness)
- spec: `22 §2.5` (the exhaustiveness property, Architect-required), `§5`
- given: a core construct at a position the extractor has **no explicit rule
  for** — neither an emit site (§2.1–§2.4) nor an explicit Γ-extension (§3) nor
  an explicitly-guarded no-emit (§2.5) — e.g. a future burden-bearing core form
  added without an extraction rule
- expect: the extractor surfaces a **visible gap** — an **error** (or a
  conservative emit) — **never** a silent recurse-past. The traversal is
  **exhaustive by construction**: every core form is an emit site, an explicit
  Γ-extension, or an explicitly-guarded no-emit; there is **no** catch-all
  `_ ⇒ skip`.
- why: completeness-of-extraction is the **sole** backstop of verification
  soundness (preamble) — a missed obligation is *not* caught downstream — so the
  one thing that must never happen is a burden silently no-emitted. **Structural
  / absence assertion (the guard is exhaustiveness by construction, not a value
  flip):** adding a burden-bearing core form **without** an emit rule must be a
  **compile/visible failure**, not a silent miss. This is not a value-verdict
  (no program exhibits it today — the core `Term` set is fixed); it is a
  property of the **extractor's shape** — assert that the traversal `match` has
  **no catch-all `_ ⇒ skip`** (which would silently swallow a future variant),
  so an unmatched construct is emit-or-error. **Disconfirming:** would a future
  burden-bearing variant be silently skipped under a catch-all skip? **Yes** —
  that is exactly the bug this forbids; the property is green (visible gap /
  error) vs red (silent vanish). The normative form of §2.5's "every no-emit is
  an **explicit guarded skip**, so a missing clause is a **visible gap, not a
  silent drop**."

---

## C. Path-sensitive hypothesis accumulation — the context Γ (`22 §3`)

### verify/obligations/match-branch-gamma-carries-scrutinee-equation (soundness)
- spec: `22 §3` (match constructor equation), `§4`; `39 §2.6` (match→elim_D)
- given: `view f (xs : List Int) : Int ensures P result = …` whose body is
  `match xs { nil → e0 ; cons y ys → e1 }` where the `cons`-branch goal
  discharges only by knowing the scrutinee shape
- expect: in the `cons y ys` branch, the obligation's `Γ` carries the
  **scrutinee equation** `(_ : Eq (List Int) xs (cons y ys))` and binds the
  fields `y, ys`; the `nil` branch carries `(_ : Eq (List Int) xs nil)`.
- why: §3 — a case split adds, per branch, the constructor equation identifying
  the scrutinee ("in the `nil` branch you may assume `xs ≡ nil`").
  **Structural/verdict flip:** an obligation needing `xs ≡ cons y ys` is
  provable **with** the equation in `Γ`, unprovable **without** it.
  **Disconfirming:** a bug dropping the scrutinee equation yields a too-weak `Γ`
  → a **false `unknown`** (§3's visible failure mode) — green (provable) vs red
  (false unknown) on the equation's presence.

### verify/obligations/let-binding-adds-equation-to-gamma
- spec: `22 §3` (let-equation)
- given: a body `let m := n + 1 in …` with a downstream obligation that
  discharges only via `m == n + 1`
- expect: the obligation's `Γ` carries `(m : Int)` and the equation
  `(_ : Eq Int m (n + 1))`.
- why: §3 — `let x := e` adds `x` and (where informative) the equation
  `Eq A x e`, so later obligations may rewrite by the binding. Structural flip:
  the obligation needing `m == n + 1` is provable with the let-equation in `Γ`,
  unprovable without it (too-weak `Γ` → false unknown).

### verify/obligations/conditional-branch-adds-boolean-equation
- spec: `22 §3` (conditional)
- given: `view f (n : Int) : Int ensures result ≥ 0 = if n ≥ 0 then n else 0`
- expect: the `then`-branch obligation's `Γ` carries
  `(_ : Eq Bool (n ≥ 0) true)`; the `else` branch carries
  `(_ : Eq Bool (n ≥ 0) false)`.
- why: §3 — `if c` adds `Eq Bool c true` / `false` per branch (elaborated
  `elim_Bool`). Structural flip: the `then` obligation `n ≥ 0` is provable from
  the branch equation in `Γ`; without it, a false unknown.

---

## D. Body-as-motive — induction surfaced from `elim_D` (`22 §4`)

### verify/obligations/recursive-fn-per-ctor-obligation-with-ih (soundness)
- spec: `22 §4`; `14 §3` (eliminator); `39 §2.6`
- given: a recursive `view sum (xs : List Nat) : Nat ensures result ≥ 0 = …`
  with body `match xs { nil → 0 ; cons y ys → y + sum ys }`
- expect: the extractor emits **per-constructor** obligations from the `elim_D`
  motive `M z = { r : Nat | r ≥ 0 }` — the `nil` branch obligation `0 ≥ 0`; the
  `cons y ys` branch obligation `(y + sum ys) ≥ 0` **with the induction
  hypothesis** `(_ : M ys) = (_ : sum ys ≥ 0)` in `Γ`.
- why: §4 — the dependent eliminator gives each constructor method the IH for
  its recursive fields (`M zᵢ`); V2 **reads** these from the elaborator's
  `match → elim_D` compilation and adds them to `Γ` (it does not synthesize an
  induction principle). **Structural/verdict flip:** the `cons` step is provable
  **with** `sum ys ≥ 0` in `Γ` (structural induction), unprovable **without**
  it. **Disconfirming:** would the inductive step discharge under a bug that
  drops the IH? **No** — without `M ys` the step is a false `unknown`.
  Structural induction, surfaced automatically.

### verify/obligations/nonrecursive-degenerate-no-induction-hypothesis
- spec: `22 §4` (degenerate motive)
- given: a non-recursive
  `view double (n : Nat) : Nat ensures result ≥ n = n + n`
- expect: the obligation `(n + n) ≥ n` is emitted with **no** induction
  hypothesis in `Γ` (the degenerate motive — no recursive fields).
- why: §4 — non-recursive functions are the degenerate motive (no recursive
  fields ⇒ no IH); the same machinery covers both, no special-casing.
  **Internal- consistency tie:** with `recursive-fn-…-with-IH`, this pins that
  an IH appears **iff** there is a recursive field — a bug adding a spurious IH
  here (or dropping the real one there) flips one of the pair.

---

## E. The acceptance flagship (`22 §8`)

### verify/obligations/inductive-postcond-hole-localization (soundness)
- spec: `22 §8` (acceptance), `§5`, `§6`; `18 §4.5`; `21 §5.4`
- given: the recursive `sum` (above) with its per-constructor obligations
  **supplied with valid proofs**; then the **same** with the `cons`-branch proof
  **removed**
- expect: (a) with all proofs — every obligation's certificate `check`s in the
  kernel (`18 §4.5`); the definition is **fully verified** (empty open set;
  nothing from it in `trusted_base()`). (b) with the `cons` proof removed — a
  **single, precisely-located** open hole at the `cons`-branch obligation
  `(y + sum ys) ≥ 0` (status `unknown`); its goal **appears** in
  `trusted_base()`; the other obligations stay `proved`.
- why: §8 — the flagship end-to-end. **Structural/verdict flip:** all-proofs →
  fully verified; one-proof-removed → exactly **one** localized hole (its
  `Γ ⊢ φ` identity + provenance pin *which*), the rest unaffected.
  **Disconfirming:** does removing a needed proof leave a vague/global failure,
  or a precisely-located hole? §8 requires the latter — the hole carries its
  `Γ ⊢ φ` and provenance, and only that obligation flips to `unknown`. Pins the
  honesty guard (`21 §5.4`) end-to-end at the obligation grain.

---

## F. Interface, provenance, and regression

### verify/obligations/provenance-and-stable-ids
- spec: `22 §1` (stable ids + provenance), `§6`; `24 §6`
- given: a multi-clause definition; then an **unrelated** edit elsewhere (e.g.
  renaming a local in a *different* function)
- expect: each obligation carries `provenance` = source span + responsible
  clause (`requires`/`ensures`/refinement/`prove`); the obligation `id`s are
  **stable** across the unrelated edit (so a verification run diffs cleanly).
- why: §1/§6 — ids are stable across edits unrelated to the clause (`24 §6`),
  and provenance traces each obligation to its clause. Structural: the id of an
  untouched clause's obligation is unchanged after the unrelated edit; a bug
  keying ids on global position (not clause identity) flips them.

### verify/obligations/decoupled-from-sigma-sort (soundness)
- spec: `22 §1.1`, `§5`; `21 §2`/§6.3
- given: a function whose `ensures`/refinement would, under the naive reading,
  be a `Σ(B,ψ)` value
- expect: V2 reads V1's **bare-carrier + separate-obligation** form — the
  obligation is a free-standing `Γ ⊢ ψ`; V2 **never** forms or inspects a core
  `Σ` over an Ω predicate.
- why: §1.1 — V2 is decoupled from the `Σ`-sort erratum (`13 §4`, on
  `wp/V1-sigma-sort`): it consumes the carrier-plus-obligation encoding, never
  `Σ(B,ψ)`. Structural: the obligation is a standalone triple, not a projection
  of a `Σ`-typed body. (V1's `ensures-emits-obligation-not-sigma` pins the
  producer side; this pins the consumer side — internal consistency across the
  V1/V2 corpus.)

### verify/obligations/non-spec-program-empty-obligation-set (soundness)
- spec: `22 §8` (regression), `§6`; `21 §6.2`
- given: a non-spec program — no `requires`/`ensures`/refinement/`prove`/`law`,
  no partial primitive — e.g. `view id (A : Type) (x : A) : A = x`
- expect: the **empty** obligation set; V1/V0 elaboration of the program is
  **unchanged**.
- why: §8 — a program with no proof burden yields no obligations; V2 adds
  nothing to a spec-free program (the extractor's emit clauses never fire).
  **Regression guard.** Mirrors `../spec-syntax/seed-spec-syntax.md`
  `v0-unchanged-for-non-spec-programs` (V1 side): spec-free → unchanged
  elaboration (V1) → empty obligation set (V2).

---

## Coverage map (acceptance, `22 §8` / frame `§1`–§5)

- **#1 extraction completeness** — `refinement-introduction-emits-phi`,
  `postcondition-emits-substituted-goal`,
  `precondition-obligation-at-call-not-in-body`,
  `partial-primitive-emits-nonzero-obligation`,
  `prove-and-law-emit-one-obligation-per-goal`; the **absent-clause scan**
  (`refined-param-is-hypothesis-not-obligation`,
  `body-requires-assumed-not-reobligated`,
  `present-cert-yields-zero-new-obligations`,
  `forgetful-coercion-emits-nothing`) + the counter-rule
  (`trivial-clause-still-emits-obligation`) + the exhaustiveness backstop
  (`exhaustive-traversal-no-silent-skip` — the sole verification-soundness
  safeguard).
- **#2 Ω + context correctness** —
  `match-branch-gamma-carries-scrutinee-equation`,
  `let-binding-adds-equation-to-gamma`,
  `conditional-branch-adds-boolean-equation`, + body-as-motive
  (`recursive-fn-per-ctor-obligation-with-ih`,
  `nonrecursive-degenerate-no-induction-hypothesis`).
- **#3 decoupled from Σ-sort** — `decoupled-from-sigma-sort`.
- **#4 V2→V3 interface** — `inductive-postcond-hole-localization` (the
  obligation set → per-obligation verdict), `provenance-and-stable-ids`.
- **#5 no regression** — `non-spec-program-empty-obligation-set`.

Build-sequencing: V2 extends the **landed** V0/V1 elaborator (`39 §5`, `21 §6`),
consumes V1's four-part interface (`21 §7`) and the `18 §4`/§5 cert API, and
reads
the elaborator's `match → elim_D` (`39 §2.6`) + kernel eliminator (`14 §3`). All
sites are decoupled from the `Σ`-sort erratum (`wp/V1-sigma-sort`). The
obligation
goals stay in Ω by construction (`22 §7`: substitution preserves Ω, `11 §5`).
