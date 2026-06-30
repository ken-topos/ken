# Behavioral-export conformance — seed cases (B1)

Format: `../../README.md`. These pin the **assumption-boundary export emitter**
(`spec/70-behavioral/71-assumption-boundary.md`, impl-ready B1): the *generated*
five-part assume-guarantee contract `Q`/`P`/`Σ`/`T`/`G` that a family of `Ward`
consumers reads. The emitter is a **deterministic projection of already-verified
content** — it adds nothing to the trusted base and proves nothing new. These
cases net the **fidelity of that projection** (status → export field), not the
meaning of the verdicts it reads (that is `../../verify/`, subsumed below).

## Reading disciplines

**The observable is the projected export field — never a kernel verdict.** A
B1 case routes a real checked program through the emitter and asserts a
**structural property of the emitted contract**: which field a claim lands in,
its status tag, the `Σ` set, or the content hash. It is **not** elaboration
accept/reject (that is `30-surface`/IFC) and **not** the upstream `proved`/
`unknown`/`disproved` verdict (that is `21 §5`, `../../verify/`). What these
cases pin is the **verdict/status → field projection** (`71 §2.1`).

**Status → field is pinned at source (`71 §2.1`, faithful to `21 §5.3/§5.4`) —
no silence to fill.** The map is design-time-locked: `proved`→`Q`, `tested`→`P`,
`unknown`→`P`, `delegated`→`T`, `disproved`→(none). The conformance author
inherits **no classification gap** (the verdict-mapping silence is foreclosed
upstream, as in Sec1ct). These cases net the projection's *fidelity*, not a
choice of mapping.

**The proved↔assumed discriminator is structural (kernel-side), not a status
string.** `21 §5.4` *mandates* the conformance pin: a claim is in `Q` rather
than `P` **iff** its certificate `check`s **and** its goal is **not** a
postulate in `trusted_base()` (`18 §4/§5`) — a structural discriminator that
**flips**, not a string compared for equality. The load-bearing net is the
**non-degenerate pair on the same postcondition** (EX-A1 `proved`→`Q` *while*
EX-A2 hole→`P`): a
lazy emitter that trusts the untrusted V-layer's `proved` string, or buckets by
"has an `ensures` clause," lands **both** in `Q` (over-claim) — so a single
proved case is green-vs-green. Only the pair, one proposition under two kernel
states, pins the honesty direction. (The discriminating-pair pattern, recurring;
here the axis is `proved`↔`assumed`.)

**Locked structure vs deferred spelling (`71 §3.1`).** The five-part structure,
each field's value-set, the per-entry status tags, the cross-field invariants
**I1–I5**, and the content-hash stability discipline are **locked** (normative,
conformance-checkable). The **literal serialized field keys** are `(oracle)`-
tagged — `Ward` finalizes the wire token (`guarantees`/`guarantee`/`Q`). Cases
refer to a field by its **concept** (`Q`/`guarantees`) and `(oracle)`-tag the
literal key; they pin the value-set + invariants and **do not over-freeze** the
token (assert-at-locked-granularity — a rename after the spelling binds is a
breaking change, a new hash, `§3.3`).

**QA gate: real verified content → real projection.** Every case drives a
**real checked program** through the **actual emitter** and observes the
projected field (or its absence). A test that builds a synthetic export literal
and checks a field **guards nothing** and is not a conforming case.

**Projection-fidelity is the net (the B1 analog of N1).** The emitter is an
**untrusted projection**; the kernel never inspects the **serialized export
bytes** (`§6`, `§4.1`, `§5.1`), so no kernel check backstops a projection bug.
Exactly the erased-before-kernel omission-hole the security labels face (Sec1/
Sec1ct N1), here for the export: the discriminating check must live where the
untrusted projection happens. The two seals — **no-measure `G`** (`§4.1`) and
the **one-way gate** (`§5.1`) — are therefore **exhaustive-by-construction** in
the emitter (a measure is unrepresentable; no `proved`-writing edge exists), and
conformance pins their **observable consequence** plus names the seal, rather
than authoring an awkward black-box case for "unrepresentable" (the sealed-set
discipline, as with Sec1ct's `LeakSink`).

**Tags.** `(soundness)` = a real no-over-claim / one-way-flow / never-ship-a-
false-claim commitment of the **seam trust model** that must never regress —
netted **solely** by conformance (the kernel is blind to the export bytes).
`(oracle)` = the literal serialized field-key spellings, finalized by `Ward`.

## EX-A. The status → field projection — the no-over-claim pair (AC2/I1)

### export/proved-postcondition-projects-to-Q (AC2)
- spec: `spec/70-behavioral/71-assumption-boundary.md §2.1`, `21 §5.3/§5.4`,
  `25 §3`
- given: a checked `view` with `ensures result > 0` whose obligation
  **discharges** (verdict `proved`; certificate `check`s; goal **absent** from
  `trusted_base()`), run through the export emitter
- expect: the postcondition projects into **`Q`** (concept `guarantees`; literal
  key `(oracle)`) tagged status `proved`, and is **absent** from `P` and `T`
- why: AC2/I1 — a genuinely proved postcondition is the guaranteed half,
  exported under `Q` so a downstream may *assume* it. Half of the no-over-claim
  pair: **alone it is green-vs-green** (an emitter that buckets everything-with-
  `ensures` into `Q` also passes); the net is the pair with the hole case below.

### export/open-hole-postcondition-rides-P-as-unknown (AC2, soundness)
- spec: `spec/70-behavioral/71-assumption-boundary.md §2.1`, `21 §5.4`, `24 §2`
- given: the **same** `ensures result > 0`, but the proof is left an **open
  typed hole** (verdict `unknown`; the hole is a postulate of the goal, so the
  goal **appears** in this target's `trusted_base_delta` / `trusted_base()`),
  run through the emitter
- expect: the postcondition projects into **`P`** (concept `assumptions`) tagged
  status `unknown` — **never** `Q`
- why: (soundness) AC2/I1, the load-bearing **no-over-claim** net. The field
  **flips** with `trusted_base()` membership on the *same* proposition — a
  structural discriminator (`21 §5.4` mandates exactly this pin), not a self-
  reported status string. A lazy emitter that trusts the untrusted V-layer's
  `proved` string, or buckets by the presence of an `ensures` clause, lands this
  in `Q` (over-claim) → red. With EX-A1 this is the **non-degenerate
  distinguishing pair** (one proposition, two kernel states): neither case alone
  nets the discriminator. Projection-fidelity is the sole net — the kernel never
  re-checks the serialization.

## EX-B. Assumption visibility (AC3/I2)

### export/removing-assume-shrinks-P-and-changes-hash (AC3)
- spec: `spec/70-behavioral/71-assumption-boundary.md §2.1`, `§3.1` (I2),
  `§3.3`, `25 §3`
- given: a program with an explicit `assume φ` (and/or a postulate in
  `trusted_base_delta`); emit the export, then **remove the `assume`** (or
  discharge the postulate, shrinking the delta) and emit again
- expect: in the first export `φ` appears as a **`P`** entry tagged `tested` (an
  `assume`) / `unknown` (an open postulate); in the second the matching **`P`
  entry is gone** and the **export hash changes**
- why: AC3/I2 — assumption visibility: `P` is a *projection* of the live
  `trusted_base_delta` + `assume`s (`25 §3`), not a hand-authored list. An
  emitter that hard-codes or caches `P` shows no change → red. Couples to AC1
  (the hash is content-addressed, `§3.3`): a smaller assumed set is a different
  contract. (Also carries the `tested`→`P` arm of the projection sweep.)

## EX-C. Alphabet reuse (AC4/I3)

### export/alphabet-equals-perform-node-signatures (AC4)
- spec: `spec/70-behavioral/71-assumption-boundary.md §2.1`, `§3.1` (I3),
  `36 §2/§2.1`
- given: a checked program whose denotation performs **two distinct** effects
  (e.g. `Console.Write : String → Unit` and `State.Get : → S`, `36 §2.1`), with
  a `T` obligation that mentions one of those event symbols; run the emitter
- expect: `Σ` (concept `alphabet`) equals **exactly** the program's L5 perform-
  node signatures — asserted as **structural equality** to the denotation's
  signatures (no orphan symbol, no missing node); and every event symbol named
  by the `T` obligation (and by `G`) is a **member** of `Σ` (alphabet closure)
- why: AC4/I3 — `Σ` is *reuse, not reinvention*: the monitored alphabet **is**
  the interaction tree's `perform` nodes (`36 §2`). A re-derived or hand-listed
  second alphabet differs (an extra or missing signature) → red. Two distinct
  signatures (not a singleton) so a coincidental match cannot hide a dropped
  node. Closure pins that `T`/`G` range only over real events.

## EX-D. Generators carry support, never measure (AC5/I5)

### export/generators-carry-support-not-measure (AC5, soundness)
- spec: `spec/70-behavioral/71-assumption-boundary.md §4.1` (I5), `§4`,
  `61 §5a.2`
- given: a program with a refinement type `{x:A | φ}` and a `match` over its
  cases; run the emitter
- expect: `G` (concept `generators`) carries **partition + boundaries + case-
  decomposition only** — and **no** entry carries a weight / likelihood /
  probability; an attempt to attach a measure to `G` is **not representable** (a
  per-class weight does not type — a compile error, `§4.1`)
- why: (soundness) AC5/I5 — *support, never measure*. The seal is **exhaustive-
  by-construction** (no field, variant, or escape hatch into which a number
  reads as a likelihood — the same no-catch-all discipline as `LeakSink`,
  `61 §5a.2`). Conformance pins the **observable consequence** (a real program's
  `G` is partition-only); the type-level *unrepresentability* is the build-side
  seal, because the kernel never inspects the export bytes (`§4.1`) — the
  discriminating check must live where the untrusted projection happens.

## EX-E. The one-way gate — no promotion path (AC6/I4)

### export/delegated-obligation-never-promoted-to-proved (AC6, soundness)
- spec: `spec/70-behavioral/71-assumption-boundary.md §5.1` (I4), `§5`,
  `63 §5a`, `21 §5`
- given: a `delegated` `Temporal` obligation `T`; simulate a **`Ward`
  discharge** of it re-entering the Ken side **only** as a `trusted_base_delta`
  entry / a discharge-attestation record (`63 §5a`) — *not* as a kernel
  certificate; run the emitter
- expect: the obligation stays **`delegated`** (in `T`) / `tested` (in `P`) — it
  is **never** re-stamped `proved` and **never** appears in `Q`; and there is
  **no emitter code path** from a `Ward` verdict (or any classical result) to a
  `proved` status
- why: (soundness) AC6/I4, the **G-Ward-seam** gate. One-way flow is realized as
  the **absence of a code path**, not a runtime check — a guard-gated absence,
  named: *no `proved`-writing edge from a classical/`Ward`/delegated source
  exists* (not merely "the happy path doesn't take one"). A `Ward` "green" fed
  back leaves the entry `delegated`; an emitter with a promotion path stamps it
  `proved` → it surfaces in `Q` → red. Disconfirming question (absence-gate):
  *would a delegated obligation with a green `Ward` result land in `Q` under the
  bug this targets?* — yes, so the case flips.

## EX-F. The disproved boundary — never exported (`71 §2.1`)

### export/disproved-claim-never-exported (soundness)
- spec: `spec/70-behavioral/71-assumption-boundary.md §2.1`, `21 §5.1/§5.3`,
  `24 §3`
- given: a program with a **refuted** claim (verdict `disproved`, a
  countermodel); run the emitter
- expect: refuted claim is in **no** export field (`Q`/`P`/`T`/`Σ`/`G`); and
  — since a `disproved` verdict is a hard verification error (`24 §3`) — the
  target is **not shippable**: the build does **not** produce a shippable export
- why: (soundness) the second verdict-trichotomy boundary, dual to AC2's honesty
  direction: *a known-false claim is fixed, not shipped*. The projection is
  total over **exportable** claims, and a refuted one is not exportable
  (`21 §5.3` — no epistemic status, no field). An emitter that exports it
  (e.g. under `P` as `unknown`, or that ships an export despite the refutation)
  → red.

## EX-G. Reproducibility (AC1)

### export/same-program-same-export-hash (AC1)
- spec: `spec/70-behavioral/71-assumption-boundary.md §3.3`, `63 §2`
- given: the **same** checked program emitted **twice** (independent runs);
  compare the export hashes
- expect: the two exports yield the **identical hash** — a deterministic
  projection over a **canonical form** (deterministic field and entry order, a
  normalized form of each proposition), `§3.3`
- why: AC1 — *reproducible*: the hash links *this model* to *this build*
  (`63 §2`) only if it is a deterministic function of the verified content. A
  non-canonical serialization (map-iteration order, an embedded timestamp)
  yields a different hash across runs → red. Structural assertion on the
  **hash** (not "an export is produced"). Pairs with EX-B1's hash-sensitivity
  (different assumptions → different hash): a **constant** hash passes G1 but
  fails B1; a **non-canonical** hash fails G1 — the pair pins the hash as a
  proper content-address.

## Coverage map (AC / invariant → case)

| AC / invariant | Case |
|---|---|
| AC1 reproducible | EX-G1 (+ EX-B1 sensitivity) |
| AC2 / I1 no over-claim | EX-A1 **+** EX-A2 (the pair) |
| AC3 / I2 assumption visibility | EX-B1 |
| AC4 / I3 alphabet reuse + closure | EX-C1 |
| AC5 / I5 no measure | EX-D1 |
| AC6 / I4 one-way gate | EX-E1 |
| `71 §2.1` disproved boundary | EX-F1 |
| `tested`→`P` arm | carried by EX-B1 |

## Cross-case sweep (group by the status-projection class; assert agreement)

- **Status-projection class agrees.** Each row of the `71 §2.1` table is pinned:
  {`proved`→`Q`} EX-A1 · {`tested`→`P`, `unknown`→`P`} EX-B1 (`tested`) + EX-A2
  (`unknown`) · {`delegated`→`T`} EX-E1 · {`disproved`→none} EX-F1. The two rows
  that share a target field (`tested`/`unknown` → `P`) **agree on field** while
  differing on tag.
- **Boundary invariant 1 — honesty (no non-`proved` claim in `Q`, AC2/I1).** Of
  the whole set, only EX-A1 (genuinely proved) reaches `Q`; EX-A2 (`unknown`),
  EX-B1 (`tested`/`unknown`), EX-E1 (`delegated`), EX-F1 (`disproved`) each land
  outside `Q`. I1 holds across the corpus.
- **Boundary invariant 2 — one-way (no `Ward`/classical result in `proved`,
  AC6/I4).** EX-E1 pins it directly (no promotion edge); EX-F1 reinforces (no
  false claim shipped). I4 holds.
- **The honesty pair is non-degenerate.** EX-A1/EX-A2 on the *same*
  postcondition under two kernel states are the **sole** net for the structural
  discriminator — neither alone (each is green-vs-green under the string-trust /
  `ensures`-bucket bug).
- **The hash pair is non-degenerate.** EX-G1 (determinism) + EX-B1 (sensitivity)
  — a *constant* hash passes one and fails the other, a *non-canonical* hash the
  reverse.
- **Projection-fidelity is the net (B1 analog of N1).** The emitter is an
  untrusted projection; the kernel does not re-check the serialization (`§6`,
  `§4.1`, `§5.1`). So the `(soundness)` cases {EX-A2, EX-D1, EX-E1, EX-F1} are
  the **sole** net for over-claim / measure-leak / promotion / false-ship — the
  erased-before-kernel omission-hole, here for the export bytes.

## Subsumed upstream (`../../verify/`) — not re-derived

The **meaning** of the verdicts and statuses these cases project — `proved` (a
kernel-re-checked certificate), `unknown` (a typed hole = a postulate),
`disproved` (a countermodel), and the kernel re-check itself — is already pinned
by `../../verify/seed-verify.md` (`verify/proved-postcondition`, `verify/
incomplete-with-hole`, `verify/disproved-with-countermodel`) and `21 §5`. These
B1 cases **do not re-derive** that meaning; they pin only the **projection** of
an already-established verdict/status into its export field (subsume-don't-
proliferate).

## Build-sequencing note

- B1 fixes the `T` **channel** and the `Σ` **alphabet**; the `Temporal`
  **datatype** and the `compile : Temporal Σ → WardFormula` faithfulness lemma
  are **B2/B3** (`§5.2`). No case here asserts `compile` or a `Temporal`
  constructor set — only the channel (`delegated` values + status + the `Σ` they
  range over).
- The literal field-key spellings are `(oracle)` — `Ward` finalizes the wire
  token; a rename after it binds is a breaking change (`§3.3`). Conformance pins
  the concept + value-set + invariants and tags the literal key, never freezing
  it (assert-at-locked-granularity).
- The Sec1ct CT-in-parameter promise (`61 §5a.4`) is a boundary-`Q`/`P` producer
  feeding the `P` channel (`§2.1`); its field shape is coordinated via spec, not
  pre-bound here (defer-spelling-not-concept).
