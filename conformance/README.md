# Conformance corpus

The executable, black-box behavioral tests that define "correct Ken." Each case
pins a specific spec section (`../spec/`) and states an **input → expected
behavior** that any conforming implementation must satisfy. Conformance is the
CI gate every team's work passes (`../docs/program/04-git-and-integration.md`);
it is also how the Spec enclave cross-checks the spec against permissive
references and first principles (`../CLEAN-ROOM.md`).

## Layout

```
conformance/
  kernel/        — 10-kernel/ behaviors (universes, pi-sigma, inductive,
                   identity, observational, conversion, judgments)
  verify/        — 20-verification/ (spec-syntax, obligations, prover,
                   diagnostics, protocol)
  surface/       — 30-surface/ (lexical, grammar, declarations, data-match,
                   numbers, effects, collections, ffi-io, elaboration)
  runtime/       — 40-runtime/ (values, evaluation, termination, capacity)
  stdlib/        — 50-stdlib/ (lawful instances, verified building blocks)
  security/      — 60-security/ (information flow, constant-time discipline,
                   capabilities & authority)
  behavioral/    — 70-behavioral/ (assumption-boundary export emitter,
                   trace/instrumentation contract, Temporal datatype +
                   delegated export flow, agentic boundary)
```

## Case format

Until the surface syntax and harness are fixed, cases are written as
**structured markdown** (or a small JSON/TOML once the harness lands,
OQ-harness). Each case:

```
## <case-id>          e.g. kernel/universes/type-in-type-rejected
- spec: <section>     e.g. spec/10-kernel/12-universes.md §1
- given: <input>      a core term / surface program / obligation
- expect: <behavior>  accepts | rejects(reason) | reduces-to(v) | proved
                      | disproved(countermodel) | incomplete(hole) | error(kind)
- why: <one line>     the property this pins (often a non-reproduction of a
                      prototype gap, or a soundness commitment)
```

Cases tagged **(oracle)** are to be confirmed against Ken's reference
interpreter once it is available; before then, ground them against the existing
`/spec`, permissive references, and first principles. Cases tagged
**(soundness)** encode a kernel soundness commitment
(`../spec/10-kernel/README.md §5`) and must never regress.

## Seeds

This directory is seeded with representative cases per area (the files below)
that establish the format and pin the **load-bearing non-reproductions** of the
prototype's gaps. The build teams grow the corpus as they implement; a spec
claim with no conformance case is a claim no one can rely on
(`../spec/00-overview.md`).

- `kernel/seed-kernel.md` — `Type:Type` rejection, dependent Σ, `J` on
  non-`refl`, SCT accept/reject (the four kernel commitments most worth pinning
  first).
- `kernel/seed-k1.md` — K1-scoped subset (33 seed cases covering AC-1 through
  AC-8).
- `kernel/observational/seed-observational.md` — K2-scoped seed cases (Omega-PI,
  funext, propext, Eq-by-type, cast regularity/computation, J-on-nonrefl,
  quotients, truncation, UIP).
- `verify/seed-verify.md` — a proved postcondition, a disproved one with a
  countermodel, an incomplete one with a hole, and the soundness regression (Z3
  cannot force a false `proved`).
- `surface/seed-surface.md` — elaboration invariants (well-typed output,
  ambiguity-is-an-error); the `data`/`match`/refinement cases are homed in
  `data-match/` (below).
- `surface/data-match/seed-data-match.md` — L2 sum types + `match` + refinements
  (`34`): real constructors + a computing `elim_D`, `match`→`elim_D` (nested),
  the **required exhaustiveness** safety (non-exhaustive rejects naming the
  unmatched pattern; kernel-backed totality, `34 §4.4`), reachability, indexed
  (GADT-like) families (impossible-application rejects *while* the impossible
  arm is omittable-by-absurdity), per-branch definitional refinement (`22 §3`
  hypothesis), and refinement types (carrier + emitted obligation, free
  forgetful coercion).
- `surface/numbers/seed-numbers.md` — L1 numeric model (`35`): arbitrary-
  precision `Int` exactness above 2⁵³, literal defaulting (`2:Int`/`2.0:Float`/
  `2.0d:Decimal`), the fixed-width no-overflow obligation + no-silent-wrap seal,
  `Decimal`-exact-vs-`Float`-honest, explicit conversions, the kernel-primitive
  vs prelude-law boundary, `Char` surrogate exclusion.
- `surface/collections/seed-collections.md` — L3 strings & collections (`37`):
  `String` as a content-addressed **NFC UTF-8 primitive** (byte-length ≠
  char-length, **not** `List Char`); `List`/`Option`/`Result` transparent
  inductive (L2) and `Array`/`Map`/`Set` abstract over the `41` heap (kinds
  `0x06`–`0x08`) with **persistence observable as slot-id**; the combinator laws
  as **emitted propositions**; **infinitude without coinduction** (the
  fuel-bounded inductive `unfoldUpTo` + the no-coinductive-**construct** absence
  net); the `DecEq`-key **verdict flip**; and the verified `sort` whose
  `isSorted ∧ Perm` obligation is asserted **with the `Perm` conjunct present**.
- `surface/bytes-io/seed-bytes-io.md` — L6 `Bytes` + binary I/O (`38 §1`):
  `Bytes` as a `14 §5` primitive (`0x05`, immutable, `b"…"`/`0x[…]` literals),
  registered reductions + no-silent-OOB partiality, effect-tracked I/O
  (`read_bytes`/`send` carry their `[FS]`/`[Net]` rows — the L5 `36 §1.4` gate,
  referenced not duplicated), explicit `encode`/`decode` (no hidden charset),
  and the one-directional round-trip law (`decode (encode s) == Ok s` provable;
  the reverse is **not** a law).
- `surface/ffi-io/seed-ffi-io.md` — L7 `foreign` FFI + the trust boundary
  (`38 §2–§4`): a `foreign` is a typed, effect-rowed, capability-gated **listed
  postulate** (`declare_postulate` → `trusted_base()`); foreign-as-listed-
  postulate routed through the **real** B1 export (the dependency pair —
  relied-on-by-**call** listed in `P` / not-relied-on absent), `pure`-as-claim
  projects to *trusted* **never** `Q` (the over-claim pair, trusted-by-typing-
  is-not-`Q`), boundary `requires`/`ensures` lower to runtime-checked `tested`
  assertions, effects mandatory via the real `36 §1.4` escape check (the flip)
  with the **`pure`-but-effectful** named residual, capability+effect
  composition (couples Sec2), and a G6 round-trip proof in an FFI-using verified
  component.
- `runtime/seed-runtime.md` — dedup + O(1) equality; `Int` past 2⁵³ exact;
  `unknown` propagation.
- `runtime/capacity/seed-capacity.md` — X2 store hardening (`44`): dedup-aware
  accounting (distinct, not occurrences), the **loud** at-limit failure
  (`CapacityExhausted` raised, never the silent `NULL_SLOT` drop), reclamation
  page release (`arena_bytes → 0`), region-scoped lifetime + escape-survival,
  retired-ids-never-resurrected, and no-lattice-on-the-hot-path.
- `security/ifc/seed-ifc.md` — Sec1 information-flow-by-typing (the
  implicit-flow discriminator, label joins, capability-gated declassify, the
  relational by-proof mode).
- `security/ct/seed-ct.md` — Sec1ct `@ct` constant-time discipline (the taint-
  axis orientation pair, the sealed `LeakSink` set, declassify ends the span,
  the CT-in-parameter `Q` promise).
- `security/capabilities/seed-capabilities.md` — Sec2 authority discipline
  (`62`): no ambient authority (a no-cap/no-row `view` is inert), least by
  default, **monotone-downward attenuation** (the order-dual non-degenerate pair
  — weaker-accepts/stronger-rejects — over a **kernel-backed** refinement bound,
  unlike Sec1's erased labels; plus the enumerated absence of any amplifying
  operation), transitive revocation (static contract; runtime membrane
  `(oracle)`-deferred), statically-known audit points + declassify-in-delta, and
  authority+flow composition (a `Net` write needs the capability **and** the
  clearance — dropping either rejects).
- `behavioral/export/seed-export.md` — B1 assumption-boundary export emitter
  (the `Q`/`P`/`Σ`/`T`/`G` status→field projection: the no-over-claim pair,
  alphabet reuse, the no-measure seal, the one-way gate, content-hash
  reproducibility).
- `behavioral/temporal/seed-temporal.md` — B2 `Temporal Σ` datatype + export
  flow (`72`): temporal/behavioral logic as deeply-embedded LTL/μ **data** over
  the B1 `Σ` (admitted by K1; first-order fixpoint binding load-bearing);
  derived `◇`/`□`/`leadsto` are `until`/`not` syntax, not constructors; the
  surface→`delegated`→`T` export flow is **total, constant, one-way** (never
  `Q`/`P`); a buildable-now reason-*about* closedness metatheorem; and the
  **structural absence** of any kernel modality (reason *about*, never *with*).
- `behavioral/trace/seed-trace.md` — B3 trace/instrumentation contract
  (`73`): the runtime companion to the B1 export — the `Σ`-event schema at the
  effect boundary, correlation keys for multi-`space` traces, the runtime
  `Q`/`P` assertion points, and the monitor projected from `T`. An **untrusted
  one-way projection** of already-verified content + instrumentation: adds
  nothing to the trusted base, proves nothing new.
- `behavioral/agentic/seed-agentic.md` — B4 the agentic boundary (WS-B capstone,
  `74`): assuring an embedded agent's outputs **reduces to the existing seam**
  aimed at a maximally-nondeterministic component (agent = maximal `P`) — **no
  new mechanism**. The four assurances **partition** the four-way status: safety
  envelope→`proved`/`Q` (the propose/act capability split), metamorphic→
  `tested`/`P`, RV watchdog→`delegated`/`T`, output quality→`unknown` (never
  `proved` — safety, **never** quality). A **projection-fidelity** corpus:
  AC1/AC2/AC5 drive landed producers (the export projection, the real
  `Cap E`/no-ambient flip, the `trusted_base()` flip); AC3/AC4 pin the landed
  projection and carry their deferred discharge engines (the `[rel-deferred]`
  relational reducer, the `(oracle)`/B2 live monitor) as named triggers.
