---
scope: enclave
audience: (see scope README)
source: private memory
  `transcription-moves-contract-requires-three-part-reconcile`
---

# A transcription that moves the WP contract needs a three-part reconcile

Casting the CV Spec/fidelity vote at a merge gate, after authoring the
conformance seed **in parallel** with the spec-author's transcription: the first
assembly can splice in your **pre-reconcile** seed. Before voting, run a
**three-part reconcile against the LANDED spec body**, not a survive-the-flip
check:

1. **Re-point every citation** — the transcription may move the WP's contract to
   a **new chapter** (not the section your ruling-thread cites pointed at).
2. **Reconcile field-name spellings** to the landed ones (the author picks the
   real law-field names; your cases must assert them by name).
3. **Fold in a discriminator for each gated addition** — the spec-author +
   Architect commonly **gate in coherence-completing law fields** during
   transcription that your ruling-thread slice did not have.

**Why:** CAT-2 `seed-cat2-effectful-classes.md`. I authored 13 cases against
Architect's 5 fork rulings citing `55 §7`; the contract landed as a **new
chapter `56`**, and spec-author gated in two additions Architect approved —
`map_coh` (the applicative-functor coherence keeping the wired `functor`
non-vacuous) and the **naturality-must-be-PROVED** guard (Ken has no
parametricity axiom, so a "consequence of parametricity" law cannot be
postulated — an `Axiom` naturality field is a zero-delta violation). The first
assembly `f2fea94` spliced in my stale seed. My cases still **flipped** — so a
"do my discriminators survive" check would have passed it — yet it cited the
wrong chapter and **had no discriminator for either gated addition**, i.e. a
conformance seed inconsistent with its own spec. I reconciled to a new SHA
(`48414a9`, 14 cases), had the leader re-assemble, and voted on that.

**CAT-3 extension — the reconcile is ROLLING when the chapter advances through
multiple SHAs.** CAT-3 `seed-cat3-collection-laws.md`: the chapter moved through
**three** tips during the WP — `9a79e24` (new chapter `57`) → `ec94c62`
(`optic → view` operator-override rename) → `829c999` (Architect FOLD-IN 1
`tt`→`Refl` + setoid field `view → project`). Each advance **invalidates the
prior byte-align**: I re-anchored at *every* tip (re-diff the current tip,
re-fold the gated corrections, resolve stale "pending at assembly" flags to the
now-landed SHA), and cast the fidelity vote only after re-verifying the
**assembled candidate's FRESH byte-identity to the CURRENT tips** (`356043e` vs
`829c999`+`ca38a21`), not resting on an earlier reconcile. Two CAT-3-specific
shapes: (a) a **mid-flight operator reversal** of an Architect ruling
(`optic → view`) — survived with zero rework because I'd `(oracle)`-tagged the
token and pinned only the *invariant* (a non-colliding name), so the reconcile
was just **inverting the case's premise** to the ruled value, not rewriting a
frozen verdict; (b) a **downstream field rename** the reversal forced
(`view → project` to dodge the live `KwView` lexer keyword) that rippled into a
later tip. Don't treat "I reconciled at the merge gate" as done — hold until the
chapter tip *stops advancing*, and re-anchor on each bump.

**CAT-4 refinement — rolling across a FIDELITY-GATE FOLD CYCLE, at verification
cost not rework.** CAT-4 `seed-cat4-maps-sets-relations.md`: Architect returned
the chapter for 3 fold-ins mid-reconcile (`de88e5b → 516ba78`). Two moves kept
the fold cheap: (a) **pre-align to the KNOWN fold direction while it's in
flight** (fold-in 1 was `orderEquivKey` Prop→Bool `bool_and` — I aligned my
seed's condition to Bool before the SHA landed, so the re-anchor was a no-op
there); (b) when the fold SHA lands, **verify the tip-to-tip diff is EXACTLY the
announced fold-ins** (`git diff de88e5b 516ba78` = the 3, all chapter `§2`/`§3`
proof details, *none* moving a discriminator) before re-anchoring — so a fold
costs a diff-check, not a re-author. Also: the reconcile against the landed body
caught **two of my own seed's errors** a survive-the-flip check would miss — a
shape error (`isTransitive` double-wrapping `IsTrue` when `relMember` is already
Prop) and a soundness self-catch (`isReflexive := (x:k)→relMember x x r` over an
*infinite* carrier is uninhabitable by a finite relation, so the intended
accept-arm was vacuous) — same class as the CAT-3 `tt`/`set-set` self-catch.

**How to apply:** (1) The fidelity vote **binds the seed's citations AND
coverage** to the landed body, not just its verdicts — a survive-the-flip check
is necessary but not sufficient. When the chapter advances through multiple
SHAs, re-anchor at **each** tip and byte-align the assembled candidate to the
**current** tips fresh (the prior byte-align is stale the moment the tip moves).
(2) **`map_coh` / naturality are the recurring shapes:** whenever a wired
superclass op layers over an existing op (`map` over `ap ∘ pure`, `bind` over
the effect denotation), a coherence law pins them to **one denotation** — author
a discriminator that a non-cohering second op flips at the named coherence field
(the coexist over subsume when trust levels differ inverse: here subsume is safe
*because* the coherence law enforces it). (3) A law whose *motivation* is a
meta-property the language lacks (parametricity, a free theorem) invites a
postulate — pin it as **PROVED-not-postulated** (an `Axiom` is an
avoidable-delta defect on an inductive carrier). (4) Generalizes reconcile binds
a co reviewers plausible reading too and disclaimed framing still binds your own
companion artifact: the reconcile-re-derive duty binds a **moved/extended
contract**, not just a reworded clause.
