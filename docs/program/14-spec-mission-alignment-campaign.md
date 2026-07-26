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
| C1 | **Runtime `unknown` execution policy** — universal Kleene third value, or artifact status + explicit execution policy | operator (deployment policy) **then** Architect (semantics) | largest semantic radius in the advisory; crosses every evaluator, backend, FFI, effect path |
| C2 | **`Ord`/`Map` key equality** — must order-equivalence yield kernel `Equal`? | Architect (class design), operator (commercial-data impact) | whether normalized `Decimal` can be a lawful key |
| C3 | **Capability revocation** — universal transitive lineage, or a revocable/non-revocable split | Architect | runtime machinery cost on every capability; `ABI-REVOKE` |
| C4 | **SCT termination** — exact SCT as source compatibility, or a kernel-checkable termination-evidence interface with SCT as one producer | Architect | which total programs are accepted; kernel TCB surface |
| C5 | **Instance coherence + package admission** — keep the exact admission graph, or find the smaller invariant | Architect | multi-package resolution; needs real multi-package cases first |
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

### 6.2 C2 — `Ord`/`Map` over non-canonical carriers

`spec/50-stdlib/52-map.md §2.1` requires order equivalence to yield kernel
`Equal`. That excludes lawful key types with multiple representations —
**including the spec's own `Decimal` example.** Rocq's `OrderedType` is a direct
constructive counterexample: it takes an explicit `eq : t -> t -> Prop`, proves
it an equivalence, and has comparison return equality *in that relation*, never
representation identity.

**Options.** (a) status quo; (b) a `KeyEq`/ordered-key equivalence independent
of kernel `Equal`, plus a proof that ordering respects it; (c) canonicalize
before keying; (d) a stronger `CanonicalOrd` only where kernel equality is
genuinely needed.

⭐ **This one is not ergonomics.** It decides whether ordinary normalized
commercial values are lawful map keys without lying about representation
equality. Prior-art support is the strongest in the advisory.

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
