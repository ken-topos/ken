# The automated prover (V3) conformance — seed cases

Format: `../../README.md`. These pin **WS-V V3** — the automated prover: taking
one obligation `⟨id, Γ ⊢ φ, provenance⟩` (`φ : Ω`, from V2) to exactly one
**verdict** of the trichotomy `{proved | disproved | unknown}` (`23 §1.2`,
`21 §5.1`), with the load-bearing soundness property that `proved` is believed
**only** because the **kernel re-checks the certificate** (`18 §4.5`, the de
Bruijn criterion). Grounded in the **landed** `23-prover.md` (`§1`–§9), V2's
`22 §1`/§5/§6 (the obligation triple V3 consumes), the `18 §4.5`/§5 cert-API +
trusted base, `16 §1.3`/§9 (the derived `Decidable`/connectives + canonicity),
V1's `21 §5.1`–§5.4 status model, and first principles. The prototype is not
mounted; none of these required it.

**The trichotomy, and the central reconcile hazard (`23 §1.2`/§1.4).** The
prover's **output** is the verdict trichotomy `proved`/`disproved`/`unknown` —
**not** a "four-way" output, and **no** `failure` catch-all (a search that
neither closes nor refutes is `unknown`-with-hole, honest). V1's *four-way*
`proved`/`tested`/`delegated`/`unknown` is an **epistemic status** (`21 §5.2`)
that V1's projection (`21 §5.3`) computes from *verdict × disposition*. The
prover realizes **three** labels and **never** `tested`/`delegated` — those are
dispositions (`test`/`assume`/temporal) that **bypass V3 entirely**. **No case
here asserts V3 emits `tested` or `delegated`** — such a case would be wrong
(`23 §1.4`, the verdict-vs-status separation).

**The honesty guard is kernel-structural (`23 §1.3`).** `Γ ⊢ φ` is `proved`
**iff** a certificate `p` `check`s (`18 §4.5`) **and** no postulate carrying `φ`
sits in `GlobalEnv::trusted_base()` (`18 §5`, `21 §5.4`). An undischarged
obligation **is** a `declare_postulate` of `φ`, so its goal is enumerated by
`trusted_base()`; discharging retires the postulate. There is **no side-channel
/ parallel "proved" store** — a prover bug can leave a hole (`unknown`) or emit
a cert the kernel rejects (→ not `proved`), but can **never** forge `proved`. So
every absence-assertion below flips on **postulate membership**, not a
status-string compare.

**Two-soundnesses (the V3 instance — carry from V2/V2-build).** The kernel
re-checks what the prover **supplies** (a cert → `check`; `18 §4.5`), so a
*wrong* or *mis-translated* cert is caught (→ not `proved`). But a
**never-routed obligation** supplies **no** cert **and** leaves **no** hole, so
its goal never enters `trusted_base()` and the claim reads **discharged though
never attempted** — the exact V2 *omission* hazard (`22`), one tier up. The
kernel cannot see what the classifier **omits**. So the **structural totality of
`classify`** (HO is the always-applicable default arm; **no `_ ⇒ skip`** —
`23 §2.1`/§7.4) is the **sole** backstop against a dropped obligation, and it is
**not** kernel-enforced. Cases tagged **(soundness)** encode this and the
cardinal-rule/honesty commitments and must never regress.

**Vocabulary (reconcile, not cite).** `check` = the **trusted kernel re-check**
(`check(env, Γ, p, φ)`, `18 §4.5`) — the de Bruijn criterion, one call.
`check_cert` = the **Ken-level reflective Bool checker** over quoted formulas
(`23 §4` route (a)) — an ordinary kernel-checked function, **distinct** from the
kernel API `check`. `Decidable P` = the **derived** sum `P + (P → Empty)`
(`16 §1.3`), *not* a kernel primitive; canonicity is `16 §9` (C6). Diagnostic
*shapes* (countermodel structure, typed-hole payload) are `24`-owned — tagged
`(oracle)` where `24`'s schema has not landed; the **verdicts** are grounded
against `23` + `trusted_base()`.

---

## A. The cardinal rule — sound by kernel re-check (`23 §1.5`)

### verify/prover/discharged-goal-cert-kernel-accepts
- spec: `23 §1.2`/§1.3; `18 §4.5`
- given: a decidable obligation `⟨id, Γ ⊢ 5 > 0, prov⟩`; the prover produces a
  certificate `p` with `Γ ⊢ p : 5 > 0`
- expect: `check(env, Γ, p, 5 > 0)` **accepts** (`Ok`); verdict **proved**; no
  postulate carrying `5 > 0` in `trusted_base()` (the honesty guard, §1.3).
- why: §1.2 — `proved` is the cert verdict, believed because the kernel
  re-checks `p`. The accept half of the de Bruijn criterion; paired with the
  corrupted-cert flip below.

### verify/prover/corrupted-cert-kernel-rejects-unknown (soundness)
- spec: `23 §1.5`; `18 §4.5`/§4.4
- given: the **same** obligation `Γ ⊢ 5 > 0`, but the emitted certificate is
  **deliberately corrupted / mis-translated** — a term `p'` whose type is
  **not** `5 > 0`
- expect: `check(env, Γ, p', 5 > 0)` **rejects** (`TypeMismatch`); the verdict
  is **not** `proved` — it is **unknown** (the honest typed hole; no
  countermodel was found), so `5 > 0` **appears** in `trusted_base()`.
- why: §1.5 — the de Bruijn criterion **exercised, not assumed**.
  **Verdict-flip:** correct cert → `proved`; corrupted cert → `unknown`
  (opposite verdicts, on cert integrity alone). A prover bug can never forge
  `proved` — `check` is the firewall. **Disconfirming:** would a *correct* cert
  also read `unknown` here? No — it reads `proved` (the case above).
  Green-vs-red on the kernel's `check` result.

### verify/prover/classically-valid-topos-invalid-cert-rejected (soundness)
- spec: `23 §1.5`, `§4`, `§7`; `README.md §4`
- given: a goal `φ` that is **classically valid but topos-invalid** — an
  instance of `¬¬p ⇒ p` for an **undecidable** atom `p`; a **buggy** prover path
  trusts Z3's classical "valid" / bypasses the Kripke embedding and tries to
  emit a certificate for `φ`
- expect: **either** path lands at **not `proved`**: the buggy path's would-be
  certificate **fails** `check` (no kernel term of type `φ` exists — `φ` is
  topos-false), → **unknown**; the **correct** path routes **FO** (§4) and the
  embedding `φ#` is **not** classically valid (the intuitionistic clause for `⇒`
  blocks `¬¬`-elimination), so Z3 fails → honest **unknown**. Never `proved`.
- why: §1.5/§7 — a classical solver **cannot** make a Ken proof unsound, because
  the cert is re-checked regardless of how clever (or buggy) the backend is. The
  critical soundness regression (the `23 §9` / `seed-verify.md`
  `soundness-regression` case, deepened). **Verdict-flip:** a buggy prover that
  *would* report `proved` vs the kernel's `check` forcing **not `proved`**; the
  flip is on the re-check, not on the prover's say-so.

---

## B. The exhaustive classifier — totality is the omission backstop (`23 §2.1`)

### verify/prover/classify-routes-each-shape-D-FO-HO
- spec: `23 §2`/§2.1
- given: three obligations — (D) a decidable atom `2 + 2 == 4`; (FO) a
  first-order intuitionistic formula over decidable atoms (e.g.
  `∀x. P x ∨ ¬ P x` is *not* assumed, so an FO goal like `(∀x. P x) ⇒ P a` with
  abstract `P`); (HO) an inductive goal `∀ xs : List Nat. length xs ≥ 0`
- expect: `classify` routes them **D → direct/reflective decision (§3)**, **FO →
  Kripke embedding (§4)**, **HO → tactics (§5)**, respectively — each obligation
  receives **exactly one** route keyed by `shape(φ)`.
- why: §2/§2.1 — routing is by syntactic analysis of `φ`. **Structural output:**
  the **route label** per shape (not a verdict) — a misroute changes the emitted
  route. Mechanism-consistency across the fragment axis; pairs with the totality
  case below.

### verify/prover/unrecognized-shape-to-HO-default-no-skip (soundness)
- spec: `23 §2.1`, `§7.4`; `22 §2.5` (the V2 exhaustiveness analog)
- given: an obligation whose `shape(φ)` matches **neither** `decidableAtoms`
  **nor** `firstOrderIntuit` — an unrecognized / future formula shape over the
  fixed `Ω` former set (`16 §1`)
- expect: `classify` routes it to the **HO default arm** (`_ → HO`, §2.1) — it
  is **attempted** (tactics, or left an honest **typed hole**), and **never**
  silently dropped as if discharged. The `classify` `case` is **total with a
  default arm** — there is **no `_ ⇒ skip`**, and the **only** path to `proved`
  is through a kernel-`check`ed certificate.
- why: §2.1/§7.4 — exhaustiveness of routing is the **sole** safeguard against a
  dropped obligation (a never-routed `φ` supplies no cert **and** no hole →
  escapes `trusted_base()` → reads discharged though never attempted; the
  two-soundnesses *omission* gap, **not** kernel-enforced). **Structural /
  absence assertion (the guard is exhaustiveness by construction, not a value
  flip):** the routing `match` has **no catch-all `_ ⇒ skip`**; an unplaceable
  obligation is route-or-hole. **Disconfirming:** would a future `φ`-shape be
  silently dropped (reading as discharged) under a catch-all skip? **Yes** —
  that is exactly the bug this forbids; green (HO default, attempted/hole) vs
  red (silent vanish). The V3 analog of V2's
  `exhaustive-traversal-no-silent-skip`.

---

## C. The reflective bridge — backend result → checkable term (`23 §3`/§4)

### verify/prover/reflective-decision-computes-cert-D
- spec: `23 §3.1`; `16 §1.3` (derived `Decidable`), `16 §9` (canonicity)
- given: a **closed** decidable goal `2 + 2 == 4` with a kernel-verified
  decision procedure `dec : (x : A) → Decidable (φ x)`,
  `Decidable P = P + (P → Empty)`
- expect: the certificate is **by computation** — the kernel evaluates `dec a`
  (canonicity, `16 §9`) to `inl proof`; that `proof : (2 + 2 == 4)` `check`s →
  **proved**, with **no external solver in the trusted path**. The companion
  **false** decidable goal `2 + 2 == 5` evaluates to `inr refutation` →
  **disproved**.
- why: §3.1 — the computing kernel discharges the decidable fragment directly.
  **Verdict-flip on the atom's truth:** true → `inl proof` → `proved`; false →
  `inr refutation` → `disproved`. **Structural output:** the cert is the
  *computed* `inl`/`inr`, not a solver's word. `Decidable` is the **derived**
  sum (`16 §1.3`), not a primitive.

### verify/prover/kripke-embedding-cert-rechecks-FO
- spec: `23 §4` (route (a) + the proved-vs-assumed ledger); `16 §9`; `18 §4.5`
- given: a first-order **intuitionistic** obligation `φ` routed to the Kripke
  embedding (§4); Z3 decides the translation `φ#` valid
- expect: the discharge is the **reified term** `sound φ π (refl true)`, where
  the reflective `check_cert (embed φ) π` reduces to `true` (canonicity,
  `16 §9`); the **kernel** `check`s that discharge term (`18 §4.5`) →
  **proved**. Because the embedding-adequacy lemma `classically_valid(φ#) → φ`
  **and** the checker-soundness lemma `check_cert (embed φ) π = true → φ` are
  **PROVED** kernel defs (the §4 ledger), **nothing is added to
  `trusted_base()`** — the FO `proved` is fully kernel-grounded. (The Kripke
  frame axioms are external `(oracle/standard)`, shaping `φ#` — the classical
  theory handed to Z3 — only; they are not Ken terms.)
- why: §4 — the classical solver is used **soundly** via the embedding; what is
  trusted is the **re-checked reified term**, not Z3's "valid". **Structural
  output:** the discharge term `sound φ π (refl true)` that `check`s, and the
  **empty** `trusted_base()` delta (adequacy is proved, not assumed). Pairs with
  the no-`π` flip below.

### verify/prover/bare-unsat-no-cert-is-unknown-not-proved (soundness)
- spec: `23 §4` (ledger row: Z3 "unsat" assumes nothing), `§1.2`/§1.5
- given: a backend returns "unsat" / "valid" for `φ#` but yields **no
  constructible certificate `π`** (reconstruction fails, or the backend emits no
  usable proof object)
- expect: there is no `check_cert`-passing `π`, hence **no** discharge term to
  `check` → verdict **unknown** (typed hole, goal in `trusted_base()`), **not**
  `proved`.
- why: §4 ledger / §1.5 — a backend "unsat" with **no constructible `π`** is
  `unknown`, never `proved` (the same de Bruijn discipline as D, one tier up).
  **Verdict-flip on cert-constructibility:** a `check_cert`-passing `π` →
  `proved` (the case above); no `π` → `unknown`. **Guard:** `proved`
  **requires** a kernel-`check`ed cert; Z3's bare word is "assumed nothing."
  **Disconfirming:** would a backend **with** a valid `π` also read `unknown`?
  No — it reads `proved`.

---

## D. Higher-order — IPC + sub-obligation descent (`23 §5`)

### verify/prover/ipc-valid-propositional-proved
- spec: `23 §5` (propositional skeleton)
- given: an **intuitionistically valid** propositional goal with **abstract**
  atoms, e.g. `(p ∧ q) ⇒ p`
- expect: the kernel-verified **IPC decision procedure** (an `Itauto`-style
  reflective tactic) returns a **proof term**; the kernel `check`s it →
  **proved**.
- why: §5 — the IPC tactic decides the intuitionistic connective scaffolding
  even when atoms are abstract. Pairs with the classically-valid-but-IPC-invalid
  flip below (the propositional analog of A's FO soundness regression).

### verify/prover/ipc-lem-invalid-not-refuted-unknown (soundness)
- spec: `23 §5` (the pinned counter-model verdict), `§1.2`; `24 §1`/§3 (the
  invalid-vs-refuted distinction — `(oracle)` model shape)
- given: `p ∨ ¬ p` for an **abstract** atom `p` — **classically valid**,
  **intuitionistically invalid** but **not refutable**
- expect: the IPC procedure returns a **counter-to-validity** Kripke model (a
  world that **fails to force** `p ∨ ¬ p`), **not** a proof and **not** a model
  forcing `¬φ` → verdict **unknown** (typed hole `?id : φ`, goal in
  `trusted_base()`), **not** `disproved`.
- why: §5 (pinned, `d736b0f`) / §1.2 — a **classically-valid** formula is
  **never `disproved`**: `¬¬(p ∨ ¬ p)` is a theorem, so `¬(p ∨ ¬ p)` is itself
  false and **no** Kripke model forces `¬φ` (it is *invalid*, not *refutable*).
  The IPC procedure returns a counter-to-validity model — the **unknown**
  diagnostic (`24 §1`/§3: the `{¬¬φ} ∖ S_φ` gap, the `¬¬φ ⇒ φ` content), **not**
  a refutation. **Verdict-flip (proved-vs-unknown):** intuitionistically valid
  (`(p ∧ q) ⇒ p`, D1) → **proved**; classically-valid-but-intuit-invalid
  (`p ∨ ¬ p`) → **unknown** — pinning the §1.2 invalid-vs-refuted boundary.
  **Internal-consistency:** matches **A3** (same class — classically valid,
  `¬φ` unprovable → `unknown`); `disproved` is reserved for genuinely-refutable
  goals where `¬φ` is provable (C1's `2 + 2 == 5`, E1's `n > 0` on `n ≤ 0`). A
  build coding `disproved` here would report an `unknown` as a refutation —
  "fix the code" when the honest message is "supply more facts" (`24 §1`).

### verify/prover/induction-descent-with-ih-and-localized-partiality (soundness)
- spec: `23 §5` (sub-obligation descent + certificate composition); `14 §3`
  (eliminator); `22 §4` (the V2 extraction analog)
- given: an HO goal needing induction, `∀ xs : List Nat. length xs ≥ 0`; the
  induction tactic decomposes it — **(a)** with every sub-certificate supplied;
  **(b)** the **same** with the `cons`-branch sub-certificate **removed**
- expect: the tactic emits **one subgoal per constructor** — `nil`:
  `length nil ≥ 0` (`0 ≥ 0`); `cons y ys`: `length (cons y ys) ≥ 0` **with the
  induction hypothesis** `M ys = (length ys ≥ 0)` in `Γ`; the composed
  certificate is the eliminator application `elim_List M methods… xs`, `check`ed
  **once** at the top goal (`18 §4.5`). **(a)** all sub-certs → **proved**.
  **(b)** the `cons` sub-certificate removed → a **single, precisely-located
  typed hole** at the `cons` subgoal (status **unknown**, its goal in
  `trusted_base()`); the `nil` leaf + the other subgoals stay **proved** —
  partiality is **per-subgoal**, not all-or-nothing.
- why: §5 — a tactic that decomposes a structured goal generates **routed**
  sub-obligations, each carrying its hypotheses (the IH for recursive fields),
  and **composes** the certificate; a single goal over the whole inductive
  structure carries **no IH** and cannot be discharged (descent is **required**,
  not an optimization). **Structural/verdict flip:** the `cons` step is provable
  **with** the IH, unprovable **without** it (a bug dropping the IH → that leaf
  is an `unknown` hole). **Disconfirming:** does removing one sub-proof yield a
  vague global failure, or exactly one localized `unknown` leaf with siblings
  `proved`? §5 requires the latter. The prover-side instance of the
  obligation-descend carry (V2's `recursive-fn-per-ctor-obligation-with-ih` /
  `inductive-postcond-hole- localization`: the **tactic** synthesizes here what
  V2 reads at extraction).

---

## E. Honest trichotomy — disproved + unknown evidence (`23 §1.2`/§1.3)

### verify/prover/disproved-carries-countermodel
- spec: `23 §1.2`; `24 §1` (countermodel shape — `(oracle)` where unlanded)
- given: a **false** obligation —
  `view f (n : Int) : Int ensures result > 0 = n` (false for `n ≤ 0`); its
  postcondition goal `Γ ⊢ n > 0`
- expect: verdict **disproved**, evidence = a **countermodel** naming the
  failing input class (`n ≤ 0`); where the backend yields a proof of `¬φ`, the
  cert `q : ¬φ` is `check`ed too (`check(env, Γ, q, ¬(n > 0))`).
- why: §1.2 — a genuine counterexample is reported with the false-vs-unknown
  distinction (a refuted claim is **fixed**, not shipped — `21 §5.3`; it is
  **not** exported as an epistemic status, §1.4). **Verdict-flip:** a
  satisfiable postcondition → `proved`/`unknown`; a falsifiable one →
  **disproved** with a concrete witness. Diagnostic *payload* shape is
  `24`-owned (`(oracle)`); the **verdict** is grounded.

### verify/prover/unknown-hole-trusted-base-distinct-from-proved (soundness)
- spec: `23 §1.3`/§1.2; `18 §5`; `21 §5.4`
- given: an obligation `Γ ⊢ φ` the prover can **neither** discharge **nor**
  refute (no certificate, no countermodel)
- expect: verdict **unknown**; a typed hole `?id : φ` is admitted as a
  `declare_postulate` of `φ`, so its goal **appears in**
  `GlobalEnv::trusted_base()` — making `unknown` **`trusted_base()`-distinct
  from `proved`** (whose discharged certificate **retires** the postulate, so
  `φ` is **absent** from the base). The program **still runs**, producing
  `unknown` where the property is observed.
- why: §1.3 — `proved` **iff** the cert `check`s **and** no postulate carrying
  `φ` is in `trusted_base()`. **Absence-assertion gate — guard = postulate
  membership** (a **structural** flip on `trusted_base()`, **not** a
  status-string compare, §1.3 explicit). **Disconfirming:** would a **proved**
  goal also appear in `trusted_base()`? **No** — discharging retires the
  postulate (`21 §5.4`; the certificate replaces the assumption). Green
  (`proved`, `φ` **absent** from base) vs red (`unknown`, `φ` **present**), on
  base membership — guard-gated, not coincidental. The §1.3 honesty-guard
  flagship.

---

## F. Regression — V2/V1 interface consumed unchanged (`23 §1.1`/§1.4)

### verify/prover/pure-pipeline-no-obligations-unaffected
- spec: `23 §1.1`; `22 §8` (empty obligation set); `21 §6.2`
- given: a **non-spec** program — no
  `requires`/`ensures`/refinement/`prove`/`law`, no partial primitive (e.g.
  `view id (A : Type) (x : A) : A = x`), so V2 emits the **empty** obligation
  set
- expect: V3 produces **no** verdicts (the per-obligation function is never
  invoked); the V1/V0 elaboration **and** pure evaluation are **unchanged**.
- why: §1.1 — the prover is a per-obligation function with no cross-obligation
  state; an empty input set yields no work. **Regression guard.** Mirrors V2's
  `non-spec-program-empty-obligation-set` (producer side) and X1's
  `pure-program-never-reaches-driver` — a spec-free program is untouched
  end-to-end.

### verify/prover/verdict-keyed-by-id-no-side-channel (soundness)
- spec: `23 §1.3`/§1.4; `21 §5.3`
- given: a set of obligations with stable `id`s (`22 §1`/§6); the prover
  attempts each independently
- expect: V3 emits the **trichotomy verdict keyed by `id`** for V1's projection
  (`21 §5.3`); the `proved`/`unknown` distinction is read from
  **`trusted_base()` membership + the `check` result**, with **no side-channel /
  parallel "proved" store**. The verdict is decidable from the **kernel's own
  state**.
- why: §1.3/§1.4 — `proved` is **kernel-structural**, not a prover flag (the
  V1-build kernel-structural-status carry). **Structural assertion:** the
  verdict is derived from kernel state (`trusted_base()` + `check`), not a
  written flag; a bug introducing a side-channel "proved" store (forging
  `proved` without a `check`ed cert) is exactly what §1.3 forbids. Keying by
  `id` (not position) keeps the verdict stable for `21 §5.3` to project into the
  epistemic status.

---

## Coverage map (acceptance, `23 §9` / frame AC §1–§5)

- **AC1 sound by re-check** — `discharged-goal-cert-kernel-accepts`,
  `corrupted-cert-kernel-rejects-unknown` (the de Bruijn flip),
  `classically-valid-topos-invalid-cert-rejected` (the critical soundness
  regression). The cert re-check also underwrites C and D.
- **AC2 honest trichotomy** — `disproved-carries-countermodel`,
  `unknown-hole-trusted-base-distinct-from-proved` (the §1.3 absence-assertion).
- **AC3 exhaustive classifier** — `classify-routes-each-shape-D-FO-HO`,
  `unrecognized-shape-to-HO-default-no-skip` (the two-soundnesses
  *omission* backstop, structural).
- **AC4 reflective bridge** — `reflective-decision-computes-cert-D`,
  `kripke-embedding-cert-rechecks-FO`,
  `bare-unsat-no-cert-is-unknown-not-proved`.
- **AC5 no regression** — `pure-pipeline-no-obligations-unaffected`,
  `verdict-keyed-by-id-no-side-channel`.
- **D/FO/HO fragments** — D: `reflective-decision-computes-cert-D`,
  `discharged-goal-cert-kernel-accepts`; FO:
  `kripke-embedding-cert-rechecks-FO`,
  `classically-valid-topos-invalid-cert-rejected`,
  `bare-unsat-no-cert-is-unknown-not-proved`; HO:
  `ipc-valid-propositional-proved`,
  `ipc-lem-invalid-not-refuted-unknown`,
  `induction-descent-with-ih-and-localized-partiality`; all three driven through
  `classify-routes-each-shape-D-FO-HO`.

Build-sequencing: V3 extends the **landed** V1/V2 verification spine — it
consumes V2's obligation set (`22 §1`/§6) and the `18 §4.5`/§5 cert API +
trusted base, emits the trichotomy verdict keyed by `id` for V1's status
projection (`21 §5.3`), and adds **no new kernel former or universe** (`23 §8`:
goals stay `φ : Ω_ℓ`, the proof `p : φ` at the same `Ω_ℓ`; the Kripke embedding
`φ#`/`World`/`P#` are **external** to the kernel; `Form`/`Cert`/`Decidable` are
derived inductives at their natural `Type ℓ`; adequacy + `check_cert`-soundness
are kernel-checked terms landing in Ω). Z3/cvc5 are **never** in
`trusted_base()` (`18 §5`, `23 §7`). Diagnostic *shapes* (countermodel, typed
hole) are `24`-owned and tagged `(oracle)` here pending `24`'s landed schema.
