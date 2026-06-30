# Trace-conformance conformance — seed cases (B3)

Format: `../../README.md`. These pin the **trace/instrumentation contract**
(`spec/70-behavioral/73-conformance.md`, impl-ready B3): the *generated*
companion to the B1 export that makes a running program **observable in the
model's vocabulary `Σ`** — the concrete `Σ`-event schema at the effect boundary,
the correlation keys for multi-`space` traces, the runtime `Q`/`P` assertion
points, and the monitor projected from `T`. The contract is an **untrusted
projection of already-verified content + runtime instrumentation** — it adds
nothing to the trusted base and proves nothing new (`73`, banner; `71 §6`).
These cases net the **fidelity of that projection and instrumentation** (events
faithful to `Σ`, assertions/monitor faithful to the export, one-way flow), not
the meaning of the verdicts/effects it reads (those are `../../verify/`,
`../export/`, and `../../surface/effects/`, subsumed below).

## Reading disciplines

**The observable is the emitted trace artifact from a real run — never a kernel
verdict, never a synthetic literal.** A B3 case runs a **real program** through
the **actual instrumentation + emitter** and asserts a **structural property of
the emitted contract**: which `Σ` member an event concretizes, the correlation
keys it carries, which point a `Q`/`P` projects to, or that the monitor
is the image of `T`. It is **not** elaboration accept/reject (`30-surface`/IFC),
**not** the upstream `proved`/`unknown` verdict (`21 §5`, `../../verify/`), and
**not** the export-field projection (`71 §2.1`, `../export/`). What these cases
pin is the **runtime concretization** of an already-established export.

**A trace event is an ITF *witness*, not a *claim* (`71 §3.2`, `73 §2.1/§3`).**
An event carries op/response **values** and has **no epistemic status** — it is
never tagged `proved`/`tested`/`delegated`. A green trace is *evidence for* a
`delegated` `T`, **never a promotion** of it (the one-way gate, TR-F). The net
here is **projection/instrumentation fidelity + one-way flow**, not verdict
meaning (subsumed upstream).

**Locked structure vs deferred spelling (`73 §2.6`, `71 §3.1`).** The five-part
contract (event schema, correlation keys, `Q`/`P` assertion points, monitor
projection, ITF serialization), each part's value-set, the correlation-key set,
and the cross-field invariants **TC1–TC5** are **locked** (normative,
checkable). The **literal serialized field keys** are `(oracle)`-tagged — `Ward`
finalizes the wire token; a rename after it binds is a breaking change (a new
hash, `71 §3.3`). Cases refer to a field/part by its **concept** and
`(oracle)`-tag the literal key (assert-at-locked-granularity).

**B2 staging — net what is buildable on the landed `T` channel + `36 §2` perform
points (`73 §2.4`).** `72-temporal.md` is DRAFT v0: the concrete `Temporal`
**datatype** (constructors, the `Pred Σ` atom language) and the full `compile :
Temporal Σ → WardFormula` faithfulness lemma are **B2-deferred**. So the monitor
case (TR-E) pins the **projection-as-projection** over the landed `T` channel
(the monitor is the image of `T`, changes with `T`) **now**, and `(oracle)`-tags
the **semantic** Büchi acceptance of a concrete `Temporal` formula. No case
asserts a `Temporal` constructor set or a `compile` faithfulness equation.

**QA gate: real run → real instrumentation.** Every case drives a **real
program** through the **actual** `drive_H` instrumentation (`36 §2` perform
points) and the **actual** emitter, observing the real events / correlation keys
/ assertion points / monitor projection. A test that builds a synthetic trace
struct and checks a field **guards nothing** and is not a conforming case.

**Projection-fidelity is the net (B3 analog of B1's N1 / `../export/`).** The
contract is an **untrusted projection + instrumentation**; the kernel never
inspects the **trace/contract bytes** (`73` banner, `71 §6/§5.1`), so no kernel
check catches a concretization bug, an out-of-boundary emission, or a promotion
path. The discriminating check must live where the projection/instrumentation
happens. The two trust-model seals — **effect-boundary containment** (TC2, no
event off the perform site) and the **one-way gate** (TC5, no monitor verdict to
`proved`) — are exhaustive-by-construction in the emitter/instrumentation, and
conformance pins their **observable consequence** plus names the seal.

**Tags.** `(soundness)` = a one-way-flow / bounded-boundary commitment of the
**seam trust model** that must never regress — netted **solely** by conformance
(the kernel is blind to the trace bytes). `(oracle)` = the literal serialized
field-key spellings (Ward) **and** the concrete `Temporal` surface / `compile`
signature / Büchi-monitor acceptance (B2).

## TR-A. The `Σ`-event schema — concretizes `Σ` (AC1/TC1)

### trace/event-symbol-is-sigma-member (AC1)
- spec: `spec/70-behavioral/73-conformance.md §2.1` (TC1), `71 §2.1` (I3),
  `36 §2/§2.1/§3.1`
- given: a checked program whose denotation performs **two distinct** effects
  (e.g. `Console.Write : String → Unit` and `State.Get : → S`, `36 §2.1`); run
  it through the instrumentation and observe the emitted `Σ`-events
- expect: each event's `effect`/`op` symbol (concept; literal key `(oracle)`) is
  a **member of B1's `Σ`** (`export.rs`) — the per-firing concretization
  of a perform-node signature; **no** event whose symbol is **outside** `Σ` (no
  second alphabet) and **no** `Σ` member that is a perform point yet **cannot**
  emit (no orphan, no unreachable node)
- why: AC1/TC1 — `Σ` is *reuse, not reinvention*: the monitored event vocabulary
  **is** the interaction tree's `perform` nodes (`36 §3.1`). The event
  schema is a 1:1 **concretization** of `Σ`, not a re-derived alphabet. **Two
  distinct** effects (not a singleton) so a coincidental match cannot hide a
  dropped/renamed node. A build inventing a second event alphabet (an extra or
  renamed symbol) emits a symbol ∉ `Σ` → red. (Runtime mirror of `../export/`
  EX-C1; that case pins the *static* `Σ`, this one pins the *runtime event*
  concretizing it.)

## TR-B. Effect-boundary containment — bounded overhead (AC2/TC2)

### trace/no-event-outside-perform-point (AC2, soundness)
- spec: `spec/70-behavioral/73-conformance.md §2/§2.6` (TC2), `36 §2`
- given: a program that **interleaves pure reduction with perform points** — `K`
  perform-nodes (`Vis` firings) separated by non-trivial pure sub-computations
  (β/ι/`Ret` steps that perform nothing); run it through the instrumentation
- expect: **exactly `K` events** are emitted — one per `Vis` firing — and the
  pure sub-computation between two performs emits **no** event (a structural
  absence-assertion: event count == `Vis`-firing count, and pure steps are
  silent)
- why: (soundness) AC2/TC2 — instrumentation sits **only** at the boundary
  (the single `Vis` site in `drive_H`, `eval.rs`), which is what makes overhead
  **instrumentation-dominated and bounded** rather than pervasive. A build that
  emits on a non-perform reduction (e.g. on every `whnf`, or on `Ret`) produces
  **more than `K`** events → red. Discriminating on **count**: correct = `K`;
  the instrument-everywhere bug = `K` + (pure-step count). The kernel never sees
  the trace, so this containment is netted **solely** here.

## TR-C. Multi-space correlation (AC3/TC3)

### trace/correlated-events-link-uncorrelated-dont (AC3)
- spec: `spec/70-behavioral/73-conformance.md §2.2` (TC3), `36 §4/§4.1/§4.4`,
  `41 §3`
- given: **two distinct `space`s** each performing an op, plus a **cross-space
  message** (a `send` in space A of a content-addressed value, a `receive` of it
  in space B, `36 §4.4`); run through the instrumentation
- expect: (a) **every** event carries its **space identity** (the space's single
  effect label, `36 §4.1`); (b) the `send`/`receive` events carry **matching
  message provenance** (the message value's content address, `41 §3`) that
  **links** them; (c) two **unrelated** events (different spaces, no message
  between them) **do not** share provenance — a monitor can place each event in
  its per-space trace and stitch the global trace **only** across matched
  provenance
- why: AC3/TC3 — correlation completeness: offline model-checking glossed
  identity; a live monitor cannot. The case asserts **both directions** — linked
  *and* not-linked — because a build that assigns a **constant/global**
  correlation key would make *everything* link (passing a one-directional
  "do they link?" check vacuously); only the pair (correlated link **while**
  uncorrelated don't) nets it. Dropping the space identity, or the provenance
  token, breaks reconstruction → red. (The discriminating-pair pattern; here the
  axis is correlated↔uncorrelated.)

## TR-D. Runtime `Q`/`P` assertion points (AC4/TC4)

### trace/Q-P-assertion-points-project-from-export (AC4)
- spec: `spec/70-behavioral/73-conformance.md §2.3` (TC4), `71 §2.1`, `21 §5`
- given: a program with a **proved** per-`space` invariant `Q` and a **boundary
  assumption** `P` (an explicit `assume` / a `trusted_base_delta` postulate);
  run the emitter, then **change the export's `Q`/`P`** (discharge vs leave a
  hole; remove the `assume`) and run again
- expect: in the first run, the proved `Q` projects to a **watched-invariant**
  assertion at its point and the `P` to a **confirm-held** assertion at its
  boundary, each asserting **the export entry's own goal** (not a re-authored
  predicate); when the export `Q`/`P` change, the **assertion-point set changes
  correspondingly** (the watched-invariant for a now-`unknown` goal becomes a
  confirm-held `P` assertion; a removed `assume` drops its confirm-held point)
- why: AC4/TC4 — *projected, never re-authored*: the assertion set is the image
  of the B1 export's `Q`/`P` and **flips with it**. A re-authored or hard-coded
  assertion list shows **no change** when the export changes → red. The
  **meaning** of `proved`→`Q` vs `unknown`→`P` (the structural proved↔assumed
  discriminator) is **subsumed** by `../export/` EX-A and `21 §5.4` — this
  case does **not** re-derive it; it pins only the **runtime assertion-point
  projection** of an already-established `Q`/`P`.

## TR-E. The monitor projected from `T` — not authored (AC5/TC4)

### trace/monitor-changes-when-T-changes (AC5)
- spec: `spec/70-behavioral/73-conformance.md §2.4` (TC4), `71 §5.2`, `72 §3`
- given: a program with a **`delegated`** `Temporal` obligation `T` (the landed
  `T` **channel** — `TEntry`, `export.rs`); run the monitor projection, then
  **change `T`** (add / remove a delegated obligation) and run again
- expect: the monitor is the **projection** of the export's `T` — it **changes
  when `T` changes** (a different `T` channel yields a different monitor); its
  atoms range over `Σ` (the same events TR-A emits). The **semantic** acceptance
  of a concrete Büchi monitor over a concrete `Temporal` formula is
  **`(oracle)` / B2-deferred** (needs the `Temporal` datatype + the `compile`
  faithfulness lemma, `73 §2.4`)
- why: AC5/TC4 — *projected, not authored*: the monitor **is** the image of `T`,
  with no separately-authored model (the no-drift property). A hand-written
  monitor ignoring `T` is **unchanged** when `T` changes → red. The case nets
  the **projection-as-projection** over the **landed** `T` channel **now**; the
  full LTL→Büchi `compile` faithfulness is B2's (`oracle`-tagged), so the build
  is not blocked on B2 for the projection plumbing — only for the lemma
  (the buildable-now vs B2-gated partition, `73 §2.4`).

## TR-F. The one-way gate — emit-only, no promotion (AC6/TC5)

### trace/monitor-verdict-never-promoted-to-proved (AC6, soundness)
- spec: `spec/70-behavioral/73-conformance.md §2.6/§3` (TC5), `71 §5.1` (I4),
  `63 §5a`, `21 §5`
- given: a `delegated` `T` obligation with a simulated **monitor accept** (a
  green run / accepting Büchi) re-entering the Ken side **only** as monitoring
  evidence / a discharge-attestation record (`63 §5a`) — *not* as a kernel
  certificate; run the emitter
- expect: the obligation stays **`delegated`** — **never** re-stamped `proved`,
  **never** appears in `Q`; the trace **event** carries **no epistemic status**
  (an ITF witness, never tagged `proved`); and there is **no
  emitter/contract code path** from a monitor verdict to a `proved` status
- why: (soundness) AC6/TC5, the **G-Ward-seam** gate. One-way flow is
  realized as the **absence of a code path** (emit-only; no ingest), not a
  runtime check — a guard-gated absence, named: *no `proved`-writing edge from a
  monitor/classical verdict exists* (not merely "the happy path doesn't take
  one"). A green monitor fed back leaves it `delegated`; a build with
  a promotion path stamps it `proved` → it surfaces in `Q` → red. Disconfirming
  question: *would a delegated obligation with a green monitor land
  in `Q`/`proved` under the bug this targets?* — yes, the case flips. (Extends
  `../export/` EX-E1 to the **runtime monitor** verdict — the offline-discharge
  case there, the live-monitor case here — same gate, not re-derived.)

## Coverage map (AC / invariant → case)

| AC / invariant | Case |
|---|---|
| AC1 / TC1 events concretize `Σ` (no second alphabet) | TR-A (∧ TR-E for the monitor half, one export) |
| AC2 / TC2 effect-boundary containment | TR-B |
| AC3 / TC3 multi-space correlation | TR-C |
| AC4 / TC4 runtime `Q`/`P` assertion points | TR-D |
| AC5 / TC4 monitor projected from `T` | TR-E |
| AC6 / TC5 one-way / emit-only | TR-F |

## Cross-case sweep (group by projection source; assert agreement)

- **Projection-source class agrees.** Each part of the contract is the image of
  one export source: {events ↔ `Σ`} TR-A · {`Q`/`P` assertion points ↔ export
  `Q`/`P`} TR-D · {monitor ↔ `T`} TR-E. Each **flips when its source changes**
  (TR-D on `Q`/`P`, TR-E on `T`) — the *projected-not-authored* property holds
  uniformly; a re-authored part would not move with its source.
- **AC1 is the conjunction over one export.** A program emits a trace in `Σ`
  (TR-A) that a monitor synthesized from the **same** export (TR-E) checks — *no
  separately-authored model*. The structural binding (both parts derive from one
  export) is netted by TR-A ∧ TR-E; the **semantic** end-to-end acceptance is
  `(oracle)` / B2-deferred (TR-E).
- **Boundary invariant 1 — containment (no event off the perform site, TC2).**
  TR-B pins it directly (event count == `Vis`-firing count; pure steps silent).
  Bounded overhead is structural, netted solely here.
- **Boundary invariant 2 — one-way (no monitor verdict in `proved`, TC5).** TR-F
  pins it (no promotion edge from a monitor verdict; the event is a witness, not
  a claim). Across the corpus, **no** case routes a monitor verdict into a
  `proved` status.
- **The correlation pair is non-degenerate.** TR-C asserts correlated events
  **link** *while* uncorrelated events **don't** — the **sole** net for the
  correlation keys: a constant/global key passes "do they link?" vacuously and
  fails "do unrelated ones stay separate?".
- **Projection-fidelity is the net (B3 analog of N1).** The contract is an
  untrusted projection + instrumentation; the kernel never inspects the trace
  bytes (`73` banner, `71 §6/§5.1`). So the `(soundness)` cases {TR-B, TR-F} are
  the **sole** net for out-of-boundary emission / verdict-promotion — the
  erased-before-kernel omission-hole, here for the trace contract.

## Subsumed upstream — not re-derived

- **The export-field projection** (status → field: `proved`→`Q`, `unknown`/
  `tested`→`P`, `delegated`→`T`, `disproved`→none) and the **structural
  proved↔assumed discriminator** are pinned by `../export/seed-export.md`
  (EX-A, EX-E1) and `71 §2.1`/`21 §5.4`. TR-D and TR-F **do not re-derive**
  that meaning; they pin the **runtime projection** of an already-established
  `Q`/`P`/`T` (assertion points; the live-monitor one-way edge).
- **The effect model** — `Vis` perform points, escape check, the meaning of a
  `Σ` member — is pinned by `../../surface/effects/seed-effects.md` and `36`.
  TR-A/TR-B **do not re-derive** the effect semantics; they pin the **trace
  concretization** of the perform points (the event per firing; containment to
  the boundary).
- **The verdict meaning** (`proved` = a kernel-re-checked cert, `unknown` =
  a postulate hole, `delegated` = a `Temporal` value) is `../../verify/
  seed-verify.md` and `21 §5` — subsumed, not re-derived (subsume-don't-
  proliferate).

## Build-sequencing note

- B3 instruments the **landed** `36 §2` perform points (the `Vis` site in
  `drive_H`, `eval.rs`) and projects the **landed** B1 export (`export.rs`). Pin
  against the code, not the frame's prose (reconcile-against-landed-code).
- B1's `Σ` is landed at **effect-label** granularity; the per-op `Op`/`Resp`
  signature detail is concretized **here** in the event `op`/`response` fields
  (`73 §2.1`). A build assuming B1's `Σ` already carries `Op`/`Resp` fields
  mis-scopes the work.
- The `Temporal` **datatype** + the `compile` faithfulness lemma are **B2**
  (`72 §3`, `73 §2.4`); no case asserts a `Temporal` constructor set or a
  `compile` equation — only the `T` channel (the `delegated` values + status +
  the `Σ` they range over) and the monitor's projection from it.
- The literal field-key spellings are `(oracle)` — `Ward` finalizes the wire
  token; a rename after it binds is a breaking change (`71 §3.3`). Conformance
  pins the concept + value-set + invariants and tags the literal key
  (assert-at-locked-granularity).
