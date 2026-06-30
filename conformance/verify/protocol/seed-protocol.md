# The machine-readable diagnostic protocol (T1) conformance — seed cases

Format: `../../README.md`. These pin **WS-V/WS-T T1** — the wire serialization
(`25-protocol.md`) of a verification result: the toolchain emits, per target, a
**structured, versioned, schema-valid JSON document** an agent consumes without
scraping human text. T1 **serializes** the verdict V3 produced (`23 §1.2`) and
the diagnostic V4 derived (`24`); it **never re-decides** either. Grounded in
the **landed** `25-protocol.md` (`§wire-contract`, `§2`–§9), the verdict
trichotomy + routing totality it serializes (`23 §1.2`/`§1.3`/`§2.1`/`§5`), the
four diagnostic shapes (`24 §1`–§4, determinism `§6`, no-regression `§7`), the
obligation id + stability model (`22 §1`, `21 §5.1`–`§5.4`), `trusted_base()`
(`18 §5`), and the Heyting structure (`16 §1`). The prototype is not mounted;
none of these required it.

**The wire cardinal rule — fidelity to the verdict (`25 §wire-contract`).**
Every status string, every `verdict` tag, every diagnostic `kind`, and every
`suggested_actions` region on the wire is a **projection of the one V3
verdict**, copied at serialization — **not** recomputed from the JSON evidence.
The serializer settles nothing: the kernel decided `proved`/not via the
certificate (`23 §1.3`), V4 rendered the structured value (`24`), and T1 only
*encodes* it. So `verdict:"false" ⟺ obligation refuted ⟺ doc disproved`, and
`verdict:"unknown" ⟺ open ⟺ incomplete` — three renderings of one source, which
**cannot disagree**.

**The failure mode T1 guards is *infidelity*, not unsoundness (★★ untrusted).**
A T1 bug is a **malformed, lossy, or unstable message** — an agentic-UX/contract
regression, **never** unsoundness (the kernel already settled the verdict). So
the load-bearing class here is **fidelity** on the wire: a relabel
(`unknown`↔`false`), a lossy round-trip, a silently-dropped diagnostic, a broken
stable-field contract, or a non-deterministic document. Cases tagged
**(fidelity)** encode this T1 commitment — the wire analog of `(soundness)` —
and must never regress. The one genuinely **kernel-structural** reuse is the
`trusted_base_delta` honesty guard (every open hole is a *listed* postulate;
empty ⟺ nothing assumed — `18 §5`, `21 §5.4`); it is tagged **(soundness)**.

**The Glivenko cross-case invariant on the wire (`25 §wire-contract`, `23
§5`).** A **classically-valid** goal serializes to `status:"incomplete"` /
`verdict:"unknown"`, **never** `disproved`/`"false"`. By Glivenko, `¬¬φ` is
intuitionistically provable iff `φ` is classically provable, so for a
classically-valid `φ`, `¬φ` is unprovable and **no world forces `¬φ`** — the
discriminator is *is `¬φ` forced*, nothing else. The discriminating pair on
shared abstract-atom metatheory is **`p ∧ ¬p` → `"false"`** (`¬(p∧¬p)` is a
theorem ⇒ `¬φ` forced) vs **`p ∨ ¬p` → `"unknown"`** (`¬(p∨¬p)` is *false* ⇒
neither forced). The class members (B2, B4) must **agree**, asserted by the
cross-case sweep, not just per-case flip.

**Normative boundary — value-sets locked, spellings `(oracle)` (LP-1,
spec-leader confirmed `evt_2c00xrv7th03p`).** The `25` preamble + §8 delegate
the exact JSON field **names** to the agent-team software; binding the literal
wire spelling now would pre-empt that. So **every literal field name in this
seed** — `status`, `verdict`, `kind`, `id`, `hole_id`,
`worlds`/`order`/`forcing`/`failure`, `trusted_base_delta`,
`suggested_actions`, `region`, `schema` — is an **`(oracle)` reference
spelling** tracking `25`'s reference schema, to be reconciled against the
finalized field names when the agent-team software lands. What is
**normative-locked** and what each case actually asserts: the **value-sets**
(`status ∈ {proved,disproved,incomplete}`, `verdict ∈ {false,unknown}`, `kind ∈
{countermodel,hole,decomposition,slice}`, `region ∈ {false,unknown}`), the
**cross-field agreement** invariants, the **id/`hole_id`-stability** semantics,
the **`trusted_base_delta`-emptiness** semantics, and the **round-trip /
determinism** properties. These hold regardless of the final spelling.

**Vocabulary (reconcile, not cite).** The `false`/`unknown` discriminator **is**
the countermodel `verdict` field, carried verbatim from V3 — there is **no**
independent `is_false` flag (`25 §wire-contract`, `24 §1`). A document `status`
is the **rollup** over its obligations with precedence `refuted ⊐ open ⊐
discharged` (`25 §3`) — distinct from V1's per-claim *epistemic* status `21
§5.2`, which an agent reads separately. A **typed hole** is an obligation
admitted as a postulate of `φ` (`22 §1`, `24 §2`), enumerated by
`trusted_base()` (`18 §5`). **Round-trip-lossless** = deserialize∘serialize is
identity on a *single* document (full surface, its own `stats` included);
**determinism** (`24 §6`) = same program+spec+prover across *runs* → byte-stable
**modulo** `stats.ms` + display strings (`goal.pretty`, `bridge`) — two distinct
properties, two distinct cases. **No case is `proved`-clean unless
`trusted_base_delta` is empty** (the honesty guard, `21 §5.4`).

---

## A. Round-trip — each status + diagnostic kind serializes lossless

### verify/protocol/proved-result-empty-diagnostics-and-delta
- spec: `25 §3`/§9 (no-regression), `24 §7` (AC5), `21 §5.4`
- given: a target every one of whose obligations is V3 `proved` (each
  certificate `check`s, `23 §1.3`)
- expect: the document has `status:"proved"`, **every** obligation
  `status:"discharged"` with `diagnostic:null`, and `trusted_base_delta:[]`
  (empty); it round-trips schema-valid. `|diagnostics| = 0`.
- why: (fidelity) no-regression — a fully-`proved` run emits **zero**
  diagnostics and an **empty** trusted-base delta (empty ⟺ nothing assumed).
  Flip vs a serializer that emits a "proved" diagnostic, or a non-empty delta
  for a clean result → red.

### verify/protocol/countermodel-kind-round-trips-lossless
- spec: `25 §5` (countermodel shape), `24 §1`
- given: a V3 `disproved` verdict carrying a Kripke countermodel (worlds `w0 ≤
  w1`, per-world forcing, a failure world) for a refuted `φ`
- expect: serializes to a `kind:"countermodel"` diagnostic with
  `verdict:"false"` and all of `worlds` / `order` / `forcing` / `failure`
  present; deserialize∘serialize returns the **same** document — every world,
  the `≤` pair list, the per-world forcing sets, and
  `failure.{world,subformula}` survive unchanged.
- why: (fidelity) each diagnostic kind round-trips without loss — the model an
  agent consumes is exactly V3's, on the stable surface. Flip vs a lossy encoder
  that drops the `order` preorder or collapses `forcing` → red.

### verify/protocol/hole-kind-round-trips-and-lists-in-delta
- spec: `25 §3`/§5 (hole shape, `trusted_base_delta`), `24 §2`, `18 §5`, `21
  §5.4`
- given: a V3 `unknown` verdict carrying a typed hole `?h : φ` admitted as a
  postulate
- expect: `status:"incomplete"`, obligation `status:"open"`, diagnostic
  `kind:"hole"` with a stable `hole_id`, `goal`, `context`, `origin`,
  `runtime:"unknown"`; **and** the hole appears as a postulate in
  `trusted_base_delta` (non-empty). Round-trips lossless.
- why: (soundness) the honesty guard on the wire — every open hole is a *listed*
  postulate, so an agent/reviewer sees exactly what is assumed. A hole present
  in the diagnostic but **absent** from `trusted_base_delta` is a
  silent-omission infidelity: the delta would read "nothing assumed" though `φ`
  is. Completeness of the delta is the **sole** backstop for the omission —
  guard it directly.

### verify/protocol/decomposition-and-slice-kinds-round-trip
- spec: `25 §5`, `24 §3`/§4
- given: (a) a three-region `decomposition` diagnostic (true/false/unknown
  regions); (b) a `slice` diagnostic (`missing_hypothesis` ψ, `sufficient:true`)
  for an `unknown` obligation
- expect: both serialize schema-valid and round-trip lossless —
  `kind:"decomposition"` carrying the three region fields; `kind:"slice"`
  carrying `missing_hypothesis`, `bridge`, `sufficient:true`. Completes
  round-trip coverage of all **four** kinds.
- why: (fidelity) every one of the four `24` mechanisms has a faithful wire
  image; no kind is un-serializable. With the two cases above this covers
  `{countermodel, hole, decomposition, slice}`.

## B. The false-vs-unknown discriminator on the wire

### verify/protocol/refuted-goal-false-tag-forcing-world
- spec: `25 §wire-contract`/§5, `24 §1`, `23 §5`
- given: `φ = p ∧ ¬p` (genuinely refutable — `¬(p∧¬p)` is an intuitionistic
  theorem) → V3 `disproved` + a countermodel forcing `¬φ`
- expect: doc `status:"disproved"`, obligation `status:"refuted"`,
  `verdict:"false"`, `failure` names a world that **forces `¬φ`**;
  `suggested_actions = [fix_counterexample]` (region `false`) **only**.
- why: (fidelity) the genuinely-refutable `"false"` exemplar — `verdict:"false"`
  ⟺ some world forces `¬φ` (the false-side of the discriminator). Cross-case
  pair with the next case; note an *abstract atom* `p` would be `unknown`, not
  `false` — only a refutable `φ` lands here.

### verify/protocol/unknown-goal-unknown-tag-no-forcing-world
- spec: `25 §wire-contract`/§5, `24 §1`, `23 §5`
- given: `φ = p ∨ ¬p` (`¬¬φ` holds; `¬(p∨¬p)` is false ⇒ not refutable) → V3
  `unknown` + a typed hole
- expect: doc `status:"incomplete"`, obligation `status:"open"`,
  `verdict:"unknown"`, **no** forcing world (the model fails to force `φ` while
  `¬¬φ` holds); `suggested_actions` drawn from the `unknown` set
  (`add_precondition` / `provide_lemma` / `case_split` / …), **never**
  `fix_counterexample`.
- why: (fidelity) flip — relabeling this `unknown` as `"false"` (or attaching a
  `fix_counterexample` action) is the load-bearing wire infidelity. `p ∨ ¬p` is
  classically valid ⇒ never `disproved` (Glivenko). Pairs with the previous case
  on shared abstract-atom metatheory.

### verify/protocol/false-unknown-non-confusable-roundtrip
- spec: `25 §8`/§9 (the discriminating round-trip)
- given: the two documents from the two cases above — serialize(`p ∧ ¬p` result)
  and serialize(`p ∨ ¬p` result)
- expect: the two messages are **distinct and non-confusable** — they differ in
  `verdict` (`"false"` vs `"unknown"`), document `status` (`"disproved"` vs
  `"incomplete"`), obligation `status` (`"refuted"` vs `"open"`), and the legal
  `suggested_actions` set. No single field of one could be read as the other.
- why: (fidelity) the explicit T1 round-trip property — a `false` and an
  `unknown` go to distinct, non-confusable messages. Flip vs a serializer that
  collapses both to a generic "failed" / drops the `verdict` tag → the two
  documents become equal → red. V4's `false`/`unknown` discriminator carried to
  the wire.

### verify/protocol/glivenko-wire-sweep-classically-valid-never-false
- spec: `25 §wire-contract` (Glivenko on the wire), `23 §5`, `24 §1`/§3
- given: the classically-valid class `{ p ∨ ¬p , ¬¬p ⇒ p }`, each verified
- expect: **every** member serializes to `status:"incomplete"` /
  `verdict:"unknown"` — **never** `disproved`/`"false"`; contrast the
  genuinely-refutable `p ∧ ¬p` → `"false"`. The class **agrees**.
- why: (fidelity) the cross-case metatheory-consistency sweep on the wire —
  grouped by shared class, all members agree; a single member serializing
  `"false"` is a relabel. Reusable invariant: a classically-valid goal is never
  `disproved`/`false` (Glivenko: `¬¬φ` provable ⇒ `¬φ` unprovable ⇒ no world
  forces `¬φ`). Asserted by the **sweep**, not just per-case flip.

## C. Cross-walk projection, rollup precedence, totality

### verify/protocol/three-renderings-agree-one-source
- spec: `25 §wire-contract` (one source, three fields), §3/§4
- given: any non-`proved` obligation, serialized into its document
- expect: the obligation `status`, the document `status`, and (when present) the
  countermodel `verdict` are **three renderings of the one V3 verdict** and
  **agree**: `verdict:"false"` ⟺ obligation `refuted` ⟺ doc `disproved`;
  `verdict:"unknown"` ⟺ `open` ⟺ `incomplete`. A document where they disagree is
  rejected as a fidelity violation.
- why: (fidelity) the serializer **projects** one verdict into three fields,
  never recomputes any independently — the wire image of the verdict-mapping
  trap (an output mapped to a status must be pinned, never left for the reader
  to fill). Flip vs an encoder that derives the doc `status` from the JSON
  evidence rather than copying V3's verdict → drift → red.

### verify/protocol/mixed-rollup-refuted-dominates
- spec: `25 §3` (rollup precedence), `21 §5.3`
- given: a target with mixed-status obligations — at least one `refuted`, at
  least one `open`, the rest `discharged`
- expect: the document `status` rolls up to `"disproved"` (precedence `refuted ⊐
  open ⊐ discharged`) — **not** `"incomplete"`.
- why: (fidelity) the mixed-verdict rollup is pinned at the source, not left to
  the serializer — a refuted obligation is a hard error that **dominates** open
  holes. Flip vs a "≥1 open ⇒ incomplete" rollup that ignores refuted-first →
  the doc reads `incomplete`, masking a hard error as soft, and an agent
  "supplies facts" when it must "fix the counterexample" → green(`disproved`)-vs
  -red(`incomplete`).

### verify/protocol/non-discharged-implies-non-null-diagnostic
- spec: `25 §4`/§6 (totality, "no silent failure"), `23 §2.1`, `24 §7`
- given: an obligation with `status ≠ discharged` (i.e. `open` or `refuted`)
- expect: its `diagnostic` is **non-null** and carries the §5 shape whose
  `verdict`/region matches `status`; conversely a `discharged` obligation has
  `diagnostic:null`. A non-discharged obligation serialized with
  `diagnostic:null` is **invalid** (rejected).
- why: (fidelity) totality — `diagnostic:null` **iff** `discharged`; the wire
  image of routing totality (`23 §2.1`, exhaustive-by-construction, no silent
  skip). Absence-gate: the guard is the **iff**, not coincidence — under the bug
  this targets (a serializer that drops a diagnostic for an obligation it can't
  render), the document **fails validation** rather than silently reading as if
  discharged.

## D. The stability surface — the contract agents code against

### verify/protocol/stable-field-drop-or-rename-rejected
- spec: `25 §6` (stable surface), §8 (required fields), §9 ("renaming
  `countermodel.verdict` fails") · `(oracle)`: the field *names* are reference
- given: two mutated documents — (a) the stable required field
  `obligations[].id` **removed**; (b) the stable `countermodel.verdict` field
  **renamed** (e.g. → `judgment`), value unchanged
- expect: the §8 reference validator **rejects** both — `id` and the `verdict`
  discriminator are required stable fields; a rename nets to a missing required
  field (the renamed field is an unknown-optional that is *ignored*, so
  drop-required(REJECT) + add-unknown(ACCEPT) → net **REJECT**).
- why: (fidelity) the stable contract — dropping or renaming a stable field is a
  breaking change (bump `schema`) and a conformance failure, never a silent
  migration. Flip vs a lax validator that treats `id`/`verdict` as optional, or
  keys on a positional/loose match that accepts the rename → accepts the mutated
  doc → an agent loses correlation / the discriminator → red.

### verify/protocol/additive-unknown-field-accepted
- spec: `25 §6` (versioned/additive), §7 (agent ignores unknowns) · `(oracle)`:
  field *names* are reference
- given: a schema-valid document extended with (a) a new **optional** field
  somewhere, and (b) a `suggested_actions[]` entry with an **unknown** `kind`
- expect: the §8 reference validator **accepts** it; a pinned (major-`schema`)
  agent **ignores** the unknown field and treats the unknown action `kind` as a
  plain advisory hint.
- why: (fidelity) the stable/versioned boundary — additive changes are
  non-breaking; forward-compatibility holds. The flip-complement of the
  drop/rename case: drop/rename a **stable** field → reject; add an **unknown**
  field → accept. Flip vs an over-strict validator that rejects unknown fields →
  breaks forward-compat → red.

### verify/protocol/obligation-id-stable-across-unrelated-edit
- spec: `25 §4`/§6, `22 §1`, `24 §6`
- given: a source file with obligation `ob:divide#post.0`; then an **unrelated
  edit elsewhere that shifts line numbers** — inserting a new definition *above*
  `divide` — re-verified (same program structure for `divide`)
- expect: `obligations[].id` for the `divide` postcondition is **unchanged**
  across the two runs; likewise `hole.hole_id` is unchanged (a function of
  obligation **provenance**, not allocation order). Only an edit *to the clause
  itself* may change them.
- why: (fidelity) `id`/`hole_id` are functions of **program structure** (the
  clause path), invariant under a line-shifting unrelated edit — so an agent
  correlates a diagnostic across edit/verify cycles. **Non-degenerate by
  construction**: the edit shifts lines, so the case discriminates against the
  real bug (id keyed on **line-number / allocation order**) →
  green(stable)-vs-red(renumbered). A no-op edit would be green-vs-green.

## E. The agent loop + determinism

### verify/protocol/agent-pivots-on-status-no-text-scraping
- spec: `25 §7` (the loop), §9 (agent-consumable)
- given: a serialized verdict document, with **no** access to human-rendered
  text (`goal.pretty`, `bridge` withheld)
- expect: from the machine fields alone an agent locates the actionable signal —
  `status:"disproved"` ⇒ fix-path (read `failure.{world,subformula}` +
  `provenance.span`, apply the `fix_counterexample` action);
  `status:"incomplete"` ⇒ supply-path (apply `add_precondition` /
  `provide_lemma` from `suggested_actions`). The pivot bit (fix-spec vs
  supply-facts) is reachable without parsing any human string.
- why: (fidelity) agent-consumable — the actionable signal lives in machine
  fields (`status`, `verdict`, `suggested_actions`, `provenance.span`), never
  only in display text. Flip vs a serializer that carries the fix-vs-supply
  distinction only in `goal.pretty` → an agent cannot pivot mechanically → red.

### verify/protocol/deterministic-modulo-stats-and-display
- spec: `25 §6` (deterministic, byte-stable), `24 §6`
- given: two verification **runs** of the **same** program + spec + prover
  version
- expect: the two documents are **byte-stable on the stable surface** —
  identical `status`, obligation `id`s, `verdict` tags, countermodel, `hole_id`s
  — differing **only** in `stats.ms` and human-display strings (`goal.pretty`,
  `bridge`), which are excluded from byte-stability.
- why: (fidelity) determinism (`24 §6` MUST) so diffs across edit/verify cycles
  are meaningful. **Distinct from round-trip** (which preserves a *single*
  document's full surface, `stats` included); determinism is across *runs*,
  modulo the non-deterministic `stats.ms`. Flip vs a serializer that lets
  `hole_id`/countermodel ordering vary run-to-run → spurious diffs → red.
  Minimality of the countermodel/slice is a `24 §6` **SHOULD**, not asserted
  here.

---

## Coverage map

Frame `25 §9` load-bearing properties + the six elaboration directions:

| Property / direction | Cases |
|---|---|
| Faithful + lossless (each verdict + kind round-trips) | A1, A2, A3, A4, B3 |
| `false` vs `unknown` honest + preserved on the wire | B1, B2, B3, B4 |
| Stable contract (stable vs versioned; rename/drop fails) | D1, D2 |
| id / `hole_id` stability across unrelated edits | D3 |
| Agent-consumable (status pivot, no text scraping) | E1 |
| No regression (`proved` → empty diagnostics + empty delta) | A1 |
| Cross-walk projection (one source, three fields) + rollup | C1, C2 |
| Totality (`diagnostic:null` iff discharged, no silent failure) | C3 |
| Determinism (MUST, byte-stable modulo `stats.ms` + display) | E2 |
| `trusted_base_delta` honesty (every hole a listed postulate) | A3 |

Six spec-author elaboration directions (`evt_11n5ncrkqt1e6`): ① cross-walk +
rollup → C1, C2 · ② fidelity false/unknown, no `is_false` flag → B3, C1 · ③ four
shapes reconciled to `24` → A2, A3, A4 · ④ stability split → D1, D2 · ⑤ agent
loop + schema invariants → E1, C3 · ⑥ laundered-cite fix → reconcile-don't-cite
(header; no `12 §5.x` inherited).

## Cross-case consistency sweep (pre-handoff gate)

Grouped by shared metatheory class, the wire verdicts must **agree**:
- **Classically-valid / abstract-atom class** = `{ B2 (p ∨ ¬p), B4 (p ∨ ¬p, ¬¬p
  ⇒ p) }` — all `status:"incomplete"` / `verdict:"unknown"`, **never**
  `"false"`. Contrast the genuinely-refutable `B1 (p ∧ ¬p)` → `"false"`. The
  invariant: a classically-valid goal is never `disproved`/`false` (Glivenko).
- **Three-renderings agreement class** = `{ A2, A3, B1, B2, C1 }` — wherever a
  countermodel `verdict`, an obligation `status`, and a doc `status` co-occur,
  the cross-walk equivalences hold (`false ⟺ refuted ⟺ disproved`, `unknown ⟺
  open ⟺ incomplete`). No case may assert a mismatched triple.

## Build-sequencing note

T1-build follows V4-build on Team Verify; it serializes V3/V4's already-decided
output, so it depends on the V3 verdict + V4 diagnostic shapes being on `main`
(both are). The four diagnostic value shapes are landed in `24 §1`–§4; the wire
**spellings** are `(oracle)` and reconcile with the finalized field names when
the agent-team software (strategy G7) lands — at which point the `(oracle)` tags
clear and the value-set/invariant assertions stand unchanged.
