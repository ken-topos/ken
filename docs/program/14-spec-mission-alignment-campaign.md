# Spec mission-alignment campaign — relax mechanism, keep the guarantees

> **Owner:** Steward (program) · **Authoring unit:** spec enclave ·
> **Origin:** operator dispatch 2026-07-26, on the
> conformance-validator's advisory with research's prior-art addendum
> (captured verbatim at
> [`spec-mission-overspecification-advisory.md`](spec-mission-overspecification-advisory.md)).
>
> **This document binds; the advisory does not.** The advisory says what looks
> over-specified. This says what we are doing about each item, who does it, and
> when. ⛔ No agent edits `spec/` from the advisory — only from a released node
> in §4/§5/§6 below.

`SPEC-CLOSURE-BOUNDARY` (merged 2026-07-26) removed persistent
content-addressed **closure** identity from observable semantics while keeping
every validity, authority, and lifetime boundary. This campaign is that WP's
generalization: **the same pattern applied to the rest of the spec.** The
advisory's own closing sentence is the charter —

> remove intensional representation from observable semantics while retaining
> every validity, authority, lifetime, and fail-closed boundary … but not
> indiscriminately to choices that define accepted programs or bind adversarial
> artifacts.

## 0. Where this sits — and ⛔ three things the first pass did NOT do

The umbrella node is **[`SPEC-MISSION-GROUNDING`](issues/SPEC-MISSION-GROUNDING.md)**
(now `active`). It owns *the method and its standard of evidence*; the tracks
below own the edits. Its `AC-M1` is discharged — `docs/MISSION.md` exists on
`origin/main` (blob `1e52e77e`) — and the advisory is its first pass's output.

⛔ **But "the pass ran" is not "the pass is sound," and three of its own
criteria are open.** They are not subsumed by any track below, because a track
audits the *spec*; these audit the *audit*:

| open | what is missing |
|---|---|
| **`AC-M3`** — the independent refutation pass | research supplied prior-art **corroboration**. It genuinely contradicted the CV on three axes (downgrading SCT and instance coherence to forks, upgrading runtime `unknown` to #1), so it was no rubber stamp — but it read the report and commented on it, which is the inheritance shape the AC names. ⇒ **The adversary refutation pass is still owed.** |
| **`AC-M2`** — a taxonomy with a cell for the honest answer | neither the mechanism-vs-property test nor the four-class scheme has an **`inherited`** cell (*in the spec because it was already in the spec*) or a **`cannot-determine`** cell. A constraint nobody can find a derivation for is currently invisible. |
| **`AC-M6`** — the pass reports its own coverage bound | ⚠ **nothing states which of the 63 spec files were read.** The findings that exist are well-grounded; what is unmeasured is what was *not* looked at. ⇒ Absence from the advisory is **not** evidence a chapter is clean. |

⭐ The third one is why §7's ledger accounts for advisory *items* and makes no
claim about spec *coverage*. Those are different completeness questions and only
one of them is answered here.

## 1. The governing method — two tests, applied in this order

**Test 1 — the advisory's discriminator.** For each normative constraint:

> Could two implementations provide identical source meaning, proof results,
> trust boundaries, security guarantees, durable artifacts, and observable
> behavior, yet one fail conformance solely because it uses different internal
> machinery?

A *yes* is an over-specification **signal**, not a verdict.

**Test 2 — research's four-class classification, which is what turns a signal
into a verdict.** A constraint absent from the mission may still properly be
normative:

| class | what it fixes | relaxable from mission text alone? |
|---|---|---|
| **1. Language semantics** | evaluation order, equality, accepted recursion, effect behavior — things that distinguish *programs* | ⛔ **no** — needs a semantic argument |
| **2. Interoperability protocol** | exact bytes/versions/identifiers so independent producers and consumers agree | ⛔ **no** — needs a compatibility argument; belongs in a *versioned profile* |
| **3. Security binding** | a repeated identity field that defeats substitution, rollback, type confusion, mix-and-match | ⛔ **no** — needs a per-edge threat argument |
| **4. Private mechanism** | hash policy, page size, solver routing, copy-vs-share — nothing observable, no protocol consumer, no threat edge | ✅ **yes** — this is the only straightforward class |

⇒ **Class 4 is Track A. Classes 2 and 3 are Track B. Class 1 is Track C.** The
tracks are not priority tiers — they are *what kind of argument the item
needs*, which is why they have different owners and different clocks.

## 2. ⭐⭐ THE MEASUREMENT — every class-4 candidate has a conformance consumer

The advisory's class-4 list reads as low-risk mechanical spec editing. **It is
not.** Measured on `origin/main=9410d7b8` — every private-mechanism family the
advisory nominates is **asserted by at least one conformance row**:

| class-4 candidate | conformance rows asserting the mechanism |
|---|---|
| FNV-1a + `memcmp` + monotonic `u64` addressing | `conformance/runtime/capacity/seed-capacity.md:156` |
| the **0.70** load factor and its resize behavior | `conformance/runtime/capacity/seed-capacity.md:167`, `:170` |
| global dedup — equal values occupy the **same slot**, `==` is O(1) | `conformance/runtime/seed-runtime.md:11`; `conformance/runtime/values/README.md:23`; `conformance/runtime/evaluation/seed-evaluation.md:159`, `:169`, `:193` |
| same-slot as the *structural-sharing* observable | `conformance/surface/collections/seed-collections.md:214`, `:256`, `:593`, `:605` |
| bignum tag `0x01`, inline-`i64` fast path | `conformance/surface/numbers/seed-numbers.md:49`, `:68`; `conformance/runtime/values/README.md:76` |
| minimal-limb sign-magnitude encoding | `conformance/runtime/values/README.md:154`–`:161`; `conformance/README.md:124` |
| canonical two-space indentation as a byte-identity expectation | `conformance/surface/elaboration/seed-multi-binding-let.md:363`, `:393`, `:585` |
| formatter line width | ⛔ **CORRECTED — this cell was wrong.** It cited `seed-canonical-format.md:10`, which asserts only `RED-UNTIL-BUILT` **status, not a width**. The real consumers are `:169`–`:187` (`FMT7`) and `:610`–`:658`, and they assert **88** while `spec/30-surface/31-lexical.md:124` and `crates/ken-elaborator/src/layout.rs:12` say **96**. ⇒ **Not a relaxation candidate — a live contradiction**, tracked as [`SPEC-31-WIDTH-ERRATUM`](issues/SPEC-31-WIDTH-ERRATUM.md). Found by the CV within an hour of the A1 kickoff (`evt_3jpxb2qhkx2d0`), Steward-verified. ⭐ **A locator has two coordinates**; re-deriving the file is not evidence about the lines. |

### What follows from that, and it is the whole shape of Track A

1. ⛔ **A relaxation is a COUPLED `spec/` + `conformance/` change.** Relaxing
   the spec alone produces a spec that permits what the conformance suite
   rejects — which is strictly worse than the over-specification, because now
   the two disagree and the suite wins in practice.
2. ⭐ **So the first deliverable of any relaxation unit is the
   constraint → consumer census, not an edit.** For every constraint proposed
   for relaxation: enumerate the conformance rows that assert it, the `crates/`
   behavior that depends on it, and any external consumer. **A constraint with
   an empty consumer set is relaxable. A constraint with a live row is a
   HARD STOP that routes to the Architect** — because moving that row is a
   conformance-granularity decision, not spec editing.
3. ⚠ **The table above is a SEARCH BUDGET, NOT A POPULATION.** It is a
   keyword grep over `conformance/` for the mechanisms the advisory happened to
   name. An inventory is bounded by an unwritten notion of its surface, so a
   constraint asserted in a spelling I did not grep is **absent from this table
   and still live**. ⇒ The released node must require the enclave to derive its
   own census, **state the reading it used**, and treat my table as a floor it
   may contradict.
4. ⭐ **One class-4 item genuinely has no consumer, and it is the most
   valuable one:** the authority-reversal defect in §3. It needs no census at
   all, because it relaxes nothing.

### And the same reversal is inside the conformance suite

`conformance/runtime/capacity/seed-capacity.md:44` closes a spec/landed-code
divergence note with *"conformance follows the landed code."* Same shape as §3,
one layer down: the artifact that is supposed to be able to **fail** the
implementation defers to it.

## 3. ⛔ FIX FIRST — the authority-reversal defect (needs no relaxation decision)

`spec/40-runtime/44-capacity.md:20` states:

> Where the F4 design and the landed code diverge, the **landed code is
> normative** and the divergence is flagged inline.

### ⚠ Read the scope exactly — narrower than the advisory's summary

The advisory renders this as *"`44-capacity.md` even states that landed code is
normative where it differs from the earlier design prose"* — literally accurate.
But the sentence sits inside an **X2 grounding block** explicitly labelled
*"perishable-frame, K2c-s2 rule"*, and what it reconciles is **two internal
drafts**: the F4 *design prose* versus the landed K3 store. It is **not** a
global declaration that implementation outranks specification.

⇒ **The defect is real but its shape is a drafting-convention leak, not an
inverted principle.** A perishable draft-reconciliation rule is sitting,
untimed and unscoped, in a normative chapter's status block — the exact place
an independent implementer reads for the authority rule. Research's objection
stands at that reading:

> never make "landed code is normative" the authority rule. That reverses the
> spec/implementation relationship and excludes an independent conforming
> implementation by construction.

⇒ **The repair is to scope or retire the convention** (name the two drafts it
arbitrates between, and its expiry), **not** to reverse a principle the spec
does not actually assert globally. Same repair at `seed-capacity.md:44`.

⭐ This is the cheapest high-value item in the entire advisory: it relaxes
nothing, changes no mechanism, closes no fork, and removes the one sentence
that makes independent conformance unattainable **as read**.

## 4. Track A — RELEASED NOW to the spec enclave

**Node:** [`SPEC-ALIGN-A1`](issues/SPEC-ALIGN-A1.md) ·
**Frame:** [`wp/SPEC-ALIGN-A1-private-mechanism-census.md`](wp/SPEC-ALIGN-A1-private-mechanism-census.md)

Scope: the §3 authority repair, plus the constraint → consumer census over the
class-4 candidates, plus the relaxation of **only** those whose consumer set is
empty. ⛔ Every consumer-bearing constraint is a hard stop, deliberately — the
value of A1 is the census and the honest stop list, not an edit count.

**Concurrency:** A1 touches `spec/` and `conformance/`. `RT-FNSPLIT-B2E` is
live in `crates/ken-runtime/`. Contention-free by path; ⚠ **not** semantically
free — A1 must not relax a constraint `B2E`/`B2F` is building against, which is
why the store family lands in the stop list rather than the edit list.

## 5. Track B — CAPTURED, DEFERRED behind the Linux ABI campaign

**Node:** [`SPEC-ALIGN-B1`](issues/SPEC-ALIGN-B1.md) — `status: draft`,
⛔ **not released.**

Operator sequencing (2026-07-26): *"things that we should address, but I'd like
to get to them after the full linux ABI campaign finishes."*

**The gate is measurable, not a vibe:** the `ABI-*` node set is **14 nodes, of
which 1 is closed (`ABI-R1`) and 13 are `draft`** as of `9410d7b8`
(`ABI-A1/A2/A3`, `ABI-M1/M2`, `ABI-R3`, `ABI-REVOKE`, `ABI-S1`–`ABI-S6`). Track B
releases when that set is closed. ⚠ A re-slice can change the set — re-derive
the count at release time rather than trusting this line.

Track B carries three method commitments so the deferral does not lose them
(full statements in the node):

- **The per-edge threat audit.** For each field and duplicated binding: *which
  producer signs it, which consumer checks it, and which concrete substitution,
  rollback, type-confusion, or stale-evidence attack succeeds if it is
  removed?* ⛔ **"Duplicate hash" is not a finding** — mature formats (in-toto
  statements, OCI manifests, TUF) repeat typed digests at layer boundaries on
  purpose. Merge two bindings only when they share authority, signed scope,
  consumer, lifetime, **and** attack set.
- **The version + algorithm agility audit.** Every durable hash or signature:
  algorithm identifier and domain separation, migration without identity
  ambiguity, downgrade prevention, canonical bytes within a version, and
  whether old artifacts stay independently checkable. In-process FNV-1a needs
  no agility once it is private; **any hash crossing a process, package,
  provenance, or archival boundary does.**
- **The three-way protocol-evolution rule.** Unknown **semantic** field →
  reject. Unknown **optional metadata** under a known major version → preserve
  or ignore per profile. Unknown **major version / type URI** → reject. This
  replaces blanket closed-schema rejection, which forces a major-version fork
  for every additive diagnostic field.

## 6. Track C — FORKS: each needs a ruling, not an edit

⛔ **None of these is dispatchable.** Each changes what programs mean, what
keys are lawful, or what a capability costs — a class-1 (or class-1-adjacent)
decision. Owner is the **operator** for product-policy forks and the
**Architect** for semantic-mechanism forks; several need both. Full topic
briefs are in §6.1–§6.8; the operator was briefed on 2026-07-26.

| # | fork | decides | blocks |
|---|---|---|---|
| C1 | **Runtime `unknown` execution policy** — universal Kleene third value, or artifact status + explicit execution policy | ✅ **FULLY DIRECTED 2026-07-26** — retire the runtime value; build-and-fault-if-reached; case 2 resolves to `fault`; §6.1 | needs the §2 census + an Architect ruling on the fault's shape before any edit |
| C2 | **`Ord`/`Map` key equality** — must order-equivalence yield kernel `Equal`? | ✅ **RULED 2026-07-26** — option (b), key relation derived from the order; §6.2 | ▶ needs framing as a localized Map/Set key-interface split |
| C3 | **Capability revocation** — universal transitive lineage, or a revocable/non-revocable split | ✅ **CLOSED — option (a), NO SPLIT, operator 2026-07-26** | nothing; two small editorial follow-ons in §6.3 |
| C4 | **SCT termination** — exact SCT as source compatibility, or a kernel-checkable termination-evidence interface with SCT as one producer | ✅ **CLOSED — option (a), operator 2026-07-26** | nothing; SCT is deliberate source compatibility |
| C5 | **Instance coherence + package admission** — keep the exact admission graph, or find the smaller invariant | ✅ **Layer 1 CLOSED** (ADR 0008) · ▶ **Layer 2 DEFERRED** behind Linux ABI/compiler, probably next | Layer 2 gates on the package-manager round |
| C6 | **Prover search portfolio** — ⚠ *not in the operator's original three-way split; it belongs here* | Architect | needs a certificate interface defined **before** the route can be relaxed |
| C7 | **Logical `space` vs physical realization** (`OQ-Space`) — ⚠ *also not in the original split* | Architect | per-space arenas, re-interning, copying; couples to the store family |
| C8 | **Purity keyword reverse-direction errors** — is `proc`-becomes-pure a reviewed promise or refactoring churn? | operator (review-policy call) | small; cheapest fork to close |

### ⚠ On C6, C7, C8 — my own three-kind summary under-covered the advisory

The three-kind table I gave the operator (relax / versioned profile / fork)
partitioned **the subset of items I had named**, and read as a partition of the
advisory. It omitted the prover portfolio, the logical-`space` mapping, the
purity reverse error, and the two prior-art additions now in §5. ⇒ Recorded
here because it is the same failure the `B2F` chain named: **a clearance names
the axes it covers, because the reader's question is "is this complete?" and
mine was only ever "did the items I listed sort correctly?"** §7 exists so the
next reader can check completeness mechanically instead of trusting a summary.

### 6.1 C1 — runtime `unknown`

**Advisory + prior art rank this #1**, above the content store. The mission
requires `unknown` as an **epistemic classification**. The spec turns it into a
**universal runtime third value** with Kleene propagation through application,
primitives, casts, equality, eliminators, and effects
(`spec/40-runtime/41-values.md §6`, `42-evaluation.md §4`).

Two distinct things are fused: *the epistemic fact* and *the execution policy*.
GHC's typed holes separate them — holes reject compilation by default, a flag
defers them, and a forced hole fails at runtime like `undefined`. So a
hole-bearing program running is a **deployment choice**, not a consequence of
honest labelling.

**Options.** (a) keep universal Kleene propagation; (b) `unknown` stays an
artifact/verifier status and execution is governed by explicit policy —
refuse / quarantine / opt-in dev-mode stub — with a typed failure boundary
rather than an inhabitant of every ordinary type; (c) three-valued propagation
only in domains that specify it.

**Why it is the operator's first.** (b) and (c) are cheaper to specify *and*
implement, but they change what Ken *is for*: whether partially-verified code
runs in production at all is a product commitment, not a semantic one.

## ▶ C1 DIRECTION — retire the runtime third value (operator, 2026-07-26)

> *"`unknown` seems expensive as a development-enabling tool, something that
> would better be handled by carefully crafted error messages."*

⇒ **Option (b).** The epistemic classification stays; the **universal runtime
third value goes**. The hole diagnostic carries the information instead.

⚠ **This is a direction with two follow-on choices it does not settle** (below).
It is **not yet a released node** — it is a class-1 language-semantics change
with live conformance consumers, so it needs the §2 census and an Architect
ruling on the replacement semantics before any `spec/` edit.

### ⭐ The measurement — and it makes the direction *cheaper* than it looks

`unknown` appears at **323 sites across 31 `spec/` files** and **283
`conformance/` rows across 20 files**. ⛔ **Those totals are misleading, and the
disaggregation is the whole point:**

| surface | size | does the direction touch it? |
|---|---|---|
| **verdict / epistemic status / diagnostics / protocol** — `24-diagnostics.md` (60 sites) + `seed-diagnostics.md` (71 rows); `25-protocol.md` (34) + `seed-protocol.md` (36); `21 §5` (16) + `seed-spec-syntax.md` (16); `23-prover.md` (12) + `seed-prover.md` (40) | the **majority** | ⛔ **NO — it all stays** |
| **the runtime value** — `41 §6` (8), `42 §4` (43), `43 §2` cases 1–2 (7); `conformance/runtime/evaluation/seed-evaluation.md` (37 rows), `runtime/effects/seed-effects.md` (16), `runtime/values/README.md` (4) | ~**57 conformance rows** | ✅ yes — this is the change |

⭐ **The vehicle the operator wants to lean on already exists and is already the
larger surface.** `24 §2` specifies `TypedHole { id, goal, context, origin }` —
precisely located, carrying its goal and context and its `22 §1` provenance — and
**71 conformance rows already assert the diagnostics.** ⇒ "Carefully crafted error
messages" is not a thing to build; it is the part of `unknown` that is *most*
thoroughly specified. The runtime value is the thinner half.

⭐ **And `tested` already serves the "run it anyway" workflow.** `21 §5.2`'s
`assume`/`test` status lowers `requires`/`ensures` to a **runtime assertion**,
registers a test/generator obligation, is **visible in the source**, and is
**exported in the assumption boundary**. ⇒ There is already a sanctioned,
explicit path for running code you could not prove — with strictly better honesty
properties than a value that propagates silently.

### ▶ Recommended replacement semantics — the spec's own preferred idiom

⭐ **`43 §2` case 5 already names the shape, and praises it.**
`CapacityExhausted` is specified as a **loud, catchable** fault at the `space`
boundary "**rather than returning a wrong or `unknown` value**." The spec, in its
own voice, already treats returning `unknown` as the worse of the two.

⇒ **Recommendation: hole-dependent evaluation raises a loud, catchable fault
carrying the `TypedHole` payload** — same shape as `CapacityExhausted`. That
keeps the development loop (the artifact builds and runs), is fail-closed, gives
per-run localization, and needs **no inhabitant of any ordinary type**.

## ✅ BOTH C1 FOLLOW-ONS SETTLED (operator concurred, 2026-07-26)

1. ✅ **An `unknown` verdict builds a runnable artifact and FAULTS if a hole is
   reached** — not refuse-to-build. `21 §5.1`'s *"leaves the program running"*
   is amended to that, not deleted: the program still runs, and the fault is
   loud, catchable, and carries the `TypedHole` payload.
2. ✅ **`43 §2` case 2's `fault / unknown` disjunction resolves to `fault`.**
   Unguarded partial primitives fault; the undecided disjunction leaves the
   normative text.

⇒ **C1 is now fully specified as a direction.** ⛔ It is still not a released
node — the §2 conformance census (~57 runtime rows) and an Architect ruling on
the fault's exact shape come first.

⛔ **The two recommendations below are the ones the operator settled above. They
are kept as the reasoning, not as open questions — do not re-ask them.**

1. **What an `unknown` verdict does operationally.** `21 §5.1` currently says it
   "leaves the program **running**." Under the direction that sentence must
   change to either *refuse to produce a runnable artifact* or *build, and fault
   if a hole is reached*. ⇒ **Recommend the latter** — refusing to build kills
   incremental verification outright, which is the one benefit worth keeping.
2. **`43 §2` case 2's undecided disjunction.** Unguarded partial primitives
   (div-by-zero, non-wrapping overflow, out-of-bounds) currently produce "a
   **runtime fault / `unknown`**" — the spec does not say which. ⇒ **Recommend
   resolving to `fault`.** It follows immediately from the direction and removes
   an undecided disjunction from a normative section. Free.

### ⚠ The honest cost of the direction, stated plainly

**Two things are genuinely lost, and neither is recovered by better messages:**

1. **Simultaneous localization across many results.** The third value let one run
   mark *which of 200 outputs* depend on the hole. A fault ends the run, so you
   learn about holes one at a time (or batched, if a harness catches and
   continues). ⇒ The per-result measurement becomes a per-run one.
2. **The absorbing connectives stop being a guarantee.** `42 §4` specifies
   `unknown ∧ false = false` decided "without forcing the `unknown` one" — so a
   partially-verified contract can still **conclusively fail**. Under faults that
   outcome survives only as a consequence of evaluation order, not as a stated
   law. Determinate under CBV, but a weaker kind of promise.

⇒ Both are acceptable if incremental verification is a development convenience
rather than a product claim. **That is exactly the judgement the operator made**,
and it is recorded here so the trade is visible to whoever implements it rather
than being rediscovered as a regression.

### 6.2 C2 — `Ord`/`Map` over non-canonical carriers

⛔ **The advisory over-states this one, and the corrected framing is what was
routed.** `52-map.md §2.1` does **not** impose `antisym → Equal` globally over
`Map`. It confines the step to **two named faces** — the overwrite/uniqueness
law (`§5.3`) and the `Distinct`-discharge lemma — and states explicitly that the
`lookup` laws need **no `Equal` promotion**, "keeping the canonical-carrier
dependency **localized**." So part of the localization the advisory proposes is
**already in the spec**.

What *is* real: `§2.1` states that over a non-canonical carrier a postulated
`antisym` proves `Equal` between distinct representations and therefore
**inhabits `Bottom`** — the `DecEq Decimal` trap of ADR 0010 — using the spec's
own `Decimal = MkDecimalPair coeff exp` example (`10×10⁻¹` vs `1×10⁰`). Rocq's
`OrderedType` is a direct constructive counterexample: it takes an explicit
`eq : t -> t -> Prop`, proves it an equivalence, and has comparison return
equality *in that relation*, never representation identity.

⇒ **The fork was therefore narrower than the advisory frames it:** does Ken need
a non-canonical-carrier route for the **overwrite/uniqueness face specifically**?

## ✅ C2 RULED — Architect, 2026-07-26 (`evt_7jppg10gk983`)

⭐ **Transcribed here because an in-thread ruling is not a durable deliverable.**
Bound to `origin/main = 870f5b65`; the Architect states it did not bind the
advisory, consult a reference implementation, or fold in C4/C5.

### The census — every `antisym → Equal` site, in three classes

⭐ **The classification is the load-bearing part**: separating the law
declaration from its semantic consumers stops the same dependency being counted
several times, which is what made the advisory's count look global.

| # | site | class |
|---|---|---|
| 1 | `spec/50-stdlib/51-lawful-classes.md:90` — `Ord.antisym` declared with kernel `Equal` as its conclusion | **the source contract**, not a consumer; it is what makes every lawful `Ord` carrier canonical w.r.t. its order |
| 2 | `catalog/packages/Core/Classes/LawfulClasses.ken.md:548-709` — `compare … = ord_eq` implies kernel `Equal`; load-bearing call to `d.antisym` at `:709` | **non-Map consumer**; stays canonical-carrier-only |
| 3 | pair `Ord` via `pair_compare` equality soundness (`LawfulClasses.ken.md:1253`); list `Ord` calls `d.antisym` (`:1674`) | **instance-construction** sites — closure of canonical `Ord` under compound carriers, lawful only when every component `Ord` is |
| 4 | `spec/50-stdlib/57-collections-and-views.md:213-224` `eq_from_ord`; shipped comparator `catalog/packages/Data/Collections/Derived.ken.md:864` | **non-Map consumer** (sort's permutation comparator); unchanged by C2 |
| 5 | `spec/50-stdlib/52-map.md:386-394` — overwrite/uniqueness promotes mutual order to `Equal k k'` | ▶ **C2 target 1** |
| 6 | `52-map.md:389-394` + `54-map-verified-laws.md:331-344,459-469` — the `Distinct` discharge | ▶ **C2 target 2** |

✅ **My named gap is closed, and the answer is that it was not a gap.** `54 §5.2`
is **not** an extra site: law 5's own proof is antisym-free — given `Distinct`,
agreement is by `refl`; only the separate `Distinct` discharge uses antisym.
`52 §5.2` lookup/found/locality are likewise antisym-free.
`spec/30-surface/37-strings-collections.md:372` merely repeats the Map boundary,
and `58-maps-sets-relations.md:117-130` supplies the canonical `Nat` witness for
the same discharge — **neither adds a semantic use site.** The lattice law in
`61-information-flow.md` is a distinct interface, not an `Ord`/Map consumer.

### The ruling — option (b), with the key relation DERIVED from the order

**Ken needs the non-canonical-carrier route.** But ⛔ **do not add an
independent `KeyEq`** that can drift from the order — derive it:

```text
KeyEq x y := IsTrue (leq x y) ∧ IsTrue (leq y x)
```

The route is a **total-preorder / key-order dictionary** with `leq`, `refl`,
`trans`, `total`, and **no** theorem from mutual order to kernel `Equal`.

⭐ **Why no second field and no postulated compatibility theorem:** `KeyEq` is an
equivalence from `refl` + `trans`, and its *compatibility with the order* is
**also** derived from `trans` — if `x ≈ y`, substituting either side of `leq`
preserves the result. ⇒ **One order remains the authority.**

⛔ **Do not weaken `Ord.antisym`, and do not create a parallel `CanonicalOrd`**
merely to restate what `Ord` already guarantees. `Ord` stays unchanged as the
**canonical refinement** and continues to serve every consumer whose result
really is kernel identity — sites 2, 3, and 4 above. Existing `Ord` adapts to the
new route by *forgetting* `antisym`.

**Binding rules for the relation-keyed route:**

- lookup and overwrite use `KeyEq`, **never** kernel `Equal`;
- `Distinct` means no two stored entries are `KeyEq`-equivalent;
- insert/from-list discharge `Distinct` **directly** from the overwrite branch
  and preorder compatibility — **no `Equal k k'` step**;
- the overwrite/uniqueness law concludes **one entry per `KeyEq` class**, not
  equality of representatives;
- ⛔ **no theorem may convert `KeyEq x y` to `Equal k x y`** unless the stronger
  canonical `Ord` evidence is explicitly supplied.

### ⚠ The stored-representative policy is OBSERVABLE and must be pinned

The current implementation already replaces **both key and value** in the equal
branch (`Map.ken.md:108-118`) ⇒ **last inserted representative and last inserted
value win**, and `to_list` exposes that representative.

⛔ Structural kernel equality of two `Map` values stays
**representation-sensitive**. Any API-level extensional map equivalence over this
route must compare keys by `KeyEq` — it may **not** claim the two
representatives are kernel-equal.

### The counterexample, closed

Take `x = (10,-1)`, `y = (1,0)` in the non-canonical `Decimal` carrier under a
semantic numeric order, so `x ≤ y` and `y ≤ x`:

- **Today:** `Ord.antisym` produces `Equal Decimal x y`; constructor injectivity
  refutes the unequal fields and **inhabits `Bottom`**.
- **Under the ruled route:** `KeyEq x y` is inhabited and **no kernel equality
  follows**. Inserting `x` then `y` takes the overwrite branch, leaves one node,
  stores representative `y`, and lookup by *either* representation returns the
  last value.

⇒ **The counterexample is excluded without a representation lie.**
Canonicalization-before-keying (option c) remains a valid *adapter* where a
canonical key type is desirable, but it is not the only lawful route. Status quo
would unnecessarily exclude ordinary quotient-like commercial values, and a
renamed stronger class alone (option d) would **preserve** that exclusion.

### ▶ Framing instruction

Frame C2 as a **localized Map/Set key-interface split**. ⛔ Do not reopen the
antisym-free lookup laws, C4, or C5.

### 6.3 C3 — capability revocation

`spec/60-security/62-authority.md §4` gives **every** capability transitive
revocation lineage with exact admission linearization and exact public error
projections. CHERI is the precedent for splitting: ordinary fine-grained
capabilities, with temporal revocation as a separate, expensive mechanism
(CHERIvoke's quarantine-and-sweep carries an explicit time/space tradeoff).

**Options.** (a) universal transitive revocation; (b) two classes — ordinary
non-revocable attenuable, plus explicitly revocable carrying the lineage,
synchronization, and failure contract.

⛔ **Either way, keep:** no ambient authority, and fail-closed use after an
actual revocation. Only the *universality* is in question.

## ✅ C3 CLOSED — option (a), NO SPLIT (operator concurred, 2026-07-26)

**Keep universal transitive revocation.** ⇒ The advisory's item 9 is **answered,
not deferred**; this section is the operative disposition and supersedes it.

⭐ **The three efficiency arguments for splitting each dissolve on measurement,
and one of them inverts.**

**1. Runtime efficiency — the spec already leaves the mechanism free.**
`62-authority.md §4.3` is explicit: *"A controlling space cell, forwarder,
validity index, or region lifetime is **not normative** here. Whatever mechanism
is chosen must preserve the lineage, descendant closure, admission boundary, two
`Revoked` projections, and settlement observations."* ⇒ **No spec change is
needed to make the common path cheap.**

⭐ **And rarity is an argument for universality, not against it.** Because
revocation is rare in Ken's target class, the cost can be paid *at revoke time*:
on `revoke(X)`, walk X's descendants once and mark each dead — O(descendants),
paid almost never. The **use path** then becomes a single liveness-bit read, with
**no ancestor walk**. That preserves all five `§4.3` observables and is
CHERIvoke's own shape (pay at revocation, not at dereference). ⇒ The universal
design costs *one bit check* per capability-consuming operation, and a split
would buy removing that bit check.

**2. Compiler efficiency — there is no compiler cost to split away from.**
`§4` makes `attenuate`/`revoke` **management semantics, not Ken terms**: "absent
from the Ken name environment, as is every public `Cap` constructor or producer."
The source path stays `ProgramCaps`/`readFile`/`writeFile`. ⇒ The compiler sees
**nothing** about revocation — no lineage tracking, no type-level revocability,
no dataflow.

⛔ **A compiler-side win would require making revocability static — i.e. putting
it in the type — which makes revocation Ken-visible and reverses the design that
keeps it out of the language.** That is a coherence cost paid to optimize a path
that currently costs nothing.

**3. Linearization — real, but not where the phrasing suggests.** `§4.2`'s
admission point separates exactly two outcomes. The expensive part is the
**atomicity** of the liveness check against a concurrent marking, not an O(depth)
traversal. ⛔ **And a split does not remove it** — every revocable capability
still needs the full admission barrier; the split only excuses capabilities that
were already down to a bit check.

### ⛔ The soundness seam that decides it

`§4` requires that revoking withdraws everything attenuated from it **to any
depth**, and that consuming a resource token cannot bypass revocation. A split
must answer: **can a non-revocable capability be derived from a revocable one?**

- **Yes** ⇒ the derivation *escapes* revocation and transitive withdrawal is gone.
  That is the entire guarantee.
- **No** ⇒ you need a one-way monotonicity rule, which means tracking lineage and
  revocability across every attenuation **anyway**.

⇒ **The split's soundness rests on a rule that reintroduces the machinery it was
meant to avoid.**

### ▶ Two small follow-ons this ruling does authorize

1. ⚠ **An editorial note at `§4.2`.** *"Every ancestor are live"* reads as an
   algorithm and invites an implementer to write the ancestor walk. It is an
   **observable property**, and `§4.3` frees the mechanism — say so, so nobody
   implements the slow version from a correct spec.
2. **Confirm `ABI-REVOKE` is scoped to the bit-check design**, not a
   validity-index walk. That is where this decision actually cashes out.

⚠ **Axes this covers.** The analysis binds `62-authority.md §4`–`§4.3` only. It
does **not** cover `44 §3`'s store-`Space` realization or the `ABI-REVOKE` node,
and `§4.3` explicitly disclaims cross-space and distributed revocation — so the
clearance is for the **single-space OS-operation face**. Distributed revocation
could change the cost picture, and it is out of scope by the spec's own words.

### 6.4 C4 — SCT termination

`spec/10-kernel/17-conversion.md §4` fixes one exact size-change-termination
graph and matrix algorithm. ⚠ **Research explicitly downgraded this from
"relax" to "fork"**, and the reason is load-bearing: the accepted set of
transparent definitions and their reduction behavior is **observable source
semantics**. Swapping SCT for another incomplete checker makes valid programs
fail or invalid ones pass. Lean accepts both structural and well-founded
recursion and documents that they differ in definitional-equality and
kernel-computation behavior.

**Options.** (a) exact SCT acceptance is deliberate Ken source compatibility —
say so, and stop calling it over-specification; (b) define a stable
kernel-checkable **termination-evidence interface** first, admit
structural/SCT/well-founded/future producers that emit accepted evidence, and
specify whether two routes preserve the same definitional equations.

⛔ **(b) without the evidence interface is not available** — "transparent
unfolding is certified terminating" alone is too weak to be a spec.

## ✅ C4 RULED — option (a). CLOSED (operator, 2026-07-26)

**"Keep what we have. Close this."** ⇒ Exact SCT acceptance is **deliberate Ken
source compatibility**, not over-specification. The evidence interface of option
(b) is **not** commissioned.

⛔ **Consequences that bind every later reader:**

- `spec/10-kernel/17-conversion.md §4` is **correct as written**. Do not file a
  relaxation node against it, and do not carry it as a known over-specification.
- **Advisory item 4 is answered, not deferred.** Anyone re-reading the advisory
  will find item 4 recommending an evidence interface; that recommendation is
  **declined**. The advisory is not amended (it is a gitignored external input),
  so **this section is the operative disposition** and the advisory's item 4 is
  superseded here.
- The accepted set of transparent definitions is now explicitly a **source
  compatibility surface**. ⇒ A future change to SCT acceptance is a
  **breaking language change**, not an internal mechanism swap — that is the
  substantive content of this ruling, and it should be honoured as such.
- ⚠ **This does not bless the algorithm's exposition.** The ruling settles
  *which programs are accepted*; if the spec's matrix presentation is later found
  to over-fix an implementation detail that does not change acceptance, that is a
  Track A editorial item, not a reopening of C4.

### 6.5 C5 — instance coherence and package admission

`spec/30-surface/33-declarations.md §5.5` fixes one canonical structure
instance, forbids orphans, and defines a detailed package-admission graph.
⚠ **Research also moved this to fork status:** Rust's orphan/overlap rules show
coherence is commonly a **language compatibility property** — it prevents two
downstream packages creating conflicting implementations and preserves library
evolution.

**Options.** (a) keep it; (b) find the smaller invariant that still gives
legible deterministic resolution — but ⛔ *"one deterministic instance"* and the
open-world ownership rule need an **alternative coherence proof** before
relaxation. Prerequisite either way: test the accumulated rules against real
multi-package cases, which do not exist yet.

## ✅ C5 SPLIT AND HALF-CLOSED (operator, 2026-07-26)

⭐ **C5 was never one item. It is two layers, and only one was ever open.**

### ✅ Layer 1 — the coherence policy: CLOSED, keep as-is

`§5.5`'s heading already carries **`(OQ-classes, ADR 0008 — do not reopen)`**.
Property classes get coherence free from proof irrelevance; structure classes get
one canonical instance per `(class, head-type)`, no overlap (ambiguity is a
compile error **naming both** candidates), orphans rejected at declaration.

⭐ **ADR 0008's reason is stronger than the compatibility argument the advisory
offers.** The resolved dictionary is **semantically load-bearing — it carries law
proofs the prover uses.** If `Monoid A` could denote different dictionaries at
different sites, *"a lemma proved about 'the `Monoid A`' could be unsoundly
combined with data built from a different `Monoid A`."* The ADR calls coherence
**"a soundness-adjacent property of client reasoning, not merely an ergonomic
preference."** ⇒ Rust's orphan rules protect an *ecosystem*; Ken's protect *proof
validity*. Different arguments, and the stronger one applies here.

⛔ **So Layer 1 was never a relaxation candidate**, and the advisory's "may still
be over-detailed" does not reach it.

⭐ **The escape hatch already answers the commercial-data worry.** Named
instances are first-class values: define a non-canonical `byLength : Ord String`
and pass it explicitly (`sortBy byLength xs`). Explicit passing is ordinary value
application, **bypasses search**, and does not perturb canonicity — implicit
`Ord String` still resolves to the canonical one. ⇒ **Coherence constrains
implicit search, not what you can express.**

### ▶ Layer 2 — package admission: DEFERRED behind the Linux ABI / compiler work

`§5.5.1` (MRES-4d/4e) is the newer, much more detailed half: the **coherence
set** (unfiltered transitive closure of the source graph) versus the
**direct-use set** (self-admitted package + `admits` roots + canonical instances
carried by re-exported public surfaces), with `UnadmittedInstance` as the hard
error and observable provenance on success.

⭐ **The audit's prerequisite is not effort — it is a whole unbuilt subsystem, and
`§5.5.1` says so itself.** Its `SPEC-NOW / BUILD-LATER` block declares
normative-but-unbuilt: package member lists, **compiled instance manifests**,
cross-package collision errors at the manifest boundary, registries, lockfiles,
and test-scoped admission — all *"deferred to the package-manager round,"* with
the gate meanwhile operating over the existing path-based source graph. ⇒ **You
cannot audit the admission graph against real multi-package cases before the
package-manager round, because the mechanism those cases would exercise is itself
deferred.**

**Sequencing (operator):** package management is **lower priority than the Linux
ABI and compiler work — but probably next after it.** ⇒ Layer 2's audit is a
**gate on the package-manager round**, not available work now.

### ⚠ The risk to carry forward into that round

`§5.5.1` specifies a **detailed** admission graph `SPEC-NOW` for delivery that is
`BUILD-LATER`, with an observable error and observable provenance already
normative. If the package-manager round finds these rules do not fit real
dependency graphs, they will by then be normative **with conformance rows
attached**.

⭐ **That is the same shape as the authority-reversal defect §3 is fixing** — a
spec surface committed ahead of the implementation that would have contradicted
it. Recorded now because it is cheap to note and expensive to rediscover late.

### 6.6 C6 — prover search portfolio

`spec/20-verification/23-prover.md §2`–`§4` normatively fixes the D/FO/HO
classifier, Kripke translation, Z3 route, and reflective-checker architecture.
Kernel rechecking already makes proof search untrusted and replaceable — Lean
tactics and SMTCoq both keep a stable trust boundary while search evolves.

⭐ **Why this is a fork and not a Track A relaxation:** the relaxable thing is
the **exclusive search route**, and you cannot relax it without first having a
**normative, versioned certificate-language + verified-checker interface** for
alternatives to emit into. Define the interface, then the portfolio becomes a
profile. The Kripke adequacy theorem may stay permanent semantic infrastructure
if it is how classical certificates enter Ken's intuitionistic logic.

⛔ **Stays normative regardless:** exhaustive obligation accounting;
`proved` only after kernel certificate acceptance; honest `disproved`/`unknown`;
and no silent promotion of a search failure or unsupported fragment to `proved`.

### 6.7 C7 — logical `space` vs physical structure

`spec/30-surface/36-effects.md §4` + `44-capacity.md §1`–`§3` couple the logical
isolation guarantee to shared-nothing actors, closure-free content-addressed
messages, per-space arenas, re-interning, and reclamation boundaries — possibly
foreclosing ownership-based sharing, regions, or alternative actor
implementations with identical observable guarantees. Erlang is the precedent:
messages are normally copied, but refcounted binaries and literals **are**
shared on a node, and the logical process model never exposes which.

Open decision `OQ-Space` already exists in `spec/90-open-decisions.md`.

⚠ **The caveat is the fork's whole difficulty:** failure isolation and message
order **are** observable. Relaxing "shared-nothing storage" must not
accidentally relax no-shared-mutable-authority. ⇒ Couples tightly to the store
family in §2 — do not decide C7 and the store separately.

### 6.8 C8 — purity keyword reverse errors

`spec/30-surface/36-effects.md §1.6` makes every `const`/`fn`/`proc` mismatch a
hard error **including a `proc` whose implementation becomes pure.** Standard
effect systems use subeffecting: an annotation is an **upper bound**, so a pure
body under an impure annotation is normally fine.

**Options.** (a) keep exactness and document the human-review benefit that
outweighs refactoring churn; (b) one-way checking — `const`/`fn` may not
perform undeclared effects, and a `proc` body may become pure.

⛔ **Not a soundness question either way.** It is a review-policy call, which is
why it is the operator's and why it is cheap.

## 7. Disposition ledger — every advisory item, accounted for

⭐ Completeness-critical: if an advisory item is not in this table, the table is
wrong. **`—` in the Track column is not permitted.**

| advisory item | class | track |
|---|---|---|
| 1. content-addressed runtime store — *durable canonical encoding* | 2 | **B1** (durable bytes are a protocol) |
| 1. content-addressed runtime store — *in-process interning, FNV-1a, probing, load factor, page size, slot retirement* | 4 | **A1 census → expected STOP** (§2) |
| 1. content-addressed runtime store — *same-slot conformance + O(1) equality as a promise* | 1/4 | **C7-coupled** (§6.7) |
| 1. authority reversal (*"landed code is normative"*) | — | **A1, fix first** (§3) |
| 2. runtime `unknown` | 1 | **C1** |
| 3. automated-prover architecture | 1/4 | **C6** |
| 4. exact SCT termination | 1 | **C4** |
| 5. logical `space` → physical structure | 1/4 | **C7** |
| 6. Ward export + trace (ITF) schemas | 2 | **B1** |
| 7. checked-package + executable envelopes | 2/3 | **B1** (per-edge threat audit) |
| 8. `Ord`/`Map` canonical carriers | 1 | **C2** |
| 9. universal transitive capability revocation | 1/3 | **C3** |
| 10. named supply-chain products (Sigstore/Cosign/in-toto/SLSA) | 2 | **B1** |
| purity keywords as bidirectional hard errors | 1 | **C8** |
| instance coherence + package admission | 1 | **C5** |
| formatter details (width, indentation, fences) | 4 | **A1 census** (has conformance rows — §2) |
| numeric inventory + representation | 1/4 | **A1 census** — split: ranges/rounding/overflow/normalization/equality are **semantics, not relaxable**; tags, limb width, coefficient layout, fast paths are class 4 |
| FFI + buffer protocol | 2 | **B1** — as an ABI profile; ⚠ intersects the live Linux ABI campaign, so B1's gate is the right clock |
| version + algorithm agility audit *(prior-art addition)* | 2/3 | **B1** |
| protocol evolution three-way rule *(prior-art addition)* | 2 | **B1** |
| "constraints not to relax" list | — | **§8 — carried into every node's guardrails** |

## 8. ⛔ DO NOT RELAX — carried into every node in this campaign

A simplification campaign is exactly the context in which these get shaved by
accident, so they are repeated in each released node's guardrails rather than
cited:

- the small auditable kernel;
- kernel rechecking of **every** claimed proof certificate;
- totality and predictability by default;
- explicit partial and foreign boundaries;
- exhaustive obligation extraction;
- honest `proved`/`tested`/`delegated`/`unknown` status;
- explicit effects, capabilities, IFC, provenance, and trust;
- loud failure rather than silent weakening or corruption;
- the prohibition on promoting Ward, test, or monitor results to `proved`.

**Their mechanisms may be simplified. The guarantees may not.**

## 9. Per-relaxation reporting contract

Every landed relaxation, in **any** track, records:

1. the mission outcome that remains protected;
2. the observable or security invariant retained;
3. the implementation choices newly permitted;
4. any external consumer requiring exact compatibility;
5. **a conformance pair showing the relaxed contract still rejects an actual
   mission-breaking implementation.**

⭐ Item 5 is the one that makes this campaign auditable rather than a
subtraction exercise: a relaxation that cannot still reject a bad
implementation did not relax a mechanism, it deleted a guarantee.

## 10. Status

| track | node | state | clock |
|---|---|---|---|
| **A** | `SPEC-ALIGN-A1` | released to the spec enclave 2026-07-26 | now |
| **B** | `SPEC-ALIGN-B1` | `draft`, captured, ⛔ not released | after the `ABI-*` set closes (§5) |
| **C** | — | operator briefed 2026-07-26; C1/C8 need the operator, C2–C7 need the Architect | per fork |

**Last updated:** 2026-07-26 (Steward). **Next action:** A1 terminal handoff →
Track C rulings as the operator schedules them → B1 release when the ABI
campaign closes.
