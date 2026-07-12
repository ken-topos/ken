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
  surface/       — 30-surface/ (lexical, grammar, declarations, classes,
                   modules, data-match, numbers, effects, collections, ffi-io,
                   elaboration)
  runtime/       — 40-runtime/ (values, evaluation, termination, capacity,
                   native backend differential-equivalence)
  stdlib/        — 50-stdlib/ (lawful instances, verified building blocks)
  security/      — 60-security/ (information flow, constant-time discipline,
                   capabilities & authority, trust model & TCB, policy-as-code)
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
- `kernel/inductive/seed-k4-omega-motive-elim.md` — K4 **elimination into `Ω`**
  (`14 §3`): the general eliminator's motive codomain is a **sort** (`Type ℓ' ∪
  Ω_l`, not a wildcard), so an `Ω`-codomain motive **proves a per-branch-varying
  proposition** by case-split on a relevant scrutinee — the **capability flip**
  (`refl : (x:Bool) → IsTrue (bool_leq x x)` via `elim_Bool`; accepts post-K4,
  rejected pre-K4) that un-gates the lawful-classes accept arm; the **non-sort
  codomain reject** (sort, not wildcard); **sort-agnostic ι** (no new reduction
  path); and the ★ **conv-embedding commutation** (into-`Ω` narrows — two elims
  with the same motive but different proof methods are **conv-equal**, proof
  irrelevance preserved through conversion with no new conv rule; a
  proof-relevant-`Ω`-elim leak-out is the danger it rules out).
- `kernel/observational/seed-k5-omega-fragment.md` — K5 **the observational
  fragment** (`16 §1.4`): the two prelude rules that **close** the two `Eq`
  reducts — `(Top-Intro)` `tt : Top` (same-nullary-ctor ⇝ `Top`) and
  `(Bottom-Elim)` `absurd C p : C` **with `C : Omega_l` only** (distinct-ctor ⇝
  `Bottom`, ex-falso). `Top`/`Bottom` are **bare `Omega_0` sub-singleton prelude
  constants**, not the K1 `Unit`/`Empty` coerced in (`§1.3` correction). Seven
  discriminating cases: the two **capability flips** (`tt` closes a `Top`-goal,
  `tt ∉ trusted_base()`; `absurd` discharges from a `Bottom`-hyp) with
  pre/post-K5 verdict flips; the **motive-is-a-sort** boundary (`Omega`-only,
  not a wildcard — `Type`-motive rejected); the **proof-is-`Bottom`** premise (a
  non-`Bottom` proof rejected — the consistency-critical net); the ★★ **AC6
  SCT-launder** reject (a recursion hidden in an `absurd` subterm must be
  rejected — flips to admit against a `collect_calls` that skips `absurd`; the
  K2c hole one position over); the **no-new-rule** posture (`absurd` neutral,
  `eq_reduce` unchanged, Ω-PI conversion); and the **antisym two-branch**
  integration (`tt`/`absurd` close a decidable-order law — the `51 §6`
  complete-instance un-gate).
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
  (GADT-like) families (explicit dependent-constructor signatures,
  impossible-application rejects *while* the impossible arm is
  omittable-by-absurdity, omitted possible arms reject), per-branch definitional
  refinement (`22 §3` hypothesis), proof-returning dependent `match` motives
  into `Ω` with wrong-specialized-branch rejection, and refinement types
  (carrier + emitted obligation, free forgetful coercion).
- `surface/numbers/seed-numbers.md` — L1 numeric model (`35`): arbitrary-
  precision `Int` exactness above 2⁵³, literal defaulting (`2:Int`/`2.0:Float`/
  `2.0d:Decimal`), the fixed-width no-overflow obligation + no-silent-wrap seal,
  `Decimal`-exact-vs-`Float`-honest, explicit conversions, the kernel-primitive
  vs prelude-law boundary, `Char` surrogate exclusion.
- `surface/numbers/seed-f1-bignum-int.md` — WP F1 genuine arbitrary-precision
  `Int` (`18a §5.2.1`, first Phase-2 BUILTINS tranche WP): no-wrap totality
  across the **i128 ceiling** (the distinct non-reproduction from
  `seed-numbers.md` AC1's f64 carrier — closes the OF1 2⁶⁶ blind-spot), the
  `18a §3` **independent** differential oracle (never the production crate on
  both sides), the eval↔`Value::BigInt` store round-trip F1 **establishes**
  (byte-identity + `minimal_limbs`; no `to_rt` `BigInt` arm today), and the
  workspace-green + §63/ADR-0009 crate-vetting hard-ACs.
- `surface/numbers/seed-decimal-char-demote.md` — WP Decimal/Char DEMOTE
  (`18a §5.6.1`/`§5.9.1`/`§5.2.2`, second Phase-2 BUILTINS tranche): a **TCB
  removal by removal-not-shadowing** — native `*_decimal` ops + the `Char`
  primitive **type** deleted, replaced by derived `(coeff:Int, exp:Int)` +
  refinement `{c:Int | isScalar c}` over F1 bignum. Pulls up the **`leq_int`
  ordering arm** (ruling (A), the derived-def prerequisite) with its
  **independent** differential oracle (never `num-bigint`'s `Ord` both-sides);
  closes **both F4 halves** (saturating-`mul` *and* the sharp false-`True` `eq`
  — a wrong value in the tested-not-trusted ring, not a false proof); the
  derived `Decimal`/`Char` **ops** (the `Num`/`DecEq`/`Ord` **law**-carrying
  instances re-home to the lawful-classes lane — zero-NEW-delta over `Int`, not
  zero-delta); and the two **Char soundness pins** — the
  `isScalar := IsTrue(inRangeBool)` Ω-encoding (**sort-not-token**) and
  extraction **computing** the scalar proof (runtime face deferred).
- `surface/collections/seed-collections.md` — L3 strings & collections (`37`):
  `String` as a content-addressed **NFC UTF-8 primitive** (byte-length ≠
  char-length, **not** `List Char`); `List`/`Option`/`Result` transparent
  inductive (L2) and `Array` abstract over the `41` heap (kind `0x06`) with
  **persistence observable as slot-id** (the heap `Map`/`Set` `0x07`/`0x08`
  model is **superseded** by the proved BST, `stdlib/map/seed-map.md`); the
  combinator laws as **emitted propositions**; **infinitude without
  coinduction** (the fuel-bounded inductive `unfoldUpTo` + the
  no-coinductive-**construct** absence net); the `DecEq`-key **verdict flip**;
  and the verified `sort` whose `isSorted ∧ Perm` obligation is asserted **with
  the `Perm` conjunct present**.
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
- `surface/classes/seed-classes.md` — Lc typeclasses/constraints (classes-as-
  subobjects, `33 §5` + `39 §6`, ADR 0008): a `class` → a Σ+η **record**, an
  `instance` → a **record value** with real law proofs, and `where C A` → an
  implicit instance argument discharged by **search**. Eight discriminating ACs:
  coherence (same `(class, head-type)` → same canonical), the orphan / overlap /
  ambiguity pairs (never a silent pick), **the sort *is* the discriminant**
  (property Ω vs structure `Type` — the both-keyed `sort_sigma`; forcing
  structure→Ω is the Σ-sort trap), named-instance explicit escape, **SCT-bounded
  termination** on the reified dictionary group (the real
  `declare_recursive_group` + `sct_check` + reification faithfulness), `derive`
  as an untrusted kernel-re-checked candidate, and a lawful instance's law proof
  cited by the prover. A **kernel-backed vs elaborator-convention** split:
  AC4/6/7/8 bottom out in landed producers; the coherence convention (AC1/2/3/5)
  is netted solely by conformance.
- `surface/classes/seed-class-field-purity.md` — SURF-2 class-field purity:
  optional `[const|fn|proc]` markers on class fields, erased-before-kernel
  metadata that is parsed/stored on the class, enforced on instance fields
  through the existing SURF-1 purity checker, surfaced by `.field` projection,
  backward-compatible for unmarked fields, and explicitly excluded from the
  AC4 Type/Omega sort discriminant. Pins the `proc traverse` unblock for CAT-2
  D3 while keeping zero kernel/Cargo.lock/trusted-base delta.
- `surface/declarations/seed-named-proof-claims.md` — `prop` families,
  `lemma` theorems, attached `proof` canonicalization, explicit bare/grouped/
  canonical selector identity, and the bare-name-rejects/selector-resolves pair.
- `surface/declarations/seed-def-refinement.md` — SURF-def-refinement: the
  `type` → `def` declaration-keyword rename (refinement + alias RHS forms),
  the discriminating negative (`type` no longer parses as a declaration),
  `type` reserved as a free identifier too, and the should-have value-position
  steering diagnostic.
- `surface/declarations/seed-declaration-collisions.md` — N1 fail-closed
  top-level declaration collisions (ADR 0014 MRES-5/7/8): an ordinary duplicate
  and a class/constructor same-name collision hard-reject through one specific
  diagnostic, paired with the live arity-gated `Eq`/`J` sugar coexistence
  control. The reject arms are explicitly red until N1 Lane B.
- `surface/formatting/seed-canonical-format.md` — WP S canonical formatter
  acceptance oracle (`31 §1`): the eight semantic gates (byte idempotence,
  parse and elaboration preservation, whole-catalog posture, literate prose
  identity, trivia/literal preservation, deterministic 88-column width, and
  token-role ambiguity), with controlled pairs for arrows, `:`/`::`, dot
  roles, `l`/level, `in`/membership, every protected literal form, and all four
  literate fence roles. Formatter-output cases are red until B3–C; the narrow
  unparseable `ignore`/`reject` exemption remains token-aware.
- `surface/modules/seed-modules.md` — ES3 minimal modules/imports (`33 §3-4`,
  the bounded L4 slice): `module`/`import`/`pub`/abstract-export **elaborate
  away** to the kernel's single flat append-only `Σ` (`11 §4`) — **zero new
  kernel feature, zero `trusted_base()` delta** (the ES1 minimality invariant
  applied to modules; a module program and its fully-qualified equivalent
  produce **identical** `Σ`/`trusted_base()`). Abstract export **IS** the
  existing opaque constant (byte-identical kernel rep; a client `match` on a
  hidden constructor is a **surface** name error). Visibility is
  **private-by-default + `pub`** (settled), resolution is **surface-only** — the
  private-name-access and N3 module-level local/import or prelude clashes are
  surface diagnostics that **never reach the kernel**. N3 pins explicit
  resolution by positive de-selection or per-name rename, keeps narrower
  lexical binder shadowing innermost-wins, and rejects `hiding` at parse time.
  N2 adds the in-repo cross-file loader golden: an import resolves another leaf
  file through the plural root API with
  exactly one root (red until N2 Lane B), while adding only a back-edge flips
  the same fixture to the specific `ImportCycle` diagnostic naming
  `A → B → A`. N4 adds source-world anonymous `program`/`package` boundaries,
  explicit `admits`, the direct-use-versus-transitive-coherence discriminator,
  self-admission, and defining-package provenance. Compiled manifests,
  re-export-carried instance surfaces, and multi-root precedence remain out.
- `runtime/seed-runtime.md` — dedup + O(1) equality; `Int` past 2⁵³ exact;
  `unknown` propagation.
- `runtime/capacity/seed-capacity.md` — X2 store hardening (`44`): dedup-aware
  accounting (distinct, not occurrences), the **loud** at-limit failure
  (`CapacityExhausted` raised, never the silent `NULL_SLOT` drop), reclamation
  page release (`arena_bytes → 0`), region-scoped lifetime + escape-survival,
  retired-ids-never-resurrected, and no-lattice-on-the-hot-path.
- `runtime/backend/seed-backend.md` — X3a native backend **differential
  equivalence** (`45`): the interpreter-is-oracle rule (same closed ground term
  through both → **identical K3 values**; interpreter right by definition,
  `42 §5`), the **observable/unobservable** discriminating pair (a divergence in
  observable value **rejects** / an unobservable-internal strategy difference
  **admits**, `42 §2` layers-may-differ), the **not-in-TCB** posture (a codegen
  bug is a wrong value, **never a false `proved`** — the trust chain kernel `Q`
  / interpreter-oracle `tested` / backend `tested`, **not** kernel-backed),
  determinism-carries, and the `44` capacity cross-ref. A design/discipline
  corpus: the interpreter half is landed (`crates/ken-interp`), the backend half
  is `(oracle)`/X3-build-deferred; target-agnostic (`OQ-backend-target` stays
  operator-open).
- `stdlib/classes/seed-lawful-classes.md` — ES4 lawful structure classes
  (`Eq`/`DecEq`/`Ord`, `50-stdlib/51`): the **laws-PROVED** discipline — an
  instance's law fields are **real kernel proofs**, so "lawful ≡
  zero-`trusted_base()`-delta" (AC3, the law-side of ES1's zero-delta): a
  **law-less** dictionary (postulated/holed/stubbed law fields) is **rejected as
  unlawful** (non-empty delta / re-check fail) while the real instance accepts —
  the discriminating flip, grep the law fields for `declare_postulate`/holes;
  the `Ord` **totality** law is `Ω`-clean as the Bool-equation
  `IsTrue (leq x y || leq y x)` (no truncation); and `where Ord a` supplies the
  **same** `sort` obligation as the explicit comparator (AC2,
  reflect-don't-extend). **Carrier axis (`§6`):** "lawful ≡ zero-delta" holds on
  an **inductive** carrier; a **primitive** carrier is audited-delta (its
  ∀-laws unprovable — no eliminator — so a *declared* delta, not zero) —
  `primitive-carrier-declared-audited-delta`. The zero-delta real-proofs path is
  realizable now for the **live-`Eq`-conclusion** laws (`refl`/`trans`/`total`,
  `Eq`'s `refl`) via **K4** (`3be0e30`); the **concrete-equality-conclusion**
  laws (`antisym`/`sound`/`complete` → `Top`/`Bottom`) needed the **K5 + K7**
  kernel capability (K5's `tt`/`absurd` terms `1c84a30`; K7's `eq_at_inductive`
  operand-`whnf` fix `4ae2baf`, `16 §8.1`) — both now landed, so they are **real
  zero-delta proofs on main** (ES4-lawproofs-remainder `9a82745`: `Ord Bool`'s
  `antisym`, `DecEq Bool`'s `sound`/`complete`, no `Axiom` remaining); `Eq`'s
  `sym`/`trans` are likewise **real zero-delta** via **case-split** (not K6 —
  each branch computes concretely, so the `conv_struct` `Eq`-congruence is never
  exercised; K6 stays a sound-but-customerless forward gap). The first real
  instances (`Ord Bool` `refl`/`trans`/`total`, `Eq Bool` `refl`) are on main
  (**ES4-lawproofs**, `72e38a5`, Team Language); `Ord Bool`, `Eq Bool`, and
  `DecEq Bool` are now **complete** zero-delta lawful instances (`9a82745` + the
  `Eq Bool` `sym`/`trans` WP). The `absurd`
  subterm is traversed by the
  `trusted_base_delta` cone walk
  (`absurd-subterm-postulate-counted-in-delta`, the elaborator-accounting
  sibling of the K5 seed's SCT launder net). Static face; declared-vs-hidden
  honesty enforceable throughout.
- `stdlib/map/seed-map.md` — Map-container proved `Map k v` over `Ord k`
  (`50-stdlib/52`, VAL2 #8 / OQ-A): a **proved, pure, `Ord k`-keyed**
  associative map shipped as **package Ken out of `trusted_base()`** — a bare
  unbalanced BST (`data Tree k v = Leaf | Node …`) whose operations are `view`
  defs and whose every correctness law is a **real kernel proof, not a
  postulate**; it **retires** the opaque `Map`/`Set` primitive (net-negative
  TCB). AC1 (inductive-not-primitive + `trusted_base()` shrinks), AC2
  (real-interpreter `insert`/`lookup` round-trips, ordered `toList`, the
  `letter-frequency` shape — value-flips, `Char`/`Bool` keys), AC3
  (**proved-not-stubbed** via the real `trusted_base_delta` cone walk — a
  stubbed `Axiom` map-proof grows the delta and is rejected **for the right
  reason**, exercised not textual; the load- bearing net, with the X1
  not-a-value-flip flag), the `§2.1` canonical-carrier `antisym → Equal`
  localization pair (`Int`/`Char`/`Bool` sound vs `Decimal` `Bottom`), and the
  named `§7` deferrals (`delete`, balance, the proof-relevant permutation law).
  **Red-until-built** — the build's acceptance target; companion supersession
  reconcile in `surface/collections/seed-collections.md`.
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
- `security/trust-model/seed-trust-model.md` — Sec4 trust model & TCB (`64`):
  the **enumerable TCB** (empty ⟺ verified — the no-phantom `[landed producer]`
  `trusted_base()` pair, plus the completeness omission-net `[by construction]`:
  a real `foreign`/hole surfaces via the `declare_postulate`→`Opaque`
  choke-point, a checked def never does), **authorship-independence** (the
  de-Bruijn security reading `[structural]` — a false-proposition cert is
  rejected regardless of any author framing; `check` exposes no provenance
  channel), the **trusting-trust invariant** (`ken-kernel`'s dependency closure
  carries no Ken-generated artifact — `[structural / architectural]`), and the
  **honest limits** normative + first-class (spec ≠ intent the headline
  residual risk). Trust levels tagged per case — no "kernel-backed" over-claim
  on the by-construction / structural faces.
- `security/policy/seed-policy.md` — Sec5 policy-as-code (`65`, ADR 0007):
  *authored by compliance, enforced by the compiler*. The **binding guarantee**
  (a policy-`Secret` value to a `Public`-cleared sink **fails to compile** via
  the real `l_sink`/`flows_to` — `SECRET⊑PUBLIC` false; + the `pc`-join implicit
  flow), **non-weakenability** as a property of the **landed** enforcement
  (`flows_to` doesn't consult authorship; the policy-`binding` mechanism is
  `[deferred: OQ-policy]`, oracle-tagged), declassification gated by the
  policy-granted capability (real `check_declassify` + a downgrade-direction
  guard), the **`@ct`** static face (the order-dual pair; runtime Ward face
  `[deferred]`), and governance riding the `63` supply-chain
  (`trusted_base_delta` payload). **Soundness anchor (AC4):** policy is
  *data + binding* — **no new kernel, no new metatheory** (empty delta),
  `[structural/by-construction]` and
  **trusted-by-typing → `P`/`tested`, never `Q`/"kernel-backed"** (the corpus is
  the net, not the kernel).
- `security/supply-chain/seed-discharge-attestation.md` — Sec6 discharge-
  attestation contract (`63 §5a`, `OQ-discharge-attestation` DECIDED, ward
  `f33276b`): the Ken-visible field set (all B1-emitted), the **one-way gate**
  over the outcome vocabulary (**no outcome — not even `discharged` — promotes a
  `T` to `proved`**; extends B1's
  `delegated-obligation-never-promoted-to-proved` over the four outcomes + the
  `Q@ct` channel — the trusted-by-typing → `P`/`tested` never `Q` shape), the
  **Ward-internal boundary** (a correctness
  judgment reading `policy`/`bound`/`evidence`/`ct.method`/`regression` is
  rejected), reject-missing-`ward.version` / accept-ignore-unknown, and
  `id`-stable-across-`export.hash`-change. Static face; the three-check
  deployment gate is the named Team-Verify build follow-on.
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
