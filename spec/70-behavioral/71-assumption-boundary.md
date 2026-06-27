# The assumption-boundary export IR

> Status: **DRAFT v0**. Normative for the **shape and discipline** of Ken's
> behavioral export; field-level wire details are tuned with the sibling
> (`Ward`). **`OQ-export-ir` DECIDED** (operator, 2026-06-27): the export is an
> **assume-guarantee contract**, generated (never hand-authored) from verified
> content; **Ken-native for the propositional contract, ITF for traces**;
> generators carry **support structure only — never a sampling measure**. ADR
> 0006 (the seam); this is its first concrete deliverable.

## 1. What the export is

When Ken finishes checking a program it emits a stable artifact — the
**behavioral export** — that hands the sibling (`Ward`, ADR 0006) exactly the
part of the specification Ken could **not** close as a static proof. It is the
seam's contract: a faithful statement of *what Ken guaranteed*, *what Ken
assumed*, and *what Ken stated but delegated*.

Two properties make it trustworthy:

- **It is a contract, not a dump** — an **assume-guarantee** record (§2). The
  shape is the permanent framing of a verification boundary (rely/guarantee),
  not an ad-hoc payload.
- **It is *generated*, never hand-authored.** Every field is a projection of
  Ken's verified content — proved `Q`, the residual `P` from
  `trusted_base_delta` (`../20-verification/25 §3`), the effect alphabet from
  the interaction-tree denotation (`OQ-8`, `../30-surface/36 §2`), the
  `Temporal` values written in source (`72`). It therefore **cannot overclaim**:
  it states exactly the four-way epistemic status (`../20-verification/21 §5`),
  with no room to assert more than Ken proved. This is the structural antidote
  to model↔code drift — the model is a *function of the code* (`73`).

## 2. The schema — an assume-guarantee contract

The export has five parts. The first four are **Ken-native** (faithful to Ken's
own terms — they are what `Ward` *reasons about*, and "one logic, two engines"
requires their meaning be identical on both sides); concrete execution witnesses
are a separate **ITF** layer (§3).

| Part | Carries | Status | Downstream use |
|---|---|---|---|
| **`guarantees` (Q)** | proved postconditions & per-space invariants | `proved` | invariants the model may *assume*, not re-prove → smaller state space |
| **`assumptions` (P)** | the assumption boundary: `trusted_base_delta`, explicit `assume`s, boundary labels | `tested` | the nondeterministic *environment*; the generator's input domain |
| **`alphabet` (Σ)** | the interaction-tree perform-node signatures (`OQ-8`) | — | the behavioral state machine's **event alphabet**; the monitor's alphabet |
| **`obligations` (T)** | the `Temporal` data values stated in source (`72`) | `delegated` | LTL / μ-calculus properties to model-check and monitor |
| **`generators` (G)** | refinement/dependent-type **support structure** (§4) | derived | the *territory map* for spec-driven test generation |

`Σ` is **reuse, not reinvention**: the event vocabulary `Ward` monitors over
*is* the interaction tree's perform-node signatures. `Ward` watches exactly the
events Ken's denotation can emit; there is no separate alphabet to define or
keep in sync.

## 3. Two layers: Ken-native contract, ITF traces

- **Propositional/contract layer → Ken-native.** `Q`, `P`, `Σ`, `T`, and `G`'s
  predicates are the objects `Ward` reasons about; they must be semantically
  faithful to Ken's terms. A lossy translation here would break the single-logic
  guarantee.
- **Trace layer → ITF** (Apalache/Quint's *Informal Trace Format*). Concrete
  execution and counterexample witnesses are the cheap interop currency, and
  `Ward`'s downstream tools already speak ITF. Adopting it buys immediate
  Quint/Apalache/MOP interop with no bespoke format to maintain.

The export is **versioned and content-addressed**, and **travels in provenance**
(`../60-security/63 §2`): its hash links *this model* to *this build*, which is
what makes "this `Ward` model corresponds to this code" checkable rather than
asserted — the hook trace-conformance (`73`) builds on.

## 4. Generators carry support, never measure

A refinement type `{x:A | φ}` is a **generator and an oracle** — but generating
*representative* tests means sampling over the combinatorics of state
equivalence classes, and that decomposes into two parts of very different
epistemic status:

- **Support — Ken owns it, faithfully.** *Which* states are valid, and the
  **structure** of that space: the equivalence-class **partition**, the
  **boundaries** between classes, and the **case decomposition** all fall out of
  refinement predicates and `match` arms (equivalence-partitioning and
  boundary-value analysis are *derivable*). Ken exports this partition as `G` —
  an honest map of the territory — claiming nothing about likelihood.
- **Measure — Ken never supplies it.** *Which* valid states are likely /
  important / risky / cheap / judged-out-of-scope. This is business logic, risk
  weighting, operational/UI exclusions, and the empirical data population in
  running systems — **human and environmental judgment**, not a derivable fact.
  A distribution fitted to production traffic is the opposite of a static proof,
  and it is **per-deployment** (the same component needs a different measure as
  an internal API vs. an external endpoint). It therefore lives **outside Ken
  source** — in the same class of deployment-adjacent artifacts as a
  `Dockerfile` or Terraform — as a separately-authored **sampling policy**
  (`OQ-sampling-policy`, hosted on `Ward`'s side, governed like the security
  policy of `../60-security/65`). Ken's exported partition is the **vocabulary**
  that policy indexes its weights over; the two compose with no overlap.

**Exclusions bifurcate** accordingly: an exclusion that is an *invariant* ("this
state never arises because operation `X` maintains `I`") is propositional and
rides the existing **`P`/`Q`** channels (tightening the support, where Ken can
*check* it); an exclusion that is a *judgment* ("valid and reachable, but
low-risk") is a **measure** decision (weight ≈ 0) and belongs to the sampling
policy. Ken handles the first kind already; it stays silent on the second.

## 5. What WS-L / the seam must deliver

The export emitter in `ken-elaborator`: the five-part contract (§2) projected
from verified content; the Ken-native serialization plus the ITF trace layer
(§3); content-addressing + provenance embedding; and the `G` support-structure
extraction (§4) with an explicit *no-measure* invariant. Acceptance: the export
is reproducible from the checked program (same code → same export hash); it
never asserts a claim Ken did not prove or state; a removed assumption shows up
as a changed `P`. Conformance: `../../conformance/behavioral/export/`.
