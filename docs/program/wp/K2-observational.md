# K2 — Observational equality layer (Eq / cast / Ω / quotients / truncation) — Steward frame

> **Owner:** Team Kernel · **Size:** L · **Risk:** ★★★ (the **ADR-0005
> headline** — the highest-rigor, highest-risk kernel WP; where any feasibility
> doubt surfaces) · **Branch:** `wp/K2-observational` · **Deps:** **K1 done**
> (`fe1ead1`) · **Feeds:** K2c (decidable conversion), K-api, V2/V3 (the prover
> re-checks against this). On the critical path: K1 → **K2** → K2c → K-api.
>
> **Pipeline status: Steward frame → _awaiting spec-leader elaboration_ → Team
> Kernel.** Per the WP release process (steward playbook §2c) the **spec-leader
> elaborates** `spec/10-kernel/15-identity` + `16-observational` + the
> `conformance/kernel/observational/` corpus to full team-ready rigor on this
> branch **before** Team Kernel is released. Kernel must **not** start until the
> elaborated package is on `main` and the WP is kicked off.

## 0. Scope — the observational layer on top of K1 (spec 15–16)

K1 reserved the grammar (`Ω`, `Eq`, `cast`, `J`, `A/R`, `‖A‖` are `[K2]` formers
that K1's `check`/`infer` reject) and left a clean `convert` seam. K2 makes them
**type and compute**, extending K1's `check`/`infer`/`whnf`/`conv`. Deliver:

1. **The strict-proposition universe Ω (`16 §1`).** `Ω` with **definitional
   proof-irrelevance** (any two `p, q : P : Ω` are `≡`), predicative (`Ω : Type
   1`, level-poly `Ω ℓ`), carrying the **Heyting** structure (`⊤`/`⊥`/`∧`/`⇒`/`¬`;
   `∨`/`∃` via truncation). Conversion may **skip the contents of propositional
   arguments** (the proof-irrelevance shortcut).
2. **Observational equality `Eq A a b : Ω` (`15`, `16 §2`),** **computed by
   recursion on the type `A`**: Π → pointwise (`funext` *definitional*); Σ →
   componentwise (with a `cast` on the second component); inductive → structural
   (same ctor ⇒ ∧ of arg-equalities, distinct ctors ⇒ `⊥`); quotient → the user
   relation `R a b`; `Type ℓ` → **structural** type-equality (NOT univalence); Ω
   → mutual implication (`propext` *definitional*); primitive → literal equality;
   neutral `A`/args → neutral. `refl a : Eq A a a`.
3. **`cast` / transport (`16 §3`).** `cast A B (e : Eq Type A B) a : B` with
   **`cast A A refl a ≡ a` (regularity)** and **`cast`-by-type** computation
   (push into Π domain/codomain, Σ components, inductive ctor args, quotient
   classes); neutral on neutral type-equality. The eliminator **computes from the
   endpoints and never inspects the equality proof** (the canonicity-friendly
   property).
4. **Derived `J` + the rest (`15 §4`, `16 §4`).** `J` defined from `cast`, with
   **`J-β` on `refl`** AND **reduction on non-`refl`** equalities (the headline —
   `J` on a non-`refl` canonical proof computes to a constructor form, not stuck);
   `subst`, `sym`, `trans`, `cong` derivable + computing; definitional
   **funext/propext/UIP**.
5. **Set-quotients `A / R` (`16 §5`).** Formation (`R : A → A → Ω`), intro `[a]`,
   `Eq (A/R) [a] [b] ≡ R a b`, and the **respect-checked eliminator**
   (`elim … [a] ≡ f a`; respect proof free when the target is in Ω).
6. **Propositional truncation `‖A‖ : Ω` (`16 §6`),** `|a| : ‖A‖`, map-into-a-prop
   eliminator; backs Ω's `∨`/`∃`.

Deliverable: the observational layer in the `ken-kernel` crate + property/
conformance tests. Extends — does not rewrite — K1's `check`/`infer`/`whnf`/`conv`.

## 1. Settled decisions — FIXED inputs (do not reopen)

| Decision | Resolution (source) |
|---|---|
| Equality discipline | **Observational (OTT)** per `TTobs`/`CICobs`. **No interval, cofibration, `transp`/`hcomp`/`comp`, `Glue`, computational univalence, `PathP`, or HITs** — none of the cubical layer. (ADR 0005; `16 §7`) |
| `J` on non-`refl` | **MUST reduce** (via `cast`), not only on `refl`. A conformance test (`observational/j-nonrefl`) **fails any kernel that only reduces `J` on `refl`**. (`15 §4`) |
| Level | **Set-level** — UIP/funext/propext **definitional**; **no univalence**, no higher path structure (`Eq (Eq …)` has no nontrivial content). (`15 §6`, `16 §4`) |
| Ω | **Predicative**, definitionally proof-irrelevant, Heyting (intuitionistic, not Boolean; no ambient excluded middle). `OQ-Prop` DECIDED: impredicativity ruled out. (`16 §1`) |
| Type-equality | **Structural** (same head former, equal parts) — explicitly **not** `(A≃B)→Eq Type A B`. (`16 §3`) |
| `cast` | **`cast A A refl a ≡ a`** (regularity) + by-type computation. (`16 §3`) |
| Quotients | **Set-quotients** (native, not HITs); general **QITs are deferred** (blueprint). (`16 §5`) |
| Reference use | Exact reduction normal forms are **(blueprint)** confirmed against `CICobs`/`TTobs` — **enclave-only, behaviour not source; never copied** (CLEAN-ROOM.md). The build team sees only `/spec`. |

## 2. Acceptance criteria (testable — the §16 §9 soundness-critical behaviours)

Tie to `spec/10-kernel/16 §9` + `15 §7`; corpus in
`conformance/kernel/observational/`.

1. **`Eq` forms in Ω and computes by type:** Π → pointwise (**funext
   definitional**: two functions equal iff equal at every arg, no axiom); Σ →
   componentwise; inductive → structural (distinct ctors ⇒ `⊥`); Ω → mutual
   implication (**propext**); primitive → literal.
2. **`cast` regularity:** `cast A A refl a ≡ a` holds definitionally.
3. **`cast` computes on closed canonical type-equalities** (canonicity) — pushes
   through Π/Σ/inductive/quotient structure, not stuck.
4. **`J` reduces on a non-`refl` equality** to a constructor form (the
   `j-nonrefl` test) — *and* `J-β` on `refl`.
5. **Ω proof-irrelevance is definitional:** any two `p, q : P : Ω` convert;
   conversion skips propositional-argument contents.
6. **UIP definitional:** no nontrivial `Eq (Eq A a b) p q`.
7. **Quotient equality reduces to the relation:** `Eq (A/R) [a] [b] ≡ R a b`;
   the respect-checked eliminator computes `elim … [a] ≡ f a`.
8. **Truncation** `‖A‖ : Ω`, `|a|`, map-into-prop eliminator.
9. Subject reduction + decidable checking hold across the new reductions on the
   K1+K2 fragment; conformance `observational/` green; CI green; 80-col;
   `scripts/ken-cargo -p ken-kernel`. **Per the K1 retro: exercise the property,
   not the obvious case** — e.g. `cast`/`Eq` at Π/Σ/inductive/quotient with
   **open** terms and **≥2 distinct** type/level variables, not single-variable
   closed instances (the gap that hid K1's two soundness bugs).

## 3. Guardrails — out of K2 scope (do NOT build here)

- **No cubical machinery whatsoever** — interval, cofibrations, `transp`/`hcomp`/
  `comp`, `Glue`, univalence, `PathP`, HITs (ADR 0005). If a reduction seems to
  need an interval, the design is wrong; it is OTT-by-recursion-on-type.
- **No full decidable conversion algorithm.** K2 adds the observational
  *definitional equations* (Ω proof-irrelevance, `Eq`-by-type, `cast`-refl +
  `cast`-by-type, quotient eq, truncation — `16 §8`) into `whnf`/`conv`. The
  **complete decidable conversion** (lazy-WHNF **NbE** + **SCT** termination
  gating δ) is **K2c (`17`)** — extend K1's conversion through its `convert` seam;
  do not build NbE/SCT here.
- **No general QITs**, no impredicative Ω, no classical/Boolean logic, no kernel
  API repackaging (K-api). 
- If a settled decision looks wrong-for-implementation, that is a `question` to
  the Spec leader (behavioral) or Architect (the K2/K2c boundary, the
  cast-by-type design) per COORDINATION §9 — never a unilateral redesign.

## 4. Logistics

- **Deps:** K1 (`fe1ead1`), F2/F3 — merged. **Source:** `spec/10-kernel/15,16`
  (+ README) — build from `/spec` only; **never** read `local/refs/` or the
  prototype (CLEAN-ROOM.md). The `CICobs`/`TTobs` references are **enclave-only**
  (spec elaboration), for boundary behaviour, never copied.
- **Branch:** `wp/K2-observational` off `origin/main`. **Size L** — split into a
  short series if it helps (e.g. Ω + `Eq`-by-type → `cast`/`J` → quotients +
  truncation), each merging on its own (04-git §2).
- **Highest-risk WP — front-loaded deliberately** (DAG §sequencing). It is where
  a feasibility doubt would surface; retire it early.
- **Done:** acceptance §2 met + retros in (COORDINATION §10). Merge Decision
  mentions **Architect** (always — the deep impl review is the trust-root gate,
  validated twice already) + **Spec** (touches `/spec`+`/conformance`).
