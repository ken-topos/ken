# Information-flow control

> Status: **Sec1 elaborated** — implementation-ready for WS-Sec. **Normative
> for the label lattice, the flow-typing discipline, the non-interference
> statement and its honest limits, and the `@ct` hook** (§2–§5a, §H);
> the concrete surface *spelling* of labels stays proposal-level (`OQ-syntax`).
> **Settled inputs (do *not* reopen): `OQ-ifc` DECIDED** (lattice-parametric +
> DLM, §2); **`OQ-relational` DECIDED** (by-proof = re-checked product
> programs, progress-sensitive; heavy machinery deferred, §5); **constant-time =
> an opt-in `@ct` label** whose *timing* guarantee is delegated to `Ward` +
> toolchain (§5a, `64 §4.2`). This is the centerpiece of Ken's tier-1 security
> goal (ADR 0004): **data-flow control is intrinsic to Ken**, built on the
> machinery Ken already has (the L5 interaction-tree denotation, `36`),
> and it directly attacks the dominant AI-codegen failure — secret leakage and
> data crossing trust boundaries. **It adds *nothing* to the trusted kernel**
> (labels are indices on the existing `ITree` container, `36 §3.1`).

## 1. Why flow is a *discipline*, not "more refinements"

Ken's verification layer proves **unary** properties: an obligation `Γ ⊢ φ : Ω`
is a predicate over **one** execution ("this result is sorted"). Information
flow is different in kind. "This secret never reaches the network" /
**non-interference** is a statement about **pairs** of executions:

> for any two runs that differ only in their *secret* inputs, the *public*
> outputs are identical.

This is a **relational (2-safety)** property. It **cannot** be written as an
`ensures φ` clause, because `φ` can only see one run — **this is normative, not
incidental** (§5 depends on it: non-interference is a *meta-theorem about the
discipline*, never a per-program `ensures`). So information-flow control is a
**typed discipline** that makes flow visible in types and *guarantees*
non-interference, plus (for bespoke claims) a relational verification mode that
**reduces to** unary kernel-checked obligations (§5) — not a library of ordinary
refinements. Conflating the two is the trap this chapter exists to prevent.

## 2. The security-label lattice

Ken adds a **label lattice** `ℒ` — a bounded lattice of security levels with
order `⊑` ("flows to"), join `⊔` (least upper bound), and meet `⊓`. **`ℒ` is an
ordinary Ken value, not a kernel primitive:** it is a record of a carrier plus
operations plus the lattice laws as `Ω`-valued obligations, riding ordinary
record/Σ (`../10-kernel/13 §3`, `14 §4`) with the order relation valued in `Ω`
(`../10-kernel/16 §1`). No new kernel former is introduced to host it (the
laws-as-`Ω`-obligations are discharged once per instance; `ℒ` is distinct from
`Ω`'s *own* Heyting algebra — it is a value *of* that machinery, not the
machinery).

### 2.1 The lattice interface (what an instance supplies)

```
record Lattice : Type (suc ℓ) where     -- an ordinary value; ℓ = carrier level
  L     : Type ℓ                          -- the carrier of labels
  _⊑_   : L → L → Ω                        -- "flows to" — the order, Ω-valued (16 §1)
  _⊔_   : L → L → L                        -- join: least upper bound
  _⊓_   : L → L → L                        -- meet: greatest lower bound
  ⊥ ⊤  : L                                 -- bottom (most public) / top (most secret)
  -- laws, each an Ω-obligation discharged ONCE per instance:
  ⊑-refl    : (a : L) → a ⊑ a
  ⊑-trans   : (a b c : L) → a ⊑ b → b ⊑ c → a ⊑ c
  ⊑-antisym : (a b : L) → a ⊑ b → b ⊑ a → Eq L a b
  ⊔-lub     : (a b : L) → a ⊑ (a ⊔ b) ∧ b ⊑ (a ⊔ b)
              ∧ (c : L) → a ⊑ c → b ⊑ c → (a ⊔ b) ⊑ c
  ⊓-glb     : (a b : L) → (a ⊓ b) ⊑ a ∧ (a ⊓ b) ⊑ b
              ∧ (c : L) → c ⊑ a → c ⊑ b → c ⊑ (a ⊓ b)
  ⊥-least   : (a : L) → ⊥ ⊑ a
  ⊤-grtst   : (a : L) → a ⊑ ⊤
```

Data flows **upward** only: a value `@ ℓ` may be observed where the context's
clearance `κ` satisfies `ℓ ⊑ κ`. `⊥` is the most public level (everything flows
to it nowhere-more-restricted — it is the *least* secret), `⊤` the most secret.

### 2.2 The DLM standard instance (`OQ-ifc` default)

The default instance is the **decentralised label model (DLM)** — a full label
is the **product** `(Conf × Integ)`, with optional compartment/region factors.
Each factor is a lattice of **principal sets** (a `Principal` is an opaque,
policy-supplied type, `65`; a set is an ordinary finite `data` value, `34`):

- **Confidentiality** `Conf = Set Principal` — the principals permitted to
  **read**. Ordered by **reverse inclusion** (fewer readers = more secret):

  ```
  c₁ ⊑ c₂   ⟺   readers(c₂) ⊆ readers(c₁)
  c₁ ⊔ c₂   =   readers(c₁) ∩ readers(c₂)     -- combine ⇒ only common readers (more secret)
  c₁ ⊓ c₂   =   readers(c₁) ∪ readers(c₂)
  ⊥_conf    =   AllPrincipals  (Public)        ⊤_conf = ∅  (no reader — maximally secret)
  ```

- **Integrity** `Integ = Set Principal` — the principals who may have
  **influenced** the value (its taint). The **order-dual** of confidentiality
  (more influencers = more tainted = higher):

  ```
  i₁ ⊑ i₂   ⟺   influencers(i₁) ⊆ influencers(i₂)
  i₁ ⊔ i₂   =   influencers(i₁) ∪ influencers(i₂)   -- combine ⇒ tainted by everyone who touched either
  ⊥_integ   =   ∅  (Trusted)                          ⊤_integ = AllPrincipals  (Untrusted)
  ```

  So `Trusted = ⊥ ⊑ Untrusted = ⊤`: low-integrity (attacker-influenced) data may
  **not** flow into a high-integrity sink, because a sink demanding integrity
  carries a *low* taint-clearance `κ` and `Untrusted ⋢ κ`. This is precise
  **taint tracking**.

- **Products and sugar.** The full label `(Conf × Integ × …compartments)` is
  ordered **componentwise** (a product of lattices is a lattice; `⊔`/`⊓`/`⊑` act
  pointwise) — the proof is once, generic. The totally-ordered chain `Public ⊑
  Internal ⊑ Secret` is **sugar** (nested reader-sets); data-residency regions /
  compartments are **product factors**.

Because the discipline is **lattice-parametric** (§5 proved once for *any*
`Lattice`), the concrete instance — principals, classifications, clearances, and
the permitted declassification edges (§4) — is a **policy** supplied separately
(`65-policy.md`, ADR 0007), not baked into the metatheory. "Fixed levels vs.
principal-sets vs. user-defined" dissolves: all are instances.

> **(OQ-ifc) — DECIDED (operator, 2026-06-27).** Lattice-parametric + DLM, as
> above. Labels are **static type indices by default** (erasable, by-typing,
> zero runtime cost) with **first-class label values admitted at boundaries**
> (§3.3) for data-derived classification. Do **not** reopen.

## 3. Labeled values and labeled effects (riding the L5 denotation)

A label attaches to data via a **labeled type** `A @ ℓ` (read "an `A` classified
at `ℓ`"): `A` paired with a label `ℓ : ℒ.L` as an **erasable type index** — at
the kernel it *is* `A`, the label living on the typing derivation and, for
effects, on the interaction-tree node (below). This is a **direct extension of
Ken's effect encoding** (`../30-surface/36 §2`, `OQ-8`): the same indexed-effect
machinery that indexes *capabilities* here indexes *labels* — a protection monad
in the DCC (Dependency Core Calculus) lineage *(named to understand, not
copied)*.

**Where the label lives (grounded in landed L5).** Per the L5 cross-workstream
contract (`36 §3.1`, the fixed contract Sec1 rides), an IFC label is **an index
on a `Vis` op/resp** of the interaction-tree denotation (`36 §2.1`: `ITree`'s
`Vis e k` node, **admitted as of K1.5 `f037451`** — a genuine strictly-positive
inductive, no new kernel primitive). The three guarantees Sec1 *depends on* from
that contract:

1. **Manifest-in-the-type** — a function's labels are recoverable from its type
   (the latent row + labeled parameters), never only from its body.
2. **Every authority-relevant act is a `Vis` node** — the IFC sinks are
   *exactly* the tree's `perform` sites; **nothing effectful hides between
   nodes** (this is what makes the no-laundering invariant, §3.2, hold).
3. **Discharge is visible** — only a handler removes an effect/label from a row.

Sec1 **indexes** the existing `Vis` container; it adds **no kernel primitive**
(`36 §3.1`, the explicit consumer constraint).

### 3.1 The flow-typing judgment (the discipline, as defensive pseudocode)

The judgment carries a **program-counter label** `pc : ℒ.L` — the join of the
labels of every value the control flow has branched on so far (it tracks
**implicit** flows). `Γ; pc ⊢ e : A @ ℓ` reads "under `Γ` and context clearance
`pc`, `e` is an `A` at label `ℓ`." The rules — **each soundness-relevant guard
is explicit, and each is backed by a rejection case (§8 conformance)**:

```
(L-PURE)      a literal / pure value v : A
              ──────────────────────────────────────────
              Γ; pc ⊢ v : A @ pc
              -- a value's label is at least the pc it was computed under

(L-COMBINE)   Γ; pc ⊢ x : A @ ℓ₁     Γ; pc ⊢ y : B @ ℓ₂     f pure
              ──────────────────────────────────────────────────────
              Γ; pc ⊢ f x y : C @ (ℓ₁ ⊔ ℓ₂ ⊔ pc)
              -- COMPUTING JOINS LABELS: you cannot "forget" a label by computing on it

(L-OBSERVE)   Γ; pc ⊢ c : Bool @ ℓ_c
              branches t,u checked at  pc' = pc ⊔ ℓ_c
              ────────────────────────────────────────
              Γ; pc ⊢ (if c then t else u) : _ @ (pc' ⊔ …)
              -- BRANCHING RAISES pc IN THE BRANCHES — closes the implicit-flow channel

(L-SINK)      Γ; pc ⊢ msg : M @ ℓ      sink s : Channel κ      (the perform is a Vis node)
              ───────────────────────────────────────────────────────────────────────────
              write s msg : Unit  visits [E]    WELL-FORMED iff  (ℓ ⊔ pc) ⊑ κ
                                                else  IFC-FLOW static error
              -- THE single soundness-relevant gate. On failure: name ℓ, pc, κ, and the
              --   offending sink site (a source location, not just a set difference).
```

`L-SINK` joins `pc` into the check so an **implicit** flow — writing inside a
secret-dependent branch — is caught even when `msg` itself is public. A channel
(`Net`, `FS`, a log, an RPC, a `space` cell — `36 §4`) carries a **clearance
label** `κ` as its `Vis`-op index; writing `@ ℓ` to it type-checks **iff `ℓ ⊔ pc
⊑ κ`**. So:

```
view send (s : Socket κ) (msg : Bytes @ ℓ) : Unit  visits [Net]
  -- type-checks only when  ℓ ⊑ κ  (and pc ⊑ κ at the call site)
```

"This secret must never reach the network" becomes: the `Net` sink's clearance
is below `Secret`, so a `Secret`-labeled value flowing to it is a **type error**
at compile time — not a lint, not a scan, a typing failure. A function's type is
thus simultaneously its **capability manifest** (`36 §3`) and its **flow
manifest**.

### 3.2 The no-laundering invariant (load-bearing; the §8 AC2 guard)

A label **cannot be stripped by routing a value through an effect**. This rests
on the L5 contract, made precise:

- **`bind`-sequencing preserves labels.** `bind ⟦e₁⟧ (λ x. ⟦e₂⟧)` (`36 §2.2`)
  feeds `e₁`'s result into `e₂` as `x`; `x`'s label flows on by `L-COMBINE`.
  `bind` grafts continuations onto `Ret` leaves — it **does not touch the label
  index** on any `Vis` node it threads through.
- **`incl_{ρ_g ↪ ρ}` re-tags without dropping.** The callee-splice map
  (`36 §2.4`) re-tags `Vis` *effect* tags along the signature inclusion; it
  **leaves the IFC label index untouched**. A callee's labeled nodes keep their
  labels in the caller's tree.

**Therefore:** a `Secret` value routed through *any* sequence of binds, handlers
(that don't declassify), and callee-splices to a `Public` sink **still carries
`⊒ Secret`** at the sink, so `(ℓ ⊔ pc) ⋢ Public` and `L-SINK` **rejects**.

> **The exact bug conformance must flip on (absence-gate, not vacuous):** a
> `bind`/`incl`/handler implementation that **drops or lowers the label index at
> a `Vis` boundary** would let a laundered `Secret` reach a `Public` sink and
> **accept**. The AC2 case must therefore be **green** (rejects) on the correct
> implementation and **red** (wrongly accepts) under exactly this label-dropping
> bug — a verdict flip, never a happy path (COORDINATION §7).

### 3.3 Data-derived classification — static by default, dynamic at the boundary

Most classification is **static** (policy- or source-determined, `65`) and costs
nothing at runtime. Real systems also classify **from data at runtime** (e.g.
per-tenant: "customer X's records go only to bucket X"). Ken supports this
**minimally**, not via full dynamic IFC:

- **Tag at ingestion.** A boundary reads the datum's compartment and assigns a
  label — a **first-class label value** — producing `A @ Tenant[X]`. It is then
  carried **statically** as an existential `∃ ℓ. A @ ℓ`; only the *value* of `ℓ`
  is dynamic.
- **Check at the sink.** Writing `@ Tenant[X]` to a sink of clearance
  `Tenant[Y]` is a flow violation (the `L-SINK` rule), caught with a **single
  runtime label comparison** where two dynamic labels meet. The static majority
  pays nothing.
- **Ingestion is trusted + audited.** Asserting "this record *is* Tenant[X]" is
  a **classification, not a proof** — so it is **capability-gated and audited**,
  the dual of declassification (§4); only **policy-sanctioned** ingestion points
  may classify (`65`). This is an *assumed* boundary (honest-limits, §H).

Deliberately **excluded**: faceted execution / pervasive runtime taint / dynamic
lattices (full dynamic IFC) — not worth their cost for Ken's targets (*better is
the enemy of good*). Ken pays for dynamism only at the boundary that demands it.

## 4. Declassification — the only downgrade, explicit and audited

Pure label-monotonicity (§3) would make secrets permanently unusable. Controlled
release is **declassification** (confidentiality) / **endorsement** (integrity),
the *only* way a label moves down:

```
declassify : Cap_declassify[ℓ→ℓ'] → A @ ℓ → A @ ℓ'      -- requires ℓ' ⊑ ℓ
```

- **Capability-gated** (`62-authority.md`): only code holding the specific
  declassification authority may downgrade, and only along the permitted lattice
  edge.
- **Explicit and local** — a syntactic operation at a named point, never
  implicit, so "where does PII get released?" is answered by *grepping for
  `declassify`*.
- **Audited** — each declassification is a recorded event at a trust boundary
  (`62 §5`), and a declassification authority a dependency uses appears in its
  **`trusted_base_delta`** (`63`, `../20-verification/25 §3`). A package that
  downgrades secrets cannot hide it.
- **Optionally conditional** — a release policy ("may release an average only
  over ≥ k records") can be a `requires` obligation on the `declassify`, tying
  release to a *proven* precondition (`../20-verification/21 §1`).

This is what data-flow compliance wants: not "no secret ever moves" but "every
downgrade is explicit, authorised, conditional, and logged."

## 5. The guarantee: non-interference (the load-bearing statement)

A program that type-checks under the label discipline is **non-interfering** up
to its declassifications. Ken offers this at **two strengths**, and the
**observable differs per strength** — they are *not* conflated.

### 5.1 Low-equivalence (the relation the theorem is stated over)

Fix an **observer clearance** `ζ : ℒ.L`. Two values/states `v₁, v₂` are
**ζ-low-equivalent**, written `v₁ ≈_ζ v₂`, iff they agree on every sub-component
whose label `ℓ ⊑ ζ` (the ζ-*observable* parts are identical; parts with `ℓ ⋢ ζ`
may differ freely). Two inputs are "related at ζ" iff they differ only in their
`⋢ ζ` (secret-to-ζ) parts.

### 5.2 By typing — the automatic guarantee (strength 1, the default)

**Well-labeled ⇒ non-interfering**, as a **meta-theorem about the discipline**
(proved once over the elaboration, in the DCC / sealing-calculus tradition —
*named, not copied*). The programmer gets non-interference **for free from
passing the type-checker** — the scalable, default property.

**The theorem, with its exact preconditions (no over-claim):**

> For a program `c` that **type-checks** under §3, for **every** observer
> clearance `ζ` and any two inputs `i₁ ≈_ζ i₂`, the ζ-observable outputs are
> low-equivalent — `⟦c⟧(i₁) ≈_ζ ⟦c⟧(i₂)` — **(P3) up to declassification**
> (outputs of an authorised `declassify` along a permitted edge are exempted at
> their target label) and **(P4) progress-sensitively** — a crash or
> non-termination is itself a ζ-observable event, so if one run is ζ-observably
> done the other is too.

Preconditions, stated explicitly:

- **P1.** `c` type-checks — *all* §3 rules hold, **including the `pc` discipline
  for implicit flows** (`L-OBSERVE`/`L-SINK`). NI is **false** without the
  `pc`-join; it is a precondition, not decoration.
- **P2.** Parametric in any bounded `Lattice` (`OQ-ifc`) and **relative to the
  policy** (`65`) — a *wrong* policy yields a *wrong* guarantee (the `64 §4.1`
  spec≠intent analog; see §H).
- **P3.** Up-to-declassification — NI holds for the residual (non-declassified)
  flows; each `declassify` is an *intended* release.
- **P4.** **Progress-sensitive by default.** Termination-/crash-*insensitivity*
  is **opt-in only**, and the relaxation **shows in the four-way status**
  (`../20-verification/21 §5`) — a silent relaxation is itself a defect.

**The observable for strength 1 is elaboration accept/reject — not a verdict,
not a runtime two-run diff.** A `Secret→Public` flow is a **type error**
(reject); a clean program **accepts** (the `36 §1.4` EFFECT-ESCAPE flip).
NI itself **cannot** be an `ensures φ` (§1) and is **not authored as one**.

### 5.3 By proof — bespoke / quantitative claims (strength 2)

For finer statements — a specific declassification policy, a quantitative bound
("at most `n` bits leak"), the integrity of a named pipeline — the claim is a
**relational obligation** that **reduces to a *unary* obligation the kernel
re-checks** (`OQ-relational` DECIDED + narrowed). The reduction, as defensive
pseudocode:

```
-- Goal: prove c is non-interfering at observer ζ (a bespoke relational claim).
-- Reduce the 2-run statement to ONE unary obligation over a PRODUCT PROGRAM.

product(c, ζ):
  rename c into two copies  c¹, c²  over disjoint (primed / unprimed) variables
  Φ_pre   :=  lowEq_ζ(in¹,  in²)                 -- inputs agree on the ⊑ ζ parts
  Φ_post  :=  lowEq_ζ(out¹, out²)                -- outputs agree on the ⊑ ζ parts
           ∧  coterminates_ζ(c¹, c²)             -- (P4) progress-sensitive conjunct
  emit the UNARY obligation   Γ ⊢ (Φ_pre ⇒ Φ_post) : Ω      -- (22 §1; 21 §5)
  -- the prover (23) discharges it; the kernel RE-CHECKS the certificate
  --   (23 §1, ../10-kernel/18 §4) — same small kernel, no relational primitive
```

- **Product programs** are preferred over naïve self-composition for **solver
  tractability** (lock-stepping the copies exposes the relational invariant),
  per `OQ-relational`.
- The **`coterminates_ζ` conjunct encodes progress-sensitivity**: a program that
  diverges/crashes on one run of a ζ-equal pair fails the obligation. The
  termination-insensitive mode **drops** this conjunct and **flags it in the
  four-way status** (`21 §5`).
- A **first-class relational logic**, if ever needed, arrives as a **reflective
  deep embedding — never a kernel primitive** (the TCB does not grow, §H, ADR
  0004 Decision 3). The **heavy value-dependent product-program machinery is
  deferred** until a concrete case needs it — tagged **`[rel-deferred]`**,
  **named not faked** (a case needing it carries that reify-trigger, never a
  silent gap).

**Verdict mapping, pinned at the source (the strength-2 observable).** A
relational obligation's outcomes map to the V3 verdict trichotomy (`23 §1.2`)
**exactly** — closing the verdict-mapping silence at source (a distinguishing
pair could naively read as `unknown`; it is pinned `disproved`):

| Relational situation | Obligation | V3 verdict | Reading |
|---|---|---|---|
| `≈_ζ` inputs ⇒ `≈_ζ` outputs **provable** | discharged | `proved` | non-interfering; cert kernel-re-checked |
| a **distinguishing pair** exists (ζ-outputs differ) | refuted | `disproved` | **the distinguishing pair IS the leak-witness** (countermodel) |
| neither provable nor refutable by the prover | open | `unknown` | a **typed hole** in `trusted_base()` — honest; **never a false `proved`** (`23 §1.3`) |

So a leak surfaces as either a **type error** (strength 1) or a **`disproved`
verdict with a distinguishing-pair witness** (strength 2); a clean program
**accepts** / is **`proved`**; an undischarged relational claim is **`unknown`
with a hole** — the prover's honesty guard (`23 §1.3`) carries to the relational
domain, so a prover bug is a *weaker* verdict, never a false `proved`.

### 5a. Constant-time — the `@ct` *hook* (Sec1 lands the label, not the timing)

Constant-time (timing side-channel freedom) is a **2-safety** property, but Ken
does **not** prove it with the relational engine and does **not** own the
*timing guarantee* itself (hardware/codegen-relative — cache lines,
`cmov`-vs-branch lowering — `64 §4.2`). **Sec1 lands the source label + the
enforcement hook; the timing guarantee is delegated.**

- **A distinct, opt-in `@ct` label**, separate from `Secret` confidentiality.
  Confidentiality constrains *where the value goes*; `@ct` constrains *where the
  value's influence goes* (it may not **steer** a leakage-relevant operation). A
  value may be `Secret` without `@ct` (a PII field not to log, but branching
  on it leaks nothing to a timing adversary); crypto keys are both. By `OQ-ifc`
  lattice-parametricity, `@ct` is just **another product factor** of the lattice
  (§2.2) — **no metatheory cost**.
- **Leakage-relevant operations are a distinguished `Vis` op class** — a
  secret-dependent **branch guard**, **memory index**, or **variable-time
  primitive** (`36 §3.1`: "a label whose sink is a distinguished `Vis` op").
- **What Sec1 lands (the hook):** the `@ct` label **parses, attaches, and is
  carried** through the denotation (riding a `Vis` index like any label), and a
  `@ct` value reaching a leakage-sink `Vis` op is a **type error** — the §3
  `L-SINK` rule **reused** with the leakage-sink as `κ`. This unary-taint check
  is a **sound static enforcement of the source-level constant-time precondition
  `Q`** (the FaCT / ct-verif "secret-types" result — *named, not copied*), so it
  needs **no product programs**. A function exports a **signature-level CT
  promise** (constant-time in a named parameter) for boundary checking.
- **What Sec1 does NOT land (deferred + named, not silent):** the **timing
  guarantee itself** is delegated to **`[Ward]`** + toolchain — CT-preservation
  through compilation + empirical validation under a stated **leakage model** on
  a **platform** (`64 §4.2`, `63 §5a`). The **full enforcement/validation pass**
  is a separate WP, **`[Sec1ct]`**. Sec1 plants a **reify-trigger** at the
  leakage-sink op pointing to `[Sec1ct]` / `[Ward]` — the precondition is real
  and checked; the timing claim is honestly *not yet* Ken's to make.
- **No region construct.** The sensitive *range* is the `@ct` label's live span
  (intro → authorised `declassify`, §4); there is **no `constant_time { … }`
  block** (a padding-based balancer is a runtime *mitigation*, `Ward`'s, not
  Ken's).

## H. Honest limits — proven vs. assumed vs. delegated vs. deferred

Per `64 §4`: **a verified language that over-claims is
itself a security risk.** Ken states its IFC boundaries exactly. **None of this
enlarges the trusted kernel** — labels are surface/effect **indices on the
existing `ITree` container** (`36 §3.1`), the by-typing discipline elaborates to
the kernel's existing Π/Σ/inductive machinery, and relational certificates are
re-checked by the *same* small kernel (ADR 0004 Decision 3, ADR 0001).

| Aspect | Status | Detail |
|---|---|---|
| Well-typed ⇒ non-interfering (progress-sensitive, up-to-declassify) | **proven *by typing*, but the meta-theorem is trusted** | a **design-level meta-theorem** over the discipline (DCC/sealing tradition); **mechanization is a named future deliverable, not claimed done** — this is the security analog of metatheory-honesty |
| A specific relational / quantitative claim | **proven *by proof*, kernel-re-checked** | product program → unary obligation → V3 cert re-checked (`23 §1`, `18 §4`); **never a false `proved`** (`23 §1.3`) |
| The lattice / policy is the *right* policy | **assumed** | a wrong policy ⇒ a wrong guarantee — the `64 §4.1` spec≠intent analog; the policy (`65`) is the human-reviewed boundary |
| Classification at ingestion ("this datum *is* Tenant[X]") | **assumed — a claim, audited** | capability-gated + audited (§3.3), the dual of declassification; not a proof |
| Declassification | **authorised release, audited** | NI holds *up to* declassify; each downgrade is explicit, capability-gated, optionally conditional, and in `trusted_base_delta` |
| `@ct` **timing** guarantee | **delegated → `[Ward]` / `[Sec1ct]`** | Ken proves only the source-level precondition `Q`; the timing guarantee is codegen/hardware-relative under a stated leakage model (`64 §4.2`) |
| Heavy value-dependent relational machinery | **deferred → `[rel-deferred]`** | named, not faked; a reflective embedding if ever needed, **never** a kernel primitive |
| Kernel / FFI / native runtime | **trusted (listed)** | `64 §4.3`; the IFC discipline adds **no** trusted primitive |
| Worst-case time / space (DoS) | **out of scope** | `64 §4.2`; totality ≠ cheapness |

## 6. The topos grounding (why this is native, not bolted on)

A security label reads category-theoretically as an (idempotent) **modality** on
Ken's topos of values, and the labeled monad `T_ℓ` is the protection modality of
that level; non-interference is then a **relational/parametricity** statement
about maps respecting the modality. Ken's topos + dependent foundation makes
information flow an *instance of structure Ken already has* (modalities, the
subobject lattice), not a foreign type system stapled on — the design reason IFC
fits Ken better than a mainstream language. *(The precise modal/sheaf
formulation is **design-direction, tagged for the Verify enclave**; the
engineering above stands without it.)*

## 7. Worked examples

```
type Email   = String @ Secret[user]            -- PII, readable by `user` only
type ApiKey  = Bytes  @ Secret[server]          -- never client-visible

-- A type error: Secret PII reaching a Public log sink.
view audit (e : Email) : Unit  visits [Log Public]
  = log e            -- REJECTED: Secret[user] ⋢ Public   (L-SINK)

-- Allowed only via an authorised, conditional, audited downgrade.
view audit (e : Email) (d : Cap_declassify[Secret[user]→Public]) : Unit
  visits [Log Public]
  requires consented(e)                          -- release tied to a proof
  = log (declassify d (redact e))                -- explicit + logged + in delta

-- Integrity (taint): untrusted input may not reach a high-integrity sink.
view run (cmd : String @ Untrusted) : Unit  visits [Shell Trusted]
  = exec cmd         -- REJECTED: Untrusted ⋢ Trusted   (no command injection)

-- Implicit flow: branching on a secret raises pc; the public write is caught.
view leak (secret : Bool @ Secret) (s : Socket Public) : Unit  visits [Net]
  = if secret then send s 1 else send s 0
                     -- REJECTED: in each branch pc = ⊥ ⊔ Secret = Secret ⋢ Public  (L-OBSERVE→L-SINK)

-- @ct hook: a key may not steer a leakage-relevant operation.
view cmp (k : Bytes @ ct) (g : Bytes) : Bool
  = branch_on (k[0] == g[0]) …
                     -- REJECTED: @ct value steers a branch guard (a leakage sink); the source-level
                     --   precondition. Timing validation itself is [Ward]/[Sec1ct].
```

A CISO reads these and sees PII boundaries, API-key confinement, injection
resistance, implicit-flow closure, and a constant-time *precondition* as
**typed, compile-time** properties with an **explicit, audited** escape hatch —
the controls compliance frameworks (GDPR/CCPA data boundaries, PCI key handling,
"no untrusted data to `exec`") ask for, enforced by construction.

## 8. What is committed vs. open

- **Committed + decided (`OQ-ifc`):** the **lattice-parametric** discipline (NI
  proved once for any `ℒ`, §5.2); the **DLM** instance (§2.2: confidentiality
  = reader-sets, integrity = dual, levels as sugar, compartments as products);
  labeled types/effects **riding the `ITree` `Vis` index** (§3, `36 §3.1`); the
  flow-typing judgment with the **`pc` discipline** (§3.1); the **no-laundering
  invariant** (§3.2); **static type-index labels** by default with **first-class
  labels at audited boundaries** (§3.3); **declassification** as the sole,
  explicit, capability-gated, audited downgrade (§4); **non-interference** as a
  **meta-theorem** (observable = **accept/reject**, never a unary `ensures`);
  **no kernel enlargement** (§H). The concrete lattice/clearances/edges
  are a **separately-authored policy** (`65`, ADR 0007).
- **Decided + narrowed (`OQ-relational`, §5.3):** the by-proof relational mode
  **reduces to unary obligations the kernel re-checks** (product programs;
  reflective embedding only if a first-class logic is ever needed);
  **progress-sensitive default**, explicit termination-insensitive opt-out shown
  in the four-way status; **verdict mapping pinned at source** (distinguishing
  pair → `disproved`-with-witness; unprovable → `unknown`-with-hole); the heavy
  product-program machinery **deferred `[rel-deferred]`**.
- **Decided (`@ct`, §5a):** constant-time is a distinct **opt-in `@ct` label**
  whose values may never reach a leakage `Vis` sink (branch/index/var-time),
  enforced **by typing** (sound static enforcement of the source-level
  precondition, no product programs); the **timing guarantee** is delegated to
  **`[Ward]` + the toolchain** under a stated leakage model, and the full
  pass is **`[Sec1ct]`** — Sec1 lands the **label + hook + reify-trigger**,
  not the timing.
- **Honest-limits (§H):** proven (by typing, meta-theorem trusted) vs. proven
  (by proof, kernel-re-checked) vs. assumed (policy, ingestion) vs. delegated
  (`@ct` timing) vs. deferred (heavy relational) — stated exactly.

## 9. What WS-Sec must deliver here (Sec1)

The deliverable is the elaboration above, made implementation-ready. Each item
is a concrete, codeable section; an implementer builds from these and the kernel
re-checks the emitted core (the elaborator is **not** in the TCB):

1. **The label lattice** — the `Lattice` record interface (§2.1) and the **DLM
   instance** (§2.2: reader-sets, the dual integrity lattice, products, level
   sugar), with the lattice laws discharged once per instance. *Acceptance 1.*
2. **Labeled types + effects riding L5** — `A @ ℓ` as an erasable index; the
   label as a **`Vis` op/resp index** (§3, `36 §3.1`); the flow-typing judgment
   with the **`pc` discipline** and the four `L-*` flow rules (§3.1); the
   **no-laundering invariant** (§3.2); boundary classification (§3.3).
   *Acceptance 1, 2.*
3. **Declassification** — `declassify` as capability-gated, explicit, audited,
   optionally `requires`-conditioned, surfaced in `trusted_base_delta` (§4).
   *Acceptance 1.*
4. **Non-interference** — the §5.1 low-equivalence relation; the **by-typing**
   meta-theorem with **exact preconditions** (§5.2, observable = accept/reject);
   the **by-proof** product-program reduction to a unary kernel-re-checked
   obligation with the **verdict mapping pinned at source** (§5.3,
   progress-sensitive default). *Acceptance 1, 3.*
5. **The `@ct` hook** — the opt-in label parses + is carried; a `@ct` value
   reaching a leakage-sink `Vis` op is a type error; the **reify-trigger** to
   **`[Sec1ct]` / `[Ward]`** is present, not silent (§5a). *Acceptance 4.*
6. **Honest limits** — the proven/assumed/delegated/deferred boundary as a
   first-class artifact, no over-claim (§H). *Acceptance 5.*

**Level reconciliation (the soundness check — spec-author duty before Architect
handoff).** The labeled constructs add **no new level rule** — only instances of
existing formation (`36 §7.4`):

| Construct | Level | Rule |
|---|---|---|
| `Lattice` (the carrier + ops record) | `Type (suc ℓ)` | record / Σ-Form (`13 §1`), laws at `Ω` |
| `A @ ℓ` (labeled type) | same as `A` | `ℓ` is an **erasable index**; the kernel sees `A` |
| label index on a `Vis` op/resp | `≤ ℓ_ITree` | rides the existing `Vis` container (`36 §2.1`); no new universe |
| relational obligation `Γ ⊢ (Φ_pre ⇒ Φ_post) : Ω` | `Ω` | an ordinary unary obligation (`22 §1`, `21 §5`) |

Every level is the **predicative `max`** of its parts (`12 §2`), non-cumulative
(`12 §3`); the elaborator emits explicit levels and the kernel re-checks them
(`12 §4`). The IFC discipline is impredicative nowhere — it adds **no new level
rule**, only Π/Σ/inductive/`ITree` instances.

Conformance: `../../conformance/security/ifc/` — covering the five acceptance
criteria with **discriminating** cases (COORDINATION §7; each negative case must
**flip** on the bug it targets): a `Secret`→`Public` flow rejects and an
authorised `declassify` accepts + shows in the delta (AC1); a label routed
through an effect to a low sink **still rejects** under no-laundering
and **wrongly accepts under a label-dropping `bind`/`incl`** (AC2, §3.2); a
forged label/cert is **kernel-rejected** (AC3); the `@ct` hook parses,
carries, and rejects a leakage-sink use, with the `[Sec1ct]`/`[Ward]` trigger
present (AC4); the proven-vs-assumed boundary is explicit (AC5).
