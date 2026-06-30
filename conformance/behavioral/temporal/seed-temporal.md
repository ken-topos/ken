# Behavioral-temporal conformance — seed cases (B2)

Format: `../../README.md`. These pin the **`Temporal Σ` datatype and its export
flow** (`spec/70-behavioral/72-temporal.md`, impl-ready B2): temporal/behavioral
logic as **deeply-embedded inductive data** (LTL / μ-calculus over the B1
alphabet `Σ`), **reasoned about but not with** — Ken *states* the obligation and
**delegates** its discharge to `Ward`, gaining **no** modal judgment in its
trusted core (`OQ-temporal` DECIDED — data-only, ADR 0006). The cases net (a)
that `Temporal` is an **ordinary inert inductive** admitted by **K1**, (b) the
**derived operators** `◇`/`□`/`leadsto` are `until`/`not` **syntax, not
constructors**, (c) the surface→`delegated`→`T` export flow is **total,
constant, and one-way**, (d) the buildable-now reason-*about* metatheorem
(closedness), **and** (e) the structural **absence** of any kernel modality.

## Reading disciplines

**The observable is structural — the elaborated `Temporal` term, the kernel's
absence-of-modality, or the projected export field — never reason-*with*.** A B2
case routes a real declaration / elaboration / export through the **real**
machinery and asserts a **structural property**: which constructor head a
derived operator elaborates to, whether the strict-positivity check admits the
datatype, which export field a `Temporal` value lands in and with what status,
or the absence of a named kernel construct. It is **not** the discharge of a
temporal obligation (that is `Ward`'s — `72 §2`) and **not** the meaning of the
verdicts/statuses it projects (that is `21 §5`, `../../verify/`, subsumed
below).

**Status → field is pinned at source (`72 §5`, faithful to `21 §5.2`) — no
silence to fill (promoted V3).** The map is design-time-locked and **constant**:
`Temporal`-in-source ↦ `delegated` ↦ **`T`**, **always and only** — never
`proved`/`Q`, never `tested`/`P`, never `unknown`. The conformance author
inherits **no** classification gap (the verdict-mapping silence is foreclosed
upstream, as in Sec1ct / B1). These cases net the projection's *fidelity* and
its *one-way* discipline, not a choice of mapping.

**`Temporal`-as-data is netted through the REAL machinery (the QA gate, `72
§9`).** Real `declare_inductive` + `check_positivity` (the K1 strict-positivity
check, `14 §2`) for the datatype; real generated `elim_Temporal` (`14 §3`) for
the metatheorem; the **real B1 emitter** (`crates/ken-elaborator/src/export.rs`)
for the export flow — never a synthetic `Temporal` literal where a real
elaboration is asserted, never a hand-built export `Value`.

**The no-kernel-modality net is a grep-for-forbidden-construct absence —
guard-gated, not happy-path (the B2 analog of B1's seals / Sec1 N1).** The
durable decision (data-only, never modalize the kernel) is realized as an
**absence in the kernel**: no `▷`/later modality, tick variable, Löb rule, clock
structure, or temporal judgment form exists. The kernel **cannot self-check the
absence of a rule it does not have**, so conformance is the **sole** net —
exactly the erased-before-kernel omission-hole the export bytes (B1) and the
security labels (Sec1/Sec1ct N1) face. An absence case **names the seal** and
passes the **disconfirming check** (*would the bug this targets also pass?*).

**Defer-spelling-not-concept (`72 §3.1`).** The LTL/μ value-set + meaning, the
inert-data property, **strict positivity (K1, not the K1.5 W-style extension)**,
and **first-order fixpoint binding** are **locked** (normative,
conformance-checkable). The **exact constructor set + spelling**, the `Pred Σ`
atom language, the fixpoint-variable form, and the ITF/`WardFormula`
serialization are `(oracle)`-tagged (the joint Ward encoding pass). Cases refer
to a constructor by **concept** (`until`/`not`/`mu`) and tag the literal token;
they pin the value-set + invariants and **do not over-freeze** the spelling
(assert-at-locked-granularity — a rename after the spelling binds is a breaking
export change, `71 §3.3`).

**Tags.** `(soundness)` = a durable trust-model commitment that must never
regress — netted **solely** by conformance where the kernel is blind (the export
bytes; the *absence* of a modality the kernel cannot self-check). `(oracle)` =
the deferred spellings, finalized by the Ward encoding pass.

## TE-A. `Temporal` is ordinary inert data — admitted by K1 (AC1)

### temporal/ordinary-inductive-admitted-by-k1 (AC1)
- spec: `spec/70-behavioral/72-temporal.md §3`, `§3.1`; `14 §1`, `§2`
- given: `Temporal Σ` declared via the **landed L2 `data` machinery** (`14 §1`:
  real `declare_inductive`) — `atom`/`not`/`and`/`or`/`next`/`until` plus
  first-order `mu`/`nu`/`var` — with every recursive occurrence of `Temporal Σ`
  **direct** (strictly positive); run the kernel's strict-positivity check and
  generate the eliminator
- expect: **accepted** by `check_positivity` (`14 §2`) **without** invoking the
  K1.5 W-style admission path (`14 §2.1`) — there is no Π-bound / branching
  recursive occurrence — and elimination is the **ordinary generated
  `elim_Temporal`** (`14 §3`); no kernel extension is engaged
- why: AC1 — `Temporal` is the **most basic inductive shape**, inert to the
  kernel. Grounds the spec's "admitted by K1" against the kernel that exists now
  (`check_positivity`, `inductive.rs`): direct occurrences are exactly the
  admitted class. Half of the strict-positivity pair below — **alone it is
  green-vs-green** (a kernel with an over-permissive positivity check that
  admits everything also passes); the net is the pair with the HOAS reject.

### temporal/hoas-fixpoint-breaks-positivity-rejected (AC1, soundness)
- spec: `spec/70-behavioral/72-temporal.md §3.1`; `14 §2`
- given: the **HOAS variant** of the fixpoint binder — `mu : (Temporal Σ →
  Temporal Σ) → Temporal Σ` — placing `Temporal Σ` in a **negative** position
  (to the left of an arrow); run the **same** `check_positivity`
- expect: **rejected** (`14 §2`: a non-strictly-positive / negative recursive
  occurrence) — the same check that admits the first-order datatype refuses the
  HOAS encoding
- why: (soundness) AC1 — **first-order binding is load-bearing, not incidental**
  (`72 §3.1`): a HOAS `mu` breaks strict positivity, so the deferred encoding
  pass **must** preserve first-order `Var`. The **non-degenerate pair** with
  TE-A1 — one datatype, two binder encodings, **opposite** verdicts — is the
  sole net for the constraint (a single accept is green-vs-green under an
  over-permissive check). The verdict **flips** on the structural discriminator
  (negative occurrence), grounded in the kernel's own `negative occurrence (Bad
  → Bool) → Bad must be rejected`. Names the guard: the polarity check's
  negative-occurrence rejection.

## TE-B. No kernel modality — the structural absence (AC1, soundness)

### temporal/no-modal-construct-in-kernel (AC1, soundness)
- spec: `spec/70-behavioral/72-temporal.md §1`, `§7`; `README §1`
- given: the kernel (`crates/ken-kernel`), with `Temporal` programs in play
- expect: a **grep-for-forbidden-construct** net finds **no** `▷`/later
  modality, **no** tick variable, **no** Löb rule, **no** clock structure, and
  **no** temporal judgment form anywhere in the kernel's term language,
  conversion, or typing judgments — a **structural absence**, not "the happy
  path avoids them"
- why: (soundness) AC1, the durable headline (`72 §1`, `OQ-temporal` DECIDED).
  The data-only decision is realized as an **absence in the kernel** —
  exhaustive-by-construction (a modal construct is unrepresentable; no temporal
  judgment exists), the B2 analog of B1's no-measure / no-promotion seals and
  Sec1's erased-before-kernel N1. Names the seal. **Disconfirming check:** would
  a kernel that grew a `▷`/later modality pass this net? **No** — the grep
  surfaces the construct → red. A guard-gated absence, not coincidence. (The net
  targets the named constructs *as kernel constructs* — distinct from the
  English word "later" in incidental prose.)

### temporal/inert-to-conversion (AC1, soundness)
- spec: `spec/70-behavioral/72-temporal.md §7`; `14 §3`
- given: a program **with** a `Temporal Σ` value vs the **same** program without
  it; compare the kernel's conversion algorithm and typing judgments
- expect: **byte-for-byte the same rules** — `Temporal Σ` is consumed **only**
  by the ordinary generated `elim_Temporal` (`14 §3`), introduces **no** new
  conversion/η rule and **no** reduction outside ordinary ι, and changes
  **nothing** in the judgmental structure
- why: (soundness) AC1 — *inert to conversion*, the second face of the
  no-modality seal (the first is construct-absence above). Adding `Temporal`
  adds an inductive type and its ι-rule, nothing more. Observable consequence
  (conversion unchanged) + names the seal (no `Temporal`-specific conversion
  edge). Pairs with TE-B1: B1 nets *no modal construct exists*; B2 nets *the
  existing structure is untouched*.

## TE-C. Derived operators elaborate to the core (AC2)

### temporal/eventually-elaborates-to-until-true (AC2)
- spec: `spec/70-behavioral/72-temporal.md §3`, `§4`
- given: the derived operator `◇φ` (`eventually φ`) elaborated to a `Temporal Σ`
  term
- expect: the elaborated term is **`until (atom ⊤) φ`** — a **structural**
  assertion on the constructor head (`until`, with second argument `φ`) —
  **not** a dedicated `eventually`/`diamond` **constructor** (no such
  constructor exists in the datatype, `72 §3`)
- why: AC2 — `◇`/`□`/`leadsto` are **derived syntax, not constructors**. The
  verdict is the **emitted term's structure** (head = `until`), which a
  primitive-modality bug changes: an elaborator that made `◇` a constructor
  yields head `eventually` → red. Structural-output assertion (the term head),
  not a value verdict — it cannot go vacuous (promoted X1). The `atom ⊤` "true"
  predicate is concept; its spelling is `(oracle)` (the `Pred Σ` atom language).

### temporal/always-elaborates-to-not-until-not (AC2)
- spec: `spec/70-behavioral/72-temporal.md §3`, `§4`
- given: `□φ` (`always φ`) elaborated
- expect: **`not (until (atom ⊤) (not φ))`** — head `not`, inner head `until`,
  innermost `not φ` (the `¬◇¬φ` unfolding); no `always`/`box` constructor
- why: AC2 — the `□` dimension, **distinct** from `◇`: a primitive-`□` bug
  yields head `box`, **undetected by TE-C1 alone** (the multi-dimensional-guard
  rule — each derived operator is its own dimension and needs its own structural
  case, promoted K2c-series-2). Flips on the head.

### temporal/leadsto-elaborates-to-box-of-or-diamond (AC2)
- spec: `spec/70-behavioral/72-temporal.md §3`, `§4`
- given: `p ~> q` (`leadsto`) elaborated
- expect: **`□ (not p or ◇ q)`** ⇒ fully `not (until (atom ⊤) (not (or (not p)
  (until (atom ⊤) q))))` — built **entirely** from the `until`/`not`/`or`/`atom`
  core, exercising the **composition** of the derived `□` over the derived `◇`
- why: AC2 — the `leadsto` dimension *and* the strongest derived-not-primitive
  net: `~>` is two layers of derivation (`□` over `◇`), so a primitive at
  **any** layer surfaces in the elaborated tree. Structural assertion on the
  full term.

## TE-D. Surface `temporal{}` → `delegated`, human-visible (AC3)

### temporal/block-elaborates-delegated-and-visible (AC3)
- spec: `spec/70-behavioral/72-temporal.md §4`; `21 §5.2`
- given: a surface `temporal { eventually settled }` claim (keywords `(oracle)`,
  `OQ-syntax`) run through elaboration
- expect: it elaborates to the **§3 constructors** (here `until (atom settled?)
  …`, the derived `◇`) and is tagged **`delegated`** in the four-way status (`21
  §5.2`) — **not** `proved`/`tested`/`unknown` — and is **human-visible** (the
  formula appears verbatim in source, not erased)
- why: AC3 — a behavioral property Ken **states but does not discharge**.
  Discriminating on the **status**: `delegated` is the one status that *adds
  nothing to the trusted base* (exported, not assumed, not closed — `21 §5.2`,
  `§5`). The keyword spelling is `(oracle)` (`OQ-syntax`); the **elaboration
  target + status** are pinned.

## TE-E. Export flow — `T`/`delegated`, never `Q`/`P`, one-way (AC4, soundness)

### temporal/value-projects-to-T-delegated-never-q (AC4, soundness)
- spec: `spec/70-behavioral/72-temporal.md §5`; `71 §2.1`, `§5.1`
- given: a real elaborated `temporal{}` value routed through the **real B1
  emitter** (`crates/ken-elaborator/src/export.rs`, the `T`/`obligations`
  channel — `TEntry`, the B2-filled body)
- expect: it lands in **`T`** (concept `obligations`; literal key `(oracle)`)
  tagged **`delegated`**, and is **absent** from `Q` (`guarantees`) and `P`
  (`assumptions`) — the total, constant mapping `Temporal`-in-source ↦
  `delegated` ↦ `T` (`72 §5`)
- why: (soundness) AC4 — the projection's honesty direction: a delegated
  property is **never** the guaranteed half (`Q`) and **never** an assumption
  (`P`). Routed through the **actual** emitter (the `temporal` parameter of the
  `T` channel; status serialized as the constant `"delegated"`), not a synthetic
  export literal. Half of the never-promote pair (with TE-E2): **alone**, an
  emitter that dumps *everything* into `T` also passes — the net is the pair
  with the post-Ward case.

### temporal/ward-green-keeps-delegated-never-promoted (AC4, soundness)
- spec: `spec/70-behavioral/72-temporal.md §5`; `71 §5.1` (I4); `63 §5a`
- given: the same `delegated` `T` obligation, then a simulated **`Ward` green
  discharge** re-entering Ken **only** as a discharge-attestation bound to the
  export hash (`63 §5a`) — **not** as a kernel certificate
- expect: the obligation **stays `delegated`** (in `T`), is **never** re-stamped
  `proved`, **never** appears in `Q`; and there is **no emitter code path** from
  a `Ward`/classical verdict to a `proved` status (the one-way gate)
- why: (soundness) AC4 / I4 — the **one-way gate** (`71 §5.1`). Realized as the
  **absence of a promotion edge**, not a runtime check — a guard-gated absence,
  named: *no `proved`-writing edge from a delegated/`Ward` source exists* (the
  emitter constructs `Q` entries **only** from kernel `Verdict::Proved` values;
  no parameter accepts a Ward "green"). **Disconfirming:** would a delegated
  obligation with a green `Ward` result land in `Q` under this bug? **Yes** →
  the case flips. The **non-degenerate pair** TE-E1/E2 (project-honestly +
  never-promote) is the sole net for the constant one-way mapping. **Subsumes**
  B1's `export/delegated-obligation-never-promoted-to-proved` (EX-E1): B1
  simulated an *abstract* delegated `T`; B2 drives the **real** `temporal{}`
  source through the same gate (subsume-don't-proliferate — the gate's *meaning*
  is B1's; B2 nets the real-source wiring).

## TE-F. Reason-*about*, not -*with* — both faces (AC5)

### temporal/closedness-metatheorem-typechecks-via-elim (AC5)
- spec: `spec/70-behavioral/72-temporal.md §6.1`, `§2`; `14 §3`
- given: `closed : Temporal Σ → Bool` (true iff every `var X` occurs under a
  binding `mu`/`nu X`) defined by **ordinary `elim_Temporal`** with a binder
  environment; and the metatheorem that the elaboration of a `temporal{}` block
  yields `closed = true`, preserved by the structural operations
  (`next`/`and`/`or`/… build `closed` from `closed`)
- expect: the definition and metatheorem **type-check as ordinary static proof**
  over the inductive type — **and** `closed` **computes discriminatingly**: a
  formula with `var X` **bound** by an enclosing `mu X` → `true`; the **same**
  formula with `var X` **free** (no enclosing binder) → `false`
- why: AC5 — the reason-*about* face: metatheory of the *embedding* by plain
  `elim`, with **no** trace / satisfaction model and **no** new kernel power
  (`72 §6.1`). The **bound/free verdict flip** pins that `closed` actually
  inspects structure (not green-vs-green) and names the guard (the
  binder-environment lookup finds `X` absent → `false`). The deeper
  semantics-preservation property (`sat`, `§6.2`) and the `compile` faithfulness
  lemma (`§6.3`) ride the deferred Ward encoding and are **not** asserted here —
  `closed` is the buildable-now deliverable.

### temporal/obligation-not-dischargeable-in-ken (AC5, soundness)
- spec: `spec/70-behavioral/72-temporal.md §2`, `§7`, `§6.2`
- given: a temporal obligation `□(req → ◇resp)` over a **concrete system's**
  behaviors
- expect: there is **no way** to discharge the obligation **itself** inside Ken
  — no `▷`/modality, no internal model-check, no kernel decision procedure over
  the system's infinite traces; the **only** outcomes are (a) reason **about**
  the formula as data (TE-F1) or (b) **export + delegate** to `Ward` (TE-E),
  observable as the obligation's status being `delegated` and its discharge
  arriving only out-of-band (TE-E2)
- why: (soundness) AC5 — the reason-*with*-is-impossible face, the dual of
  TE-F1. **Both** faces asserted: *about* works (TE-F1 type-checks), *with*
  cannot (this). The impossibility is realized as the **absence** netted by
  TE-B1 (no modality) **plus** the delegated-only export path (TE-E) —
  discharging `□(req→◇resp)` of *this system* quantifies over its behaviors,
  which is `Ward`'s fragment (`72 §2`), not Ken's. **Disconfirming:** would a
  hidden internal model-checker satisfy this? **No** — TE-B1's grep + the
  delegated-only status would surface it. Observable consequence + named seal
  (the no-internal-discharge composition), per the B1 seal pattern rather than
  an awkward black-box case for "unrepresentable".

## Coverage map (AC → case)

| AC | Case |
|---|---|
| AC1 ordinary inert data, K1-admitted | TE-A1 **+** TE-A2 (positivity pair) |
| AC1 no kernel modality | TE-B1 (construct absent) **+** TE-B2 (conversion inert) |
| AC2 derived operators | TE-C1 (`◇`) + TE-C2 (`□`) + TE-C3 (`leadsto`) |
| AC3 surface → `delegated` | TE-D1 |
| AC4 export `T`/`delegated`, never `Q`, one-way | TE-E1 **+** TE-E2 (the pair) |
| AC5 reason-about, not -with | TE-F1 (about) **+** TE-F2 (with-impossible) |

## Cross-case sweep (group by the shared mechanism; assert agreement)

- **Verdict-mapping class is constant (`72 §5`).** Every `Temporal`-in-source
  obligation maps to **exactly** `delegated`/`T` (TE-E1) and **stays** there
  after a green `Ward` result (TE-E2); **no** case maps a `Temporal` value to
  `Q`/`P`/`unknown`. The mapping is total and constant — the verdict-mapping
  silence is foreclosed at source (no V3 gap to fill).
- **Strict-positivity pair is non-degenerate.** TE-A1 (first-order, accepted) +
  TE-A2 (HOAS, rejected) on the *same* datatype under two binder encodings is
  the **sole** net for "first-order binding is load-bearing"; neither alone
  (each is green-vs-green under an over-permissive / broken positivity check).
- **Derived-not-primitive agrees across operators.** TE-C1/C2/C3 each assert the
  elaborated head is in the `until`/`not`/`or`/`atom` core (never a modal
  constructor); the datatype's constructor set (`72 §3`) contains **no**
  `eventually`/`always`/`leadsto` — the three cases agree that the modalities
  are syntax, not constructors.
- **No-modality is a guard-gated absence (the B2 N1 analog).** TE-B1 (no modal
  construct) + TE-B2 (conversion inert) + TE-F2 (no internal discharge) are the
  **sole** net for "the kernel gained no temporal power" — the kernel cannot
  self-check the *absence* of a rule it does not have; conformance is the net,
  exactly as for B1's export-byte seals.
- **About / with boundary holds across the set.** The corpus exhibits **about**
  power (TE-A, TE-C, TE-F1 — all ordinary `declare_inductive` /
  `check_positivity` / `elim_Temporal`) and **zero** *with* power (TE-B, TE-E,
  TE-F2). TE-F1 (about: type-checks) and TE-F2 (with: impossible) are the two
  faces the spec mandates be asserted together (`72 §7`).

## Subsumed upstream — not re-derived

- **B1 export channel + `Σ` alphabet (`../export/seed-export.md`).** The `T`
  channel, the `delegated`→`T` projection fidelity, the one-way gate's *meaning*
  (EX-E1), and the `Σ` alphabet's reuse (EX-C1) are pinned by B1. These B2 cases
  **drive a real `temporal{}` source** through that channel (the B2-new wiring);
  they do **not** re-derive the channel or the alphabet (B1 fixed both, `71
  §5.2`; subsume-don't-proliferate).
- **Verdict / status meanings (`../../verify/seed-verify.md`, `21 §5`).**
  `proved`/`tested`/`unknown`/`delegated` and the four-way status are pinned
  upstream; these cases pin only the `Temporal`→`delegated` mapping and its
  export, not the status semantics.
- **L2 `data`/`elim`/positivity machinery (`../../kernel/seed-k1.md`, `14`).**
  The inductive declaration, `elim_D` generation, ι-reduction, and the
  strict-positivity check are pinned by the K1 seed; TE-A / TE-F **reuse** them
  on `Temporal`, asserting only the `Temporal`-specific shape (direct recursion,
  first-order binding).

## Deferred / `(oracle)`-tagged — the joint Ward encoding pass (`72 §3.1`)

The **exact constructor set + spelling** (whether duals like
`release`/weak-until are primitive or derived; the surface keyword per
constructor), the **`Pred Σ` atom language** (events only vs events + observable
state, over the B1 `Σ`, `36 §2`), the **fixpoint-variable form** (named vs de
Bruijn) and its *guardedness* well-formedness condition, the **satisfaction
relation `sat`'s shape** (finite vs ω-behaviors; the μ/ν interpretation —
`§6.2`), and the **ITF / `WardFormula` serialization** (`§6.3`) are an encoding
pass with `Ward`. These cases lock the **value-set + LTL/μ meaning + inert-data
+ strict-positivity + first-order-binding** and **`(oracle)`-tag the literal
spelling**: a rename after the spelling binds is a breaking export change (`71
§3.3`), so the cases refer to constructors by **concept** (`until`/`not`/`mu`)
and tag the surface token. The `compile : Temporal Σ → WardFormula` faithfulness
lemma (`§6.3`) is **B2/B3-joint** — its target spelling and full proof are
deferred; B2 pins the `Temporal` datatype it ranges over. It is distinct from
B3's `compile : Temporal Σ → Monitor` (`73 §2.4`) — **two sibling projections**
sharing the `Temporal Σ` source and the `Σ` alphabet, **not** a function
(consistent with the landed errata).
