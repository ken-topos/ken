# Information-flow control

> Status: **Sec1 + Sec1ct elaborated** — implementation-ready for WS-Sec.
> **Normative for the label lattice, the flow-typing discipline, the
> non-interference statement and its honest limits, and the `@ct` constant-time
> discipline** (§2–§5a, §H) — Sec1ct elaborates §5a from the Sec1 *hook* to the
> enforced `@ct` discipline (the `CT` axis, the sealed `LeakSink` set, the
> `L-CT-SINK` rule, the CT-promise/`P` export, declassify-ends-span); the
> concrete surface *spelling* of labels stays proposal-level (`OQ-syntax`).
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

**The reduction is itself a trusted step (N2).** The kernel re-checks the
*certificate* for **whatever obligation it is handed** — **not** that the
obligation **faithfully encodes 2-safety**. The product-program construction
(the renaming, the `lowEq_ζ` / `coterminates_ζ` encoding) is done by the
**untrusted** verifier, so a **wrong reduction** — a too-weak `Φ_post`, a
silently-dropped `coterminates_ζ` — yields a **kernel-valid cert for a non-NI
claim**: a **false `proved` the kernel cannot catch** (AC3's forged-cert
reject does *not* cover a **correct cert for an unsound obligation**). So
**reduction-faithfulness is the trusted part** of the by-proof path; its sole
backstops are (a) the reduction's own soundness argument and (b) a
**positive-soundness conformance case — a known-interfering program must
reduce to `disproved`** (the reduction cannot be massaged to make a leak look
`proved`). This is the exhaustiveness-as-sole-backstop discipline (a
producer's *omission* is invisible to a re-checker) carried to the relational
domain.

So a leak surfaces as either a **type error** (strength 1) or a **`disproved`
verdict with a distinguishing-pair witness** (strength 2); a clean program
**accepts** / is **`proved`**; an undischarged relational claim is **`unknown`
with a hole** — the prover's honesty guard (`23 §1.3`) carries to the relational
domain, so a prover bug is a *weaker* verdict, never a false `proved`.

### 5a. Constant-time — the `@ct` discipline (by typing; timing delegated)

> **Sec1 landed the `@ct` *hook* (parse, attach, carry, reject a
> leakage-sink flow); Sec1ct elaborates it to the enforced *discipline*
> below.** Constant-time (timing side-channel freedom) is a **2-safety**
> property, but Ken enforces only the **source-level precondition** by
> **unary taint-typing** — **not** the relational engine, **no product
> programs** (the FaCT / ct-verif "secret-types" result — *named, not
> copied*) — and Ken does **not** own the *timing guarantee* itself
> (hardware/codegen-relative — cache lines, `cmov`-vs-branch lowering —
> `64 §4.2`), which is **delegated to `[Ward]` + the toolchain**. The
> split is fixed (§H): Ken proves the precondition `Q`; Ward proves the
> binary.

`@ct` reuses Sec1's lattice, `pc` discipline, declassify, and labeled effects —
**extended, not rebuilt**. The discipline is six parts.

#### 5a.1 The `@ct` lattice axis (a distinct product factor)

`@ct` rides the existing lattice (`OQ-ifc` lattice-parametricity, §2.2) as a
**distinct product factor** `CT` of the full label
`(Conf × Integ × …compartments × CT)` — ordered **componentwise** (§2.2), so the
`@ct` axis is **independent** of confidentiality and integrity (a value may be
`Secret` without `@ct` and vice versa; a crypto key is both). `CT` is the
**two-point lattice**:

```
CT = { ct⊥, ct⊤ }      ct⊥ ⊑ ct⊤        -- one non-trivial edge
  ct⊥  = not-timing-sensitive (its influence may steer a leakage op)   -- "@ct-bottom"
  ct⊤  = @ct = timing-sensitive (its influence may NOT steer one)
  _⊔_  = ∨   (any @ct input ⇒ @ct result; join is the OR — a value computed
              from a @ct value is itself @ct; you cannot compute @ct away)
  _⊓_  = ∧
  an UN-annotated value is  ct⊥  (the default: code is timing-agnostic until opted in)
```

**Orientation (pin both directions — the `[Sec1-dual]` discipline).** `@ct` is a
**taint** axis: it is the **order-dual of confidentiality**, oriented exactly
like **integrity** (§2.2) — `⊥` is *safe* (low taint), `⊤` is *the taint*, and a
**sink demands `ct⊥`**. Concretely: `@ct` joins **upward** by `L-COMBINE` (its
influence spreads to anything computed from it), and a leakage sink carries
clearance `κ.ct = ct⊥`, so a `@ct` value (`ct⊤`) reaching it satisfies
`ct⊤ ⋢ ct⊥` → **reject**. Getting the order *backwards* (confidentiality-style,
sink demands `⊤`) would **silently invert** accept/reject — so both directions
are stated, and the **discriminating conformance net** (§5a.3, AC4) is the
non-degenerate distinguishing pair that holds the orientation.

**Surface.** `A @ ct` rides the existing labeled-type production
`type "@" label`, with `label ::= … | "ct"` (`../30-surface/32 §`,
`[OQ-syntax]`). The **value-set (`{ct⊥, ct⊤}`) and the invariants above
are locked**; the
**literal surface token spelling `@ct` is `(oracle)`-tagged** — the surface
grammar is proposal-level (`OQ-syntax`), so the token is the deferred degree of
freedom, the concept/value-set is not (assert-at-locked-granularity).

#### 5a.2 The leakage-sink classification (a sealed set, no catch-all)

The leakage-relevant operations are a **distinguished `Vis` op class** (`36
§3.1`: "a label whose sink is a distinguished `Vis` op"), enumerated as a
**sealed sum** — **exactly three**, each with its exact trigger:

```
LeakSink =                                    -- a sealed Vis-op classification; NO `_ => non-sink`
  | BranchGuard    -- the SCRUTINEE of a control-flow branch that lowers to a machine
  |                --   branch/`match`; trigger: a non-ct⊥ discriminant steers `if`/`match`
  | MemIndex       -- a DATA-DEPENDENT memory/array index feeding an indexing Vis op;
  |                --   trigger: a non-ct⊥ index expression (data-dependent cache access)
  | VarTimePrim    -- a primitive whose run time depends on operand VALUE (naive bignum
                   --   compare/divide, non-CT `==`); trigger: a non-ct⊥ operand reaches a
                   --   primitive flagged `var-time` in the effect signature (`36 §3.1`)
```

**Exhaustive-by-construction (COORDINATION §7; the omission-hole discipline).**
There is **no `_ => non-sink` catch-all**. Adding an effect/`Vis` op that
can leak timing **without** classifying it into `LeakSink` must be a
**compile error** (an exhaustiveness obligation over the `Vis`-op
alphabet), **never** a silent
non-sink. This is load-bearing because labels are **erased** before the kernel
(§9 N1): a *silent omission* — a new leaky op defaulting to non-sink — is
**invisible** to the kernel re-check, so the classification's totality is the
sole structural guard against an un-netted leak.

#### 5a.3 The flow rule (`L-CT-SINK` — a `pc`-aware instance of `L-SINK`)

One rule, a **projection of `L-SINK` (§3.1) onto the `CT` factor**, reusing
Sec1's `pc` discipline verbatim:

```
(L-CT-SINK)  Γ; pc ⊢ g : A @ ℓ        o : LeakSink   (BranchGuard | MemIndex | VarTimePrim)
             g is o's TIMING-RELEVANT operand  (the guard / the index / the var-time operand)
             ──────────────────────────────────────────────────────────────────────────────
             o(g, …)  WELL-FORMED iff  (ℓ.ct ⊔ pc.ct) = ct⊥
                                       else  IFC-CT static error
                                             -- name ℓ.ct, pc.ct, and the sink site (a source
                                             --   location), per L-SINK's diagnostic contract
```

- **`pc.ct` closes the implicit channel.** Branching on a `@ct` value raises
  `pc.ct = ct⊤` in both branches (`L-OBSERVE` projected onto the `CT`
  factor), so a leakage op **inside** a `@ct`-guarded branch is caught **even
  when its own operand is `ct⊥`** — the implicit-flow case, identical to
  Sec1's `L-OBSERVE → L-SINK` composition.
- **Each path is backed by a reject case:** `BranchGuard` (AC1), `MemIndex`
  (AC2), `VarTimePrim` (AC3) — each a verdict **flip** (the correct program with
  no such steering **accepts**).
- **The observable is elaboration accept/reject — *never* a V3 verdict.**
  `@ct` does **not** route through the by-proof engine (§5.3): a `@ct` leak
  is a **type error** (reject), a clean program **accepts**, full stop. *Do
  not* map a `@ct` leak to `disproved`/`unknown` — that strength-2 trichotomy
  is a different mode (verdict-mapping pinned at source, the §5.3 discipline
  applied to foreclose the cross-mode silence).

#### 5a.4 The CT signature promise + the `P` boundary obligation

A function **declares** constant-time-in-a-named-parameter — a
**signature-level CT promise** on parameter `x` — and the boundary **checks** it
and **emits** it as a source-level boundary obligation on `P` (tagged `tested`):

- **Check (by typing, no product program).** The body is checked with `x` bound
  at `ct⊤`; if any `LeakSink` op's timing-relevant operand depends on `x` —
  directly or via `pc.ct` — the function is **rejected**. A body that passes
  **discharges** the promise; this *is* the §5a.3 rule applied with the named
  parameter as the `@ct` source.
- **Export to `P` (the assumption boundary), tagged `tested`.** An accepted
  CT-promising function emits a **source-level boundary obligation** onto the
  `71` **`assumptions` (`P`)** channel, **tagged `tested`** — "constant-time-in-
  `x` *at the source level*, relative to the stated leakage model" —
  content-hashed into the `71` assume-guarantee contract. It is **not** a
  `guarantees`/`Q` entry: B1 is the authority, and the projection's honesty
  discriminator (`70-behavioral/71 §2.1`) reserves `Q` for **kernel-certified**
  claims (a certificate the kernel re-checks, its goal **absent** from
  `trusted_base()`). The CT promise is **proved by trusted typing** — the
  `L-CT-SINK` rule is a trusted meta-theorem and `@ct` labels are **erased
  before the kernel** (§9 N1), so it is **never kernel-re-checked** — a
  **trusted-by-typing boundary obligation**, which rides `P`/`tested`, the
  safe direction. *(Whether trusted-by-typing eventually earns a distinct
  epistemic status is a separate, deferred question — not reopened here; the
  `P`/`tested` routing holds regardless.)* The loop still closes at the
  **`63 §5a` discharge attestation**: its field #1 binds the `71` contract hash,
  and its field #4 carries **Ward's** timing-validation result for exactly this
  obligation.
- **Coordinate the shape with B1; do not pre-bind names
  (defer-spelling-not-concept).** **Locked:** the *concept* (a CT promise is a
  `P`-channel boundary obligation, tagged `tested`, naming the parameter), the
  *value-set + invariants* (it is a **source-level** precondition, **not** a
  timing guarantee; it pairs
  **1:1** with a `63 §5a` Ward discharge result; it is relative to a stated
  leakage model), and the *stability discipline* (the clause binds to the `71`
  contract **content-hash** — **renaming the field after binding is a contract
  break**, `63 §5a` #1). **`(oracle)`-tagged:** the **literal field-token
  spelling** of the CT-promise clause — the B1 / `71` emitter binds it.

#### 5a.5 Declassify ends the span (the sole terminator)

The sensitive **span = the `@ct` label's live span** — introduced where a value
becomes `@ct`, ended **only** by an authorised **`declassify`** (§4), reused as
the sole terminator on the `CT` axis:

```
declassify : Cap_declassify[ct⊤ → ct⊥] → A @ ct⊤ → A @ ct⊥      -- requires ct⊥ ⊑ ct⊤ + the cap
```

- After an **authorised** declassify the value is `ct⊥`, so its influence into a
  leakage sink is **unconstrained** (`(ct⊥ ⊔ pc.ct)` — accepted when `pc.ct =
  ct⊥`). The downgrade asserts "this value is now safe to leak timing-wise" (it
  was blinded, or is public after a CT operation completed) — an **intended**
  release.
- **Capability-gated, explicit, audited, in `trusted_base_delta`** —
  identical to confidentiality declassify (§4): the `@ct` declassify authority
  a dependency holds appears in its delta (AC5).
- **No region construct.** There is **no `constant_time { … }` block** — the
  span *is* the label lifecycle. (A padding/balancing region is a *runtime
  mitigation*, `Ward`'s, not Ken's.)

#### 5a.6 What this does and does **not** prove (honest split → §H)

The §5a.3 check, projected to the `@ct` observer, gives the **source-level
constant-time precondition** `Q` (no `@ct` value steers a `LeakSink`) — **not**
constant-time *execution*. The execution guarantee additionally requires (a) the
compiler **preserves** the source CT property through lowering, and (b) the
hardware **honours** the assumed **leakage model** — both **`[Ward]`**'s
(`64 §4.2`, `63 §5a`). Claiming "well-typed ⇒ constant-time execution" is the
exact over-claim §H forecloses. And because `@ct` labels are **erased** before
the kernel (§9 N1), the `L-CT-SINK` rule and the `LeakSink` classification are
**trusted** — the **discriminating conformance corpus**, not the kernel, is the
net (§H names the three kernel-blind surfaces as scoped work).

## H. Honest limits — proven vs. assumed vs. delegated vs. deferred

Per `64 §4`: **a verified language that over-claims is
itself a security risk.** Ken states its IFC boundaries exactly. **None of this
enlarges the trusted kernel** — labels are surface/effect **indices on the
existing `ITree` container** (`36 §3.1`), the by-typing discipline elaborates to
the kernel's existing Π/Σ/inductive machinery, and relational certificates are
re-checked by the *same* small kernel (ADR 0004 Decision 3, ADR 0001).

| Aspect | Status | Detail |
|---|---|---|
| Well-typed ⇒ non-interfering (progress-sensitive, up-to-declassify) | **proven *by typing*, but the meta-theorem AND the flow rules are trusted** | a **design-level meta-theorem** over the discipline (DCC/sealing tradition); **mechanization is a named future deliverable, not claimed done** — the security analog of metatheory-honesty. **The kernel backstops core type-safety only; labels are erased, so the elaborator's flow rules are trusted** (§9 N1) — conformance, not the kernel, nets a flow bug |
| A specific relational / quantitative claim | **proven *by proof* — cert kernel-re-checked, *reduction* trusted** | product program → unary obligation → V3 cert re-checked (`23 §1`, `18 §4`); `proved` is honest **for the handed obligation** (`23 §1.3`), but **the reduction's faithfulness to 2-safety is trusted** (§5.3 N2) — cert-re-check ≠ reduction-faithfulness; positive-soundness AC: interfering → `disproved` |
| The lattice / policy is the *right* policy | **assumed** | a wrong policy ⇒ a wrong guarantee — the `64 §4.1` spec≠intent analog; the policy (`65`) is the human-reviewed boundary |
| Classification at ingestion ("this datum *is* Tenant[X]") | **assumed — a claim, audited** | capability-gated + audited (§3.3), the dual of declassification; not a proof |
| Declassification | **authorised release, audited** | NI holds *up to* declassify; each downgrade is explicit, capability-gated, optionally conditional, and in `trusted_base_delta` |
| `@ct` source-level CT precondition (no `@ct` value steers a `LeakSink`) | **proven *by typing* — but the flow rule is trusted** | the `L-CT-SINK` check (§5a.3) statically enforces the precondition by unary taint, no product program; but `@ct` labels are **erased** before the kernel, so the rule + the `LeakSink` classification are **trusted** (§9 N1) — the discriminating conformance corpus, not the kernel, is the net; exported on `P`/`tested`, not `Q` (§5a.4) |
| `@ct` **timing** guarantee (constant-time *execution*) | **delegated → `[Ward]` / toolchain** | Ken proves only the source precondition (above, exported on `P`/`tested`); the timing guarantee is codegen/hardware-relative under a stated leakage model and needs CT-preserving lowering + empirical validation (`64 §4.2`, `63 §5a`) — **Ken must not claim it** |
| Heavy value-dependent relational machinery | **deferred → `[rel-deferred]`** | named, not faked; a reflective embedding if ever needed, **never** a kernel primitive |
| Kernel / FFI / native runtime | **trusted (listed)** | `64 §4.3`; the IFC discipline adds **no** trusted primitive |
| Worst-case time / space (DoS) | **out of scope** | `64 §4.2`; totality ≠ cheapness |

**Named scoped work (the kernel-blind surfaces — reify-triggers for the next
increment).** Three flow-property surfaces are **trusted** today (the kernel
does **not** see them, §9 N1) and netted only by the conformance corpus + the
design-level argument; each is named here as scoped work, **not silently
absorbed**:

- **`[Sec1-dual]` — IntegLabel (and `@ct`) ordering.** The integrity axis (§2.2)
  and the `@ct` axis (§5a.1) are **both** the order-dual of confidentiality
  (taint-oriented: `⊥` safe, `⊤` the taint, sink demands `⊥`). A flipped order
  **silently inverts** accept/reject. The orientation is asserted in prose and
  netted by the **non-degenerate discriminating pair** (`@ct` rejects *while*
  `Secret`-not-`@ct` accepts on the same branch, AC4); **mechanizing** the
  dual-ordering check is the scoped follow-up.
- **`[Sec1-launder]` — real `Vis`-routed label preservation.** The
  no-laundering invariant (§3.2) — that `bind`/`incl`/handler routing
  preserves the label index (including the `CT` factor) on **every** `Vis`
  node — rests today on the **L5
  contract** (`36 §3.1`) as a *design* guarantee; verifying the **actual
  elaborator's** `Vis`-routing drops/lowers no index at a boundary is the scoped
  follow-up (the AC2 label-dropping flip is its conformance net).
- **`[Sec1-reduce]` — `Φ_post` reduction-faithfulness.** The by-proof
  reduction's `Φ_post` faithfully encoding 2-safety is **trusted** (§5.3 N2).
  `@ct` uses
  **unary taint, not** the relational engine, so this trigger does **not** gate
  Sec1ct's `@ct` path — it names the general reduction-faithfulness surface for
  the **strength-2** increment, carried here so it is tracked, not absorbed.

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

-- @ct: a key may not STEER a leakage-relevant operation (BranchGuard sink).
view cmp (k : Bytes @ ct) (g : Bytes) : Bool
  = branch_on (k[0] == g[0]) …
                     -- REJECTED (L-CT-SINK): @ct value (ct⊤) steers a BranchGuard sink; ct⊤ ⋢ ct⊥.
                     --   Source-level precondition only; timing validation itself is [Ward].

-- AC4 — the axes are INDEPENDENT: Secret-but-not-@ct branches freely (a verdict
--   FLIP vs cmp above on the same branch shape — not green-vs-green).
view route (p : Tag @ Secret) (g : Bytes) : Resp
  = if p == Admin then handleA else handleB
                     -- ACCEPTED: p is Secret (confidentiality) but ct⊥ — branching on it leaks
                     --   nothing to a TIMING adversary. Conf and CT are orthogonal product factors.

-- AC5 — declassify ends the @ct span: after an AUTHORISED downgrade the formerly-
--   @ct value steers freely (and the @ct declassify cap shows in trusted_base_delta).
view cmp_ok (k : Bytes @ ct) (d : Cap_declassify[ct⊤→ct⊥]) (g : Bytes) : Bool
  = let k' = declassify d (ct_eq k g)   -- k' : Bool @ ct⊥  (blinded/CT-compared, now safe to act on)
    in branch_on k' …
                     -- ACCEPTED: k' is ct⊥ post-declassify; span ended. (Same sink as cmp; the only
                     --   difference is the authorised, audited downgrade — a real flip.)

-- AC6 — CT-in-parameter signature promise: declared CT-in-`k`, checked by typing.
view ct_eq (k : Bytes @ ct) (g : Bytes) : Bool @ ct   -- promises constant-time-in-k
  = fold_and (map2 ct_byte_eq k g)
                     -- ACCEPTED + emits P/tested("constant-time-in-k", source-level) onto the 71
                     --   assumptions channel; a body that did `branch_on k[0]` instead is REJECTED (flip).
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
- **Decided (`@ct`, §5a) — elaborated to the enforced discipline (Sec1ct):**
  constant-time is a distinct **opt-in `@ct` label**, the order-dual `CT`
  product factor of the lattice (§5a.1), whose values may never **steer** a
  leakage `Vis` sink — the **sealed** `LeakSink` set
  (`BranchGuard`/`MemIndex`/`VarTimePrim`,
  §5a.2, no catch-all) — enforced **by typing** via the `pc`-aware `L-CT-SINK`
  rule (§5a.3, sound static enforcement of the source precondition, no product
  programs); a function exports a **CT-in-parameter promise** onto the `71` `P`
  (`tested`) channel (§5a.4); the span ends **only** at an authorised
  `declassify` (§5a.5).
  The **timing guarantee** stays delegated to **`[Ward]` + the toolchain** under
  a stated leakage model (Sec1 landed the **label + hook**; Sec1ct landed the
  **discipline**; `[Ward]` owns the **timing**).
- **Honest-limits (§H):** proven (by typing, meta-theorem trusted) vs. proven
  (by proof, kernel-re-checked) vs. assumed (policy, ingestion) vs. delegated
  (`@ct` timing) vs. deferred (heavy relational) — stated exactly.

## 9. What WS-Sec must deliver here (Sec1)

The deliverable is the elaboration above, made implementation-ready. Each item
is a concrete, codeable section; an implementer builds from these and the kernel
re-checks the emitted core **for core type-safety** (the elaborator is **not**
in the TCB *for type-safety* — but see the two-soundnesses note below for the
erased **flow** property):

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
5. **The `@ct` discipline** (Sec1 landed the *hook*; **Sec1ct** elaborates the
   *discipline*, §5a) — the opt-in `CT` product-factor axis (§5a.1); the
   **sealed** `LeakSink` classification, no catch-all (§5a.2); the `pc`-aware
   `L-CT-SINK` flow rule (§5a.3); the CT-in-parameter promise + `P`/`tested`
   boundary obligation (§5a.4); declassify as the sole span terminator (§5a.5);
   the timing
   guarantee delegated to
   **`[Ward]`** with the three `[Sec1-*]` triggers named in §H (§5a.6).
   *Acceptance 4; Sec1ct AC1–AC7.*
6. **Honest limits** — the proven/assumed/delegated/deferred boundary as a
   first-class artifact, no over-claim (§H). *Acceptance 5.*

**The two soundnesses — what the kernel does and does *not* backstop (N1).**
The kernel re-check backstops **core type-safety only**. IFC labels are
**erased** before the kernel (§3: at the kernel a labeled value *is* `A`), so
a **flow-typing bug** — a wrong `⊑` in `L-SINK`, a dropped `pc`-join, a
label-laundering `bind`/`incl` — emits a **well-typed** core term the kernel
**accepts** while non-interference is **violated**. So **IFC-by-typing's flow
rules ARE trusted** (consistent with §H row 1): the by-typing discipline's
**only** backstops are the **§H-trusted meta-theorem** + the **discriminating
conformance corpus** (§8) — **never the kernel**. This is the security analog
of the verification layer's two-soundnesses (a wrong or *omitted* obligation
reads as verified because the kernel checks only what it is *handed*); it is
why §8's flow cases must **flip** — a label-dropping `bind`/`incl` must
*wrongly accept* — so conformance, not the kernel, is the net under the erased
discipline.

**Level reconciliation (the soundness check — spec-author duty before Architect
handoff).** The labeled constructs add **no new level rule** — only instances of
existing formation (`36 §7.4`):

| Construct | Level | Rule |
|---|---|---|
| `Lattice` (the carrier + ops record) | `Type (suc ℓ)` | record / Σ-Form (`13 §1`), laws at `Ω` |
| `A @ ℓ` (labeled type) | same as `A` | `ℓ` is an **erasable index**; the kernel sees `A` |
| label index on a `Vis` op/resp | `≤ ℓ_ITree` | rides the existing `Vis` container (`36 §2.1`); no new universe |
| `CT` factor `{ct⊥, ct⊤}` (§5a.1) | level `0` | a finite two-point lattice — **another product factor** (§2.2), an *instance* of the `Lattice`/product rules above; **no new level rule** |
| `A @ ct` (the `CT` projection of `A @ ℓ`) | same as `A` | `ct` is an **erasable index** like any label; the kernel sees `A`; `LeakSink` is a `Vis`-op classification, not a type former |
| relational obligation `Γ ⊢ (Φ_pre ⇒ Φ_post) : Ω` | `Ω` | an ordinary unary obligation (`22 §1`, `21 §5`) |

The `Vis` label-index row carries the side-condition **`ℓ_carrier ≤ ℓ_ITree`**
on the parametric `Lattice` (trivially true for DLM's finite `Set Principal`
carrier at level 0): a label index **never raises** the ITree universe, so a
high-carrier instance must place its label at or below `ℓ_ITree`.

Every level is the **predicative `max`** of its parts (`12 §2`), non-cumulative
(`12 §3`); the elaborator emits explicit levels and the kernel re-checks them
(`12 §4`). The IFC discipline is impredicative nowhere — it adds **no new level
rule**, only Π/Σ/inductive/`ITree` instances.

Conformance (Sec1): `../../conformance/security/ifc/` — covering the five
Sec1 acceptance criteria with **discriminating** cases (COORDINATION §7; each
negative case must **flip** on the bug it targets): a `Secret`→`Public` flow
rejects and an authorised `declassify` accepts + shows in the delta (AC1); a
label routed through an effect to a low sink **still rejects** under
no-laundering and **wrongly accepts under a label-dropping `bind`/`incl`**
(AC2, §3.2); a forged label/cert is **kernel-rejected** (AC3); the `@ct` hook
parses, carries, and rejects a leakage-sink use, with the `[Sec1ct]`/`[Ward]`
trigger present (AC4); the proven-vs-assumed boundary is explicit (AC5).

**Conformance (Sec1ct, the `@ct` discipline):** extends `ifc/` (or a `ct/`
sibling) covering §5a. Because `@ct` labels are **erased before the kernel**
(N1), these cases — not the kernel — are the trust boundary, so each must
**route a real `@ct` value through a real `LeakSink`** and observe the verdict
(never *predicate about* a synthetic `is_ct`/`is_sink` over literals — that
guards nothing). **Per-case flip:** a `@ct` value steering a `BranchGuard`
rejects / the same shape on a `ct⊥` value accepts (AC1); likewise `MemIndex`
(AC2) and `VarTimePrim` (AC3); a `Secret`-but-not-`@ct` value branches freely —
**accepts** — *while* the `@ct` case rejects on the same branch shape (AC4, the
non-degenerate `[Sec1-dual]` distinguishing pair, not green-vs-green); after an
authorised `declassify` the formerly-`@ct` value steers a sink and **accepts**,
its declassify cap in `trusted_base_delta` (AC5); a CT-in-parameter promise is
checked and the accepted function **emits the `P`/`tested` boundary-obligation
clause** — a *structural* assertion on the emitted boundary obligation, not
merely "accepts" (AC6); and an
**honesty** case asserts **no** test claims Ken proves constant-time *execution*
— §H carries the split + the three `[Sec1-*]` triggers (AC7). **Cross-case
sweep:** the `@ct`-reject class agrees (AC1–AC3), the accept class agrees
(AC4–AC6), and **every** `@ct` observable is **accept/reject — never** a V3
verdict (§5a.3): a case mapping a `@ct` leak to `disproved`/`unknown` is a
mode-confusion bug.
