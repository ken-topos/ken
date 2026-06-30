# Information-flow control conformance — seed cases (Sec1)

Format: `../../README.md`. These pin the tier-1 IFC guarantees of
`spec/60-security/61-information-flow.md` (Sec1, elaborated impl-ready),
grounded on the landed L5 interaction-tree denotation
(`spec/30-surface/36-effects.md`) and the verdict spine
(`spec/20-verification/`). This file **supersedes** the four `ifc/`
placeholders in `../seed-security.md` (retired in the same WP — see
"Placeholder absorption" below); the `authority/`, `supply-chain/`, and
`trust/` placeholders there are Sec2+ and untouched.

Grounding (landed `§`-bodies, content-reconciled — not the plan):
`61 §1`–`§9`/`§H` (the lattice, flow-typing, NI, `@ct` hook, honest limits);
`36 §2.1`/`§2.2`/`§2.4`/`§3.1` (`ITree`/`Vis`, `bind`, `incl`, the fixed
cross-workstream contract: an IFC label is an index on a `Vis` op/resp,
nothing effectful hides between nodes); `23 §1.2`/`§1.3` (verdict trichotomy +
the prover honesty guard); `21 §5` (the four-way status); `22 §1` (the unary
obligation); `18 §4`/`64 §2` (kernel re-check / authorship-independence);
`25 §3` (`trusted_base_delta`); `16 §1` (the `Ω` order `ℒ` rides); `62`
(declassification is capability-gated/audited).

## Reading these cases — the Sec1-specific disciplines

**IFC is relational (2-safety), with two strengths and DISTINCT observables
(`61 §1`/`§5`; LP-1, stage-split locked at source).** Non-interference (NI) is
a statement about *pairs* of runs and **cannot** be an `ensures φ` clause — it
is never authored as one here. The corpus splits cleanly:

- **By typing (strength 1, the default/load-bearing, `61 §5.2`).** Observable =
  **elaboration accept/reject** of the labeled program (the `L-SINK` channel
  rule `(ℓ ⊔ pc) ⊑ κ`, the monotone join, the `pc` discipline) — the exact
  `36 §1.4` EFFECT-ESCAPE flip pattern. NI itself is a **meta-theorem about the
  discipline** (and is itself *trusted* — mechanization is a named-future
  deliverable, `61 §H`), so these cases assert that **the rules fire correctly**
  (accept vs reject), never a per-program verdict and never a runtime two-run
  output diff. **The kernel does NOT backstop the flow discipline (N1,
  `61 §9`/`§H` row 1):** IFC labels are **erased** before the kernel (§3), so a
  flow-typing bug (wrong `⊑` in `L-SINK`, a dropped `pc`-join, a label-dropping
  `bind`/`incl`) emits a **well-typed core term the kernel accepts** while NI is
  violated — the by-typing flow rules are **trusted**, and these flip cases are
  the **sole net** (the §H meta-theorem + discriminating conformance, never the
  kernel). Groups A, C, F1.
- **By proof (strength 2, bespoke, `61 §5.3`).** Observable = **kernel re-check
  of the product-program certificate** → a V3 verdict (`23 §1`): a *related*
  pair → `proved`; a *distinguishing* pair → `disproved` with the pair as the
  witness; an undischargeable obligation → `incomplete` with a typed hole.
  **The reduction is itself a trusted step (N2, `61 §5.3`/`§H` row 2):** the
  product-program construction (the renaming, the `lowEq_ζ`/`coterminates_ζ`
  encoding) is done by the **untrusted** verifier, so the kernel re-checks the
  cert **for the obligation it is handed**, NOT that the obligation faithfully
  encodes 2-safety — a wrong reduction (too-weak `Φ_post`, dropped
  `coterminates_ζ`) yields a **kernel-valid cert for a non-NI claim**.
  `cert-recheck ≠ reduction-faithfulness`; the sole conformance backstop is the
  positive-soundness case D5 (a known leak must reduce to `disproved`). Group D.

**Honest-limits is a conformance-SCOPE constraint (`61 §H`, `64 §4`; LP-2).**
Over-claiming is itself the security failure, so each case asserts **exactly
what Ken proves** and **defer-tags (a reify-trigger, never silent)** what Ken
delegates: `[Sec1ct]` (the `@ct` timing-enforcement WP), `[Ward]` (runtime
timing validation under a leakage model), `[rel-deferred]` (the heavy
value-dependent product-program machinery). A case that asserted a delegated
guarantee (e.g. "`@ct` prevents a timing leak") would over-claim past Ken's
locked granularity — it is a wrong case.

**The no-laundering guard names its exact flip-bug (`61 §3.2`; LP-3,
absence-gate).** The IFC label rides the `Vis` op/resp (`36 §3.1`); `bind
(Vis e f) k = Vis e (λr. …)` reconstructs the *same* `Vis e` node (`36 §2.2`)
and `incl` re-tags only the *effect* tag (`36 §2.4`) — neither touches the
label index. The targeted bug a no-laundering case must flip on is a
**label-dropping `bind`/`incl`/handler at the `Vis` boundary**: green (rejects)
on the correct implementation, red (wrongly accepts) under exactly that bug.

**The verdict-trichotomy boundary invariants (the Glivenko-analog for the
cross-case sweep, `61 §5.3`/`23 §1.3`).** Grouped over the by-proof class
(group D): (1) a **non-interfering** program is never `disproved` — only a
**genuine distinguishing pair** maps to `disproved`-with-witness; (2) an
obligation the prover cannot settle is `incomplete`-with-hole, **never a false
`proved`** (the honesty guard carries to the relational domain — a prover bug
yields a weaker verdict, not a false accept); (3) **progress-sensitivity is the
default** — a `ζ`-equal-input pair where one run diverges/crashes is a leak
(`disproved`), and termination-*insensitivity* is opt-in only and **shows in
the four-way status** (`21 §5`), never silent.

**Tags.** `(soundness)` = a real confidentiality/integrity commitment that must
never regress. `(oracle)` = an expected result to confirm against Ken's
reference elaborator/interpreter once it exists, and (per LP-1) the literal
label *spelling* `@ ℓ` stays `OQ-syntax`-deferred — cases pin the value-set +
the flow invariant, not the surface token.

---

## A. Flow typing by accept/reject (AC1) — strength 1

### security/ifc/secret-to-public-rejected
- spec: `61 §3.1` (`L-SINK`), `§7`
- given: `log (e : String @ Secret[user])` to a sink of clearance `Public`
- expect: **rejects** — `Secret[user] ⊔ pc ⋢ Public`, an IFC-FLOW type error
  naming `ℓ`, `pc`, `κ`, and the sink site
- why: (soundness) the dominant AI-codegen leak, ruled out by typing. Verdict
  flip: the *same* `log` with the sink at clearance `Secret` (or `e` at
  `Public`) **accepts** — right=accept / wrong-path=reject on the discriminator
  `ℓ ⊑ κ`, never green-vs-green.

### security/ifc/integrity-taint-rejected
- spec: `61 §2.2` (the order-dual integrity lattice), `§3.1`, `§7`
- given: `exec (cmd : String @ Untrusted)` into a `Shell Trusted` sink
- expect: **rejects** — `Untrusted = ⊤_integ ⋢ ⊥_integ = Trusted`
- why: (soundness) taint/integrity, no untrusted data into a high-integrity
  sink (command injection). Pins the *dual* lattice direction (a bug that
  treated integrity like confidentiality — `⊔=∩` instead of `∪`, or `Untrusted`
  as `⊥` — would wrongly accept). Flip: `cmd : String @ Trusted` accepts.

### security/ifc/implicit-flow-pc-rejected
- spec: `61 §3.1` (`L-OBSERVE` → `L-SINK`, the `pc` discipline), `§7`
- given: `if (secret : Bool @ Secret) then send s 1 else send s 0`, where
  `s : Socket Public` — **the sent values `0`/`1` are themselves public**
- expect: **rejects** — in each branch `pc = ⊥ ⊔ Secret = Secret ⋢ Public`, so
  the public `send` is caught as an implicit flow
- why: (soundness) THE implicit-flow discriminator. The leak is the *choice*,
  not the datum; NI is false without the `pc`-join (precondition P1). Flip: a
  type checker that **drops the `pc`-raise in `L-OBSERVE`** wrongly accepts
  (the value is public) — green-vs-red on exactly the `pc` bug. (Contrast: with
  `secret : Bool @ Public`, the same program accepts.)

### security/ifc/combine-raises-label
- spec: `61 §3.1` (`L-COMBINE`)
- given: `f (x : A @ Secret) (y : B @ Public) : C`, the result written to a
  `Public` sink
- expect: result is `C @ (Secret ⊔ Public ⊔ pc) = C @ Secret`; the write
  **rejects** (`Secret ⋢ Public`)
- why: (soundness) computing **joins** labels — you cannot "forget" a label by
  computing on it. Flip: a `L-COMBINE` that took `ℓ₂` (or `ℓ₁ ⊓ ℓ₂`) instead of
  the join would type the result `Public` and wrongly accept the write.

## B. Declassification — the only downgrade (AC1)

### security/ifc/declassify-authorised-accepts-and-listed
- spec: `61 §4`, `62 §5`, `25 §3`, `63`
- given: the `Secret[user] → Public` flow via `declassify d (redact e)` with
  `d : Cap_declassify[Secret[user]→Public]` and `requires consented(e)`
- expect: **accepts**; the declassification authority appears in
  `trusted_base_delta`; the release is audited
- why: (soundness) controlled release is explicit, capability-gated,
  conditional, and **visible**. Pins both halves: the downgrade is permitted
  AND it surfaces in the delta.

### security/ifc/declassify-without-capability-rejected
- spec: `61 §4`, `62`
- given: the same `Secret[user] → Public` downgrade attempted **without**
  `Cap_declassify` in scope (or along a non-permitted edge `ℓ' ⋢ ℓ`)
- expect: **rejects** — only code holding the specific authority may downgrade,
  only along the permitted edge
- why: (soundness) declassification is capability-gated, not ambient. Flip
  against B1: same syntactic downgrade, capability present → accept, absent →
  reject. Right=accept / wrong=reject on the capability discriminator.

### security/ifc/declassify-absent-from-delta-is-infidelity
- spec: `61 §4`/`§H` (declassification audited + in delta), `25 §3`,
  `18 §5`
- given: a package that performs an authorised `declassify` but whose emitted
  `trusted_base_delta` **omits** that declassification authority
- expect: **rejected as an honesty-guard violation** — completeness of the
  delta is the sole backstop; a downgrade hidden from the delta is a silent
  confidentiality hole
- why: (soundness) the trusted_base_delta-completeness guard, here for a
  declassification (cf. the V2 silent-omission backstop:
  what a layer *omits* is not caught by re-checking what it *emits*). Flip: the
  same package with the authority **present** in the delta is accepted — the
  case flips on presence/absence in the delta, not on the declassify itself.

## C. No laundering through effects (AC2) — the load-bearing guard

### security/ifc/label-survives-effect-routing
- spec: `61 §3.2` (the no-laundering invariant), `36 §2.2`/`§2.4`/`§3.1`
- given: a `Secret` value routed to a `Public` sink **through** a sequence of
  `bind`s, a non-declassifying handler, and a callee-splice `incl` — i.e. the
  value passes through `Vis` nodes before reaching the sink
- expect: **still rejects** — the label rides the `Vis` op/resp; `bind`
  reconstructs the same `Vis e` and `incl` re-tags only the effect tag, so the
  value carries `⊒ Secret` at the sink and `(ℓ ⊔ pc) ⋢ Public`
- why: (soundness) AC2, the guard that a label cannot be stripped by routing a
  value through an effect. **Absence-gate, exact flip-bug named:** the case is
  green (rejects) on the correct implementation and **red (wrongly accepts)
  under a label-dropping `bind`/`incl`/handler at the `Vis` boundary** — the
  one bug AC2 exists to catch. It is NOT vacuous: a `bind` that dropped the
  index would let the laundered `Secret` reach `Public` and accept. **Doubly
  load-bearing under N1:** the label-dropping `bind`/`incl` emits a *well-typed*
  core term the kernel accepts (labels are erased), so the kernel cannot net
  this — **C1 is the sole backstop**.

## D. Non-interference by proof — relational verdict mapping (AC3)

> Strength-2 cases use the two-runs / product-program shape: a relational claim
> at observer clearance `ζ` reduces to one unary obligation
> `Γ ⊢ (Φ_pre ⇒ Φ_post) : Ω` over the product program (`61 §5.3`, `22 §1`),
> which the prover discharges and **the kernel re-checks** (`23 §1`, `18 §4`).

### security/ifc/related-pair-proved
- spec: `61 §5.1`/`§5.3` (low-equivalence, the product reduction)
- given: a bespoke NI claim for a well-labeled `view` at observer `ζ`;
  `product(c, ζ)` emits `lowEq_ζ(in¹,in²) ⇒ lowEq_ζ(out¹,out²) ∧
  coterminates_ζ` and it is **provable**
- expect: **proved** — discharged; the certificate is kernel-re-checked
- why: (soundness) the by-proof related-pair maps to `proved` (the verdict
  mapping pinned at source). A non-interfering program is **never** `disproved`
  (boundary invariant 1 — cross-case sweep member).

### security/ifc/distinguishing-pair-disproved-with-witness
- spec: `61 §5.3` (verdict mapping), `23 §1.2`
- given: a leaking `view` for which a **distinguishing pair** exists — two
  `ζ`-low-equivalent inputs whose `ζ`-observable outputs differ
- expect: **disproved(countermodel)** — the distinguishing pair **is** the
  leak-witness
- why: (soundness) a real leak maps to `disproved`-with-witness, **not**
  `unknown`. Flip / cross-case: the verdict-mapping silence resolved at source —
  a distinguishing pair must not read as `incomplete`. Contrast D1 on the same
  metatheory class (related → proved; distinguishing → disproved).

### security/ifc/unprovable-relational-incomplete-never-false-proved
- spec: `61 §5.3`/`§H`, `23 §1.3` (the honesty guard)
- given: a relational obligation the prover can **neither** discharge **nor**
  refute (e.g. a value-dependent claim needing the deferred machinery)
- expect: **incomplete(hole)** — a typed hole `?h` in `trusted_base()`; carries
  a `[rel-deferred]` reify-trigger; **never** `proved`
- why: (soundness) the prover honesty guard carries to the relational domain — a
  prover limitation yields a *weaker* verdict, never a false accept (boundary
  invariant 2). Absence-gate: the hole must be a **listed** postulate; an
  obligation silently dropped (not emitted) would read `proved` though
  unproven — the completeness-of-extraction backstop. (Also exercises LP-2:
  the deferred case is `[rel-deferred]`-tagged, not silent.)

### security/ifc/progress-sensitive-divergence-is-a-leak
- spec: `61 §5.2` (P4) / `§5.3` (the `coterminates_ζ` conjunct), `21 §5`
- given: a `view` non-interfering on *values* but where one run of a
  `ζ`-equal-input pair **diverges/crashes** while the other terminates
- expect: **disproved** by default (the `coterminates_ζ` conjunct fails — a
  crash/non-termination is itself a `ζ`-observable event); under the
  termination-**insensitive** opt-in the conjunct is dropped and the relaxation
  **shows in the four-way status** (`21 §5`)
- why: (soundness) progress-sensitivity is the default (boundary invariant 3);
  the relaxation is explicit, never silent. Flip: a checker that silently
  dropped `coterminates_ζ` (no status change) would wrongly read this as
  non-interfering — green-vs-red on the progress-sensitivity bug.

### security/ifc/reduction-faithfulness-interfering-disproved
- spec: `61 §5.3` (N2, the reduction is a trusted step), `§H` row 2
- given: a **known-interfering** program (a genuine leak) put through the
  by-proof path — `product(c, ζ)` emits the unary obligation, the prover
  discharges, the kernel re-checks the cert
- expect: the path must yield **disproved** — the reduction **cannot be
  massaged** (a too-weak `Φ_post`, a silently-dropped `coterminates_ζ`) to make
  the known leak read `proved`
- why: (soundness) **positive-soundness / reduction-faithfulness — the trusted
  part of the by-proof path (N2).** The kernel re-checks the cert for the
  *handed* obligation, **not** its faithfulness to 2-safety, so a kernel-valid
  cert for a too-weak obligation is a **false `proved`** that E1's forged-cert
  reject (a *non-typechecking* cert) does **not** cover. **Distinct from D2**
  (which assumes a faithful reduction and tests the verdict tag) **and E1** (a
  cert that fails to typecheck). Flip: a reduction that weakened `Φ_post` would
  let this known leak read `proved` — green-vs-red on reduction-faithfulness.
  This is the exhaustiveness-as-sole-backstop discipline in the relational
  domain — an unfaithful reduction is a producer **omission** the re-checker
  cannot see, so a positive interfering→`disproved` case is the only net.

## E. Kernel-re-checkable, not trusted (AC3)

### security/ifc/forged-label-or-cert-kernel-rejected
- spec: `61 §H` (no kernel enlargement), `64 §2` (authorship-independence),
  `18 §4`
- given: a relational certificate (or a labeled term) that is **fabricated**
  or **forged** — claims `proved` but does not kernel-typecheck
- expect: the consumer's kernel **re-check fails** → rejected; never a false
  `proved`
- why: (soundness) the de Bruijn criterion as a *security* property — a bug or
  malice in the prover/elaborator/SMT/AI yields a rejected certificate, never a
  false accept (`64 §2`). The IFC discipline adds **no** trusted primitive
  (labels are `Vis` indices); the same small kernel filters it.

## F. The `@ct` hook — source precondition only (AC4)

### security/ifc/ct-value-steers-leakage-sink-rejected
- spec: `61 §5a` (the `@ct` hook), `36 §3.1` (leakage-sink `Vis` op class)
- given: `cmp (k : Bytes @ ct) (g : Bytes)` whose body `branch_on (k[0]==g[0])`
  lets the `@ct` key steer a **branch guard** (a leakage-relevant `Vis` op)
- expect: **rejects** — `L-SINK` reused with the leakage-sink as `κ`; a type
  error (the source-level constant-time precondition `Q`)
- why: (soundness) the unary-taint enforcement of the source-level CT
  precondition (no product programs). Flip: the same value not steering a
  leakage op (or `k` not `@ct`) accepts. Also covers memory-index and
  variable-time-primitive sinks.

### security/ifc/ct-label-parses-carries-and-defers-timing
- spec: `61 §5a`/`§H`, `64 §4.2`, `63 §5a`
- given: a function exporting a signature-level CT promise; the `@ct` label is
  attached and routed through the denotation (riding a `Vis` index)
- expect: the `@ct` label **parses, attaches, and is carried** (assert the
  label **survives** the denotation, structurally) AND a `[Sec1ct]`/`[Ward]`
  **reify-trigger is present** at the leakage-sink op — **not silent**
- why: (soundness) for the precondition the label is real and checked;
  **(oracle) the timing guarantee itself is NOT asserted** — it is delegated to
  `[Ward]`/`[Sec1ct]` under a leakage model (LP-2 honest-limits scope). A case
  asserting "no timing leak" here would over-claim past what Sec1 lands.

## G. Honest limits — scope, no over-claim (AC5)

### security/ifc/deferred-machinery-carries-reify-trigger
- spec: `61 §5.3`/`§H` (`[rel-deferred]`, named not faked)
- given: a relational/quantitative claim ("at most `n` bits leak") that needs
  the **deferred** heavy product-program machinery
- expect: the case carries a **`[rel-deferred]` reify-trigger** (a named,
  non-silent gap) — it is **not** silently passed and **not** silently omitted
- why: (the honest-limits non-silence rule, LP-2 / `64 §4`) a deferred
  capability is named with its trigger, never faked. Distinguishes "Ken does
  not yet prove this" from "Ken proves this" — over-claiming is the failure.

### security/ifc/by-typing-meta-theorem-is-trusted-scope
- spec: `61 §5.2`/`§H` (the by-typing meta-theorem is itself *trusted*)
- given: a well-labeled program that type-checks under §3
- expect: conformance asserts **the rules fired correctly** (accept), i.e.
  rule-enforcement — and **does not** assert a mechanically-verified NI
  meta-theorem (mechanization is a named-future deliverable)
- why: (the scope-honesty rule, AC5) the by-typing NI guarantee rests on a
  design-level meta-theorem flagged trusted in `§H`; a conformance case that
  claimed to verify NI *mechanically* from a single type-check would over-claim.
  This case pins the honest scope: test the discipline, label the meta-theorem
  as the trusted boundary it is.

---

## Coverage map (AC → cases)

- **AC1** non-interference by typing + lattice + declassify → A1–A4, B1–B3.
- **AC2** labels compose through effects (no laundering) → C1.
- **AC3** kernel-re-checkable (by-proof + forged cert) → D1–D5, E1 (D5 = the
  N2 positive-soundness reduction-faithfulness backstop).
- **AC4** `@ct` hook present, enforcement deferred → F1, F2.
- **AC5** honest limits (proven/assumed/delegated/deferred, no over-claim) →
  G1, G2 (and the `(oracle)`/defer-tags throughout F2, D3).

## Cross-case consistency sweep (pre-handoff gate)

- **By-proof verdict-trichotomy class {D1, D2, D3, D4}** — assert verdict
  agreement on the shared metatheory: a non-interfering program is **never**
  `disproved` (D1); only a genuine distinguishing pair is `disproved`-with-
  witness (D2); an undischargeable obligation is `incomplete`-with-hole, never
  a false `proved` (D3); a `ζ`-equal divergence pair is a leak under the
  progress-sensitive default (D4). The Glivenko-analog for IFC.
- **The two trusted parts conformance is the sole net for (N1/N2,
  `61 §9`/`§H`).** (i) the **by-typing flow rules** — labels erased = the kernel
  is blind to a flow bug, so the flip cases {A1–A4, C1, F1} are the sole net;
  (ii) **reduction-faithfulness** in the by-proof path — the kernel re-checks
  the cert, not the obligation's fidelity to 2-safety, so the positive case D5
  (interfering → `disproved`) is the sole net. Both are the security analog of
  the verification layer's silent-omission backstop: a producer's *omission*
  (an over-accepting flow rule, an unfaithful reduction) is invisible to the
  re-checker — only a discriminating/positive conformance case nets it.
- **By-typing accept/reject class {A1–A4, C1, F1}** — each is a clean verdict
  flip (right=accept / targeted-bug=reject), none green-vs-green; the targeted
  bug is named per case (dropped `pc`-raise, `⊓`-for-`⊔`, integrity-as-
  confidentiality, label-dropping `bind`/`incl`, non-`@ct`-steer).
- **Absence-gate cases {C1, D3, D5, B3, F2}** — each names its exact guard (the
  `Vis`-index label, the listed-postulate hole, reduction-faithfulness, the
  delta-completeness backstop, the present reify-trigger) and passes "would
  this also be absent/rejected under the precise bug it targets?".

## Placeholder absorption (reconcile note)

This seed supersedes the four `ifc/` cases in `../seed-security.md`:
`secret-to-public-rejected` → A1; `integrity-taint-rejected` → A2;
`declassify-allowed-and-listed` → B1; `non-interference` (the relational
placeholder) → D1/D2 (split into the related/distinguishing pair with the
verdict mapping pinned). Those four are retired from `seed-security.md` in this
WP to avoid a stale-sibling contradiction; the `authority/`, `supply-chain/`,
and `trust/` placeholders there are out of Sec1 scope and untouched.

## Build-sequencing note

By-typing cases (A, B1–B2, C, F1) exercise the elaborator's flow-typing pass
(§3 rules) and need no kernel feature beyond landed K1.5. By-proof cases
(D1–D5) ride V3's prover + the kernel re-check (`23 §1`, `18 §4`) and the
product-program reduction; D3/D4 + F2 + G1 carry the `[rel-deferred]`/`[Ward]`/
`[Sec1ct]` reify-triggers — the deferred enforcement lands in those follow-on
WPs, not Sec1. E1 rides the kernel's existing re-check. The `Vis` label index
carries the side-condition **`ℓ_carrier ≤ ℓ_ITree`** (`61 §9` level table): a
parametric `Lattice` with a high-universe carrier must place its label at or
below `ℓ_ITree` (trivially true for DLM's level-0 `Set Principal`). Literal
label spellings (`@ ℓ`) stay `OQ-syntax`-deferred (`(oracle)`); cases pin
value-sets + invariants, not surface tokens.
