# FFI / trust-boundary conformance — seed cases (L7)

Format: `../../README.md`. These pin the **`foreign` FFI and the trust
boundary** (`spec/30-surface/38-ffi-io.md §2–§4`, impl-ready L7): a `foreign` is
a **typed, effect-rowed, capability-gated, *listed* postulate** — the place
Ken's guarantees stop, marked **honestly and structurally**. The cases net the
**trust-boundary discipline** (foreign-as-listed-postulate, `pure`-as-claim-not-
`Q`, runtime contracts at the edge, mandatory effects + the named residual), not
the meaning of the verdicts/statuses they observe (that is `../../verify/` and
`../../behavioral/export/`, subsumed below).

## Reading disciplines

**A `foreign` is a real postulate through the real kernel API.** A `foreign f :
T visits ρ` elaborates to `declare_postulate(Σ, [], rowed(T, ρ))` — an opaque
constant (`11 §4`) recorded in `trusted_base()` (`18 §4.2`/`§5`). Cases route a
**real** `foreign` decl through the **actual** elaborator + `trusted_base()` +
B1 export (`71`); a test that asserts a **synthetic** trust-base literal guards
nothing (the hand-feeds-the-deliverable trap). **No new kernel rule** is
introduced (`38 §2.3`) — the net is over existing postulate/effect/export
machinery.

**"Relies on" is by *use*, not *declaration* (the AC2 silence, resolved at
source `38 §3.1`).** What lists a foreign in an artifact's `trusted_base_delta`
is a verified definition that **calls** it (dependency-reachable), **not** the
`foreign` decl being in scope. AC2 nets **reliance-by-call** — a consumer that
calls the foreign lists it; one that does not is **absent** — never the vacuous
decl-lists-itself. The honesty property ranges over the **dependency cone** (`18
§5`, `71 §2.1`).

**`pure` projects to *trusted*, never `Q` — and the net is a pair.** A
`foreign`'s assumed guarantee (its type, its `pure` claim) is a `trusted_base()`
postulate, so by the structural discriminator (`21 §5.4`, `71` I1: `Q` **iff**
certificate `check`s **and** goal **∉** `trusted_base()`) it can **never** reach
`Q` — **under-claim is the safe direction**. *Nothing* about a foreign is ever
in `Q`, so "foreign absent from `Q`" alone is **green-vs-green**; the net is the
**pair on one artifact** — a kernel-proved Ken-side claim → `Q` **while** the
foreign assumption → `P` — so the field tracks **kernel-provedness**, not the
`pure` keyword. This is the **trusted-by-typing-is-not-`Q`** discipline (cf.
Sec1's declassify edge, B1's I1).

**Effects are mandatory through the *real* `36 §1.4` escape check.** The AC5
flip routes a real world-touching `foreign` through the **same** escape check L6
I/O rides (`38 §1.3`), not a synthetic flag: a caller that drops the foreign's
row is an EFFECT-ESCAPE reject; declaring it accepts — the verdict flips. The
**`pure`-but-effectful** case is the **named residual** the type discipline
**cannot** catch (`38 §3.4`): conformance asserts it is **flagged + listed**
(visible in `P`), **not** a netted verdict — naming an honest limit, not pinning
a catch that does not exist.

**Static vs runtime faces (the boundary-contract split, `38 §3.3`).** A
`requires`/`ensures` on a `foreign` that is **statically unprovable** lowers to
a `tested` runtime-checked assertion (`21 §5.2`); conformance observes the
**emitted runtime check** (structural), not "compiles". A `foreign` `ensures`
that **is** statically discharged over the marshalling needs no runtime check
and may reach `Q` — the two faces flip on static provability.

**Tags.** `(soundness)` = a real no-over-claim / honest-boundary commitment that
must never regress — the over-claim direction (a `pure` claim or a foreign
assumption reaching `Q`) is the soundness hazard, netted at the export
projection (the kernel is blind to which symbols a postulate's C body actually
touches — `38 §3.4`). `(oracle)` = the literal `foreign`/`symbol`/`library`/
`pure`/`visits` keyword spellings, finalized by the build team (`OQ-syntax`, `38
§2.1`); cases refer to the **concept** and `(oracle)`-tag the token.

## A. AC1 — the `foreign` declaration binds + marshals

### ffi-io/foreign-decl-binds-typed-effect-rowed (AC1)
- spec: `spec/30-surface/38-ffi-io.md §2.1`, `§2.2`, `§2.3`, `11 §4`, `41 §1`
- given: `foreign os_write : Int32 → Bytes → Int visits [FS] = symbol "write"
  library "c"`, elaborated
- expect: a postulate `os_write : (Int32 → Bytes → Int) with row [FS]` is
  admitted via `declare_postulate` (opaque constant, `11 §4`) and appears in
  `trusted_base()`; a call marshals the `Bytes` argument as **`(ptr, len)`** and
  the scalars as their machine types (`41 §1`) — a **structural** assertion on
  the elaborated binding + the marshalling record, **not** "compiles"
- why: AC1 — the binding is typed + effect-rowed + a real postulate, and the
  marshalling reuses the L6 `Bytes`↔`(ptr,len)` boundary (`38 §1.1`). Asserts
  the elaborated value/row + the marshalling, never just acceptance (the
  structural-output discipline). The literal keywords are `(oracle)`.

## B. AC2 — foreign-as-listed-postulate (the dependency pair)

### ffi-io/relied-on-foreign-listed-in-P (AC2)
- spec: `spec/30-surface/38-ffi-io.md §3.1`, `25 §3`, `18 §5`,
  `spec/70-behavioral/71-assumption-boundary.md §2.1` (I2)
- given: a verified `view useFFI ... = ... os_write ...` that **calls**
  `os_write`, run through the **real** B1 export
- expect: `os_write`'s postulate is in `useFFI`'s `trusted_base_delta` (`25 §3`)
  and projects into the export's **`P`** (assumptions, concept; literal key
  `(oracle)`) tagged `tested` under the **FFI boundary-label** case (`71 §2.1`)
  — **visible, not hidden**
- why: AC2 — the honesty headline. A verified artifact's contract **lists**
  exactly the foreign functions it relies on. Half of the dependency pair: alone
  it is green-vs-green (an emitter that lists *every* in-scope `foreign` also
  passes); the net is the pair with the not-relied-on case below.

### ffi-io/not-relied-on-foreign-absent-from-P (AC2, soundness)
- spec: `spec/30-surface/38-ffi-io.md §3.1`, `25 §3`,
  `spec/70-behavioral/71-assumption-boundary.md §2.1` (I2), `§3.3`
- given: the **same** module with `os_write` in scope, but a verified `view
  noFFI (x : Int) : Int = x + 1` that does **not** call it, run through the
  emitter
- expect: `os_write` is **absent** from `noFFI`'s `trusted_base_delta` and from
  its export `P` (and a relied-on vs not export differ in **hash**, `71 §3.3`) —
  the foreign decl being in scope is **not** a reliance
- why: (soundness) AC2 — the field **flips on dependency**, not on the decl's
  presence. With the relied-on case this is the **non-degenerate pair** (`38
  §3.1`, the silence resolved at source: reliance = call, not declaration). A
  buggy emitter that lists every in-scope `foreign` (or hard-codes `P`) lands
  `os_write` in `noFFI`'s `P` → red. Routed through the **real** export — the
  FFI instance of `export/removing-assume-shrinks-P` (`71` I2; subsume-don't-
  proliferate).

## C. AC3 — `pure` projects to *trusted*, never `Q` (the over-claim pair)

### ffi-io/pure-foreign-assumption-rides-P-not-Q (AC3, soundness)
- spec: `spec/30-surface/38-ffi-io.md §3.2`, `21 §5.4`,
  `spec/70-behavioral/71-assumption-boundary.md §2.1` (I1)
- given: an artifact calling `foreign c_sqrt : Float → Float = symbol "sqrt"
  library "m" pure`, **and** a co-located `view wrap (x : Float) : Float ensures
  result == c_sqrt x = c_sqrt x` whose Ken-side `ensures` is provable by `refl`
  (it relates the wrapper to the foreign's *result*, **not** sqrt's C
  semantics), so its obligation **discharges** (verdict `proved`); run through
  the emitter
- expect: `c_sqrt`'s assumed `pure`/type guarantee lands in **`P`** (`tested`,
  the trusted boundary) and is **absent from `Q`**; **while** `wrap`'s
  discharged postcondition lands in **`Q`** (`proved`) — the field tracks
  **kernel-provedness**, the `pure` keyword does **not** lift the assumption
  into `Q`
- why: (soundness) AC3 / `71` I1 — the no-over-claim direction, **trusted-by-
  typing-is-not-`Q`**. *Nothing* foreign reaches `Q`, so the
  foreign-absent-from- `Q` half is **green-vs-green** alone; the net is this
  **pair on one artifact** (kernel-proved → `Q` **while** foreign-assumed →
  `P`). The bug — trust the `pure` annotation, bucket it as a guarantee — lands
  the assumption in `Q` → red. Under-claim is the safe direction (`38 §3.2`).
  Reuses B1 EX-A's proved↔assumed pair, here `kernel-proved`↔`foreign-assumed`.

### ffi-io/wrong-pure-confined-and-listed (AC3)
- spec: `spec/30-surface/38-ffi-io.md §3.2`, `18 §5`
- given: a `pure foreign` whose external C symbol is **not** actually pure (an
  incorrect purity claim)
- expect: the unsoundness is **confined to that postulate** — it cannot
  masquerade as `proved` (the foreign never reaches `Q`, C1) — **and** the
  postulate is **listed** in `trusted_base_delta` → `P`, so the assumption is
  **visible** even though wrong
- why: AC3 — a wrong `pure` is a soundness bug *bounded + listed*, not a silent
  hole. Pins that the over-claim hazard is structurally foreclosed (never `Q`)
  and the residual is visible (in `P`). (Couples the AC5 residual E2.)

## D. AC4 — boundary contracts become runtime-checked

### ffi-io/unprovable-foreign-ensures-emits-runtime-check (AC4)
- spec: `spec/30-surface/38-ffi-io.md §3.3`, `21 §5.2`
- given: `foreign c_sqrt : Float → Float ... pure` with a **statically
  unprovable** `ensures result ≥ 0.0` (no body for the kernel to reason about)
- expect: the contract lowers to the **`tested`** status (`21 §5.2`): a
  **runtime-checked, fail-fast assertion** emitted at the call boundary **and**
  a `P`/`tested` entry in the assumption boundary — observe the **emitted
  runtime check** (structural), not "compiles". A statically-**provable**
  Ken-side `ensures` over the marshalling emits **no** runtime check and may
  reach `Q`
- why: AC4 — runtime contracts earn their keep exactly at the unverifiable
  boundary (`21 §5.2`: "boundaries, FFI, untrusted input"). The two faces **flip
  on static provability**: unprovable ⇒ a runtime check exists; provable ⇒ none.
  Asserts the emitted check, never a silent assumption.

## E. AC5 — effects mandatory (the escape-check flip + the named residual)

### ffi-io/world-foreign-without-row-rejected (AC5)
- spec: `spec/30-surface/38-ffi-io.md §3.4`, `36 §1.4`, `38 §1.3`
- given: `foreign os_write : Int32 → Bytes → Int visits [FS] = symbol "write"`
  called from a `view send_it ...` whose declared row is **∅** (no `visits`) —
  and a second `view send_ok ... visits [FS]` declaring `[FS]`
- expect: `send_it` is an **EFFECT-ESCAPE** static error (`36 §1.4`: `ρ_inf ⊄
  ρ_decl` — the `[FS]` from `os_write` escapes the empty declared row);
  `send_ok` **accepts** — a **verdict flip** through the **real** `36 §1.4`
  escape check (the same gate L6 I/O rides, **not** a new gate)
- why: AC5 — effects are mandatory at the boundary; the flip is on a **real**
  foreign through the **real** escape check, not a synthetic flag. The accepting
  case carries the row; the rejecting case drops it.

### ffi-io/pure-but-effectful-foreign-is-the-named-residual (AC5, soundness)
- spec: `spec/30-surface/38-ffi-io.md §3.4`, `64`
- given: `foreign sneaky : Int → Int = symbol "..." pure` (declared **empty
  row**) whose C symbol **actually performs I/O**
- expect: the type discipline **cannot** mechanically catch this — the kernel
  sees an empty row and no caller is forced to declare an effect — so it is a
  **reviewer-surfaced flag**, **not** a verdict the system flips. Conformance
  asserts it is **flagged + listed** (the postulate is visible in `P`, §3.1),
  **not** silently accepted as sound; there is **no** claim that the type system
  rejects it
- why: (soundness) AC5 — the **one** named residual the discipline cannot close,
  pinned as the honest limit (`64`). It is *mitigated, not eliminated* by §3.1
  (visible in `P`). This case names a gap (not a netted catch) — the absence is
  the point: a test that asserts the type system *rejects* `sneaky` would encode
  a guarantee Ken does not make (over-claim). Couples C2 (a wrong claim is
  confined + listed).

## F. AC6 — capability + effect compose (couples Sec1/Sec2)

### ffi-io/foreign-world-action-needs-row-AND-capability (AC6)
- spec: `spec/30-surface/38-ffi-io.md §4`, `36 §1.4`, `36 §3`,
  `spec/60-security/62-authority.md`
- given: a `foreign os_write ... visits [FS]` call, in three contexts — (a)
  declares `[FS]` **and** holds `Cap_FS` (`using c : Cap FS`); (b) declares
  `[FS]` but holds **no** `Cap_FS`; (c) holds `Cap_FS` but declares **no**
  `[FS]`
- expect: (a) **accepts**; (b) **rejects** — missing authority (Sec2 `62`: no
  ambient authority, the cap is a real Π value the call needs); (c) **rejects**
  — EFFECT-ESCAPE (`36 §1.4`). **Both** concessions are required; dropping
  **either** rejects
- why: AC6 — authority + flow compose: the row says *may this code perform the
  effect*, the capability says *is it authorized to* (`38 §4`). A 3-way pin
  (like Sec2 F1): a single accept is green-vs-green; dropping each concession in
  turn nets that **neither** alone suffices. Routes a **real** cap + row through
  the real gates.

## G. G6 — a serialization round-trip in an FFI-using verified component

### ffi-io/verified-component-with-foreign-call-and-roundtrip-proof (G6)
- spec: `spec/30-surface/38-ffi-io.md §6` (G6), `§1.5`, `§3.1`
- given: a verified component that makes **≥1** `foreign` call (e.g. reads bytes
  via a `foreign`), carries the provable round-trip law `decode (encode s) == Ok
  s` (`38 §1.5`, one-directional), and is run through the export emitter
- expect: the round-trip obligation is **provable** (verdict `proved` → its
  claim in `Q`); the `foreign` call's postulate appears in the component's
  `trusted_base_delta` → `P` (the trust base **shows exactly what is assumed**);
  the component ships with a **non-empty, enumerated** delta listing the foreign
- why: G6 — a verified component making an FFI call with the trust base showing
  what is assumed (the gate). Ties AC2 (foreign listed in `P`) to the L6
  round-trip (provable in `Q`): the same artifact carries a kernel-proved
  guarantee **and** an honestly-listed assumption — the headline of the whole
  trust-boundary story. The reverse round-trip is **not** asserted (`38 §1.5`).

## Coverage map (AC / gate → case)

| AC / gate | Case |
|---|---|
| AC1 binds + marshals | A1 |
| AC2 foreign-as-listed-postulate | B1 **+** B2 (the dependency pair) |
| AC3 `pure` → trusted, never `Q` | C1 (the over-claim pair) **+** C2 (confined + listed) |
| AC4 boundary contracts runtime-checked | D1 |
| AC5 effects mandatory | E1 (the flip) **+** E2 (the named residual) |
| AC6 capability + effect compose | F1 (3-way) |
| G6 verified FFI component + round-trip | G1 |

## Cross-case sweep (group by metatheory class; assert agreement)

- **Trust-base-projection class agrees.** Every fact a `foreign` contributes
  projects per `71 §2.1`: the foreign **type/postulate** → `P` (B1, B2 absence,
  C2, G1); a **`pure` claim** → `P`, **never** `Q` (C1); a **kernel-proved**
  Ken-side claim → `Q` (C1's `wrap`, G1's round-trip). No foreign fact ever
  lands in `Q`; every kernel-proved fact does — the rows agree.
- **Boundary invariant — honesty (no foreign assumption in `Q`, AC3/I1).** Of
  the set, only the kernel-proved claims (C1 `wrap`, G1 round-trip) reach `Q`;
  every foreign assumption (B1, C1 `c_sqrt`, C2, G1's foreign) lands in `P`. The
  over-claim direction is foreclosed structurally (`21 §5.4`).
- **The honesty pairs are non-degenerate.** AC2 = B1/B2 on dependency (relied-on
  → listed *while* not-relied-on → absent); AC3 = C1 on the same artifact
  (kernel-proved → `Q` *while* foreign-assumed → `P`). Each single case is
  green-vs-green under its bug (list-every-in-scope; trust-the-`pure`-string);
  only the pair nets the discriminator.
- **Effect-tracking class agrees.** E1 and F1(c) are the **same** `36 §1.4`
  escape-check flip (a dropped row → reject) on a real foreign — the ≥2-effect
  discrimination L6 exercises (`FS`/`Net`, `38 §1.3`) carries here.
- **The named residual is named, not netted.** E2 (`pure`-but-effectful) asserts
  a **flag + listing**, not a verdict flip — the honest limit (`64`). A case
  that asserted the type system *rejects* it would over-claim; the absence of
  that claim is deliberate (couples C2: confined + listed).

## Subsumed upstream — not re-derived

- The **meaning** of `proved` (a kernel-re-checked certificate), `unknown`/
  `tested`/postulate, and `trusted_base()` membership is pinned by
  `../../verify/seed-verify.md` + `21 §5`; the **status → export field**
  projection (`proved`→`Q`, `tested`/`unknown`→`P`) and the structural proved↔
  assumed discriminator are pinned by `../../behavioral/export/seed-export.md`
  (EX-A pair, EX-B). These L7 cases **do not re-derive** that machinery; they
  pin the **FFI instance** — the foreign postulate as a `P` entry,
  reliance-by-call, `pure`-not-`Q` (subsume-don't-proliferate).
- The L6 `Bytes`↔`(ptr,len)` boundary and the round-trip law are
  `../bytes-io/seed-bytes-io.md`; AC1 reuses the marshalling boundary, G1 reuses
  the round-trip law — not re-derived.

## Build-sequencing note

- L7 builds on **landed** L6 (`Bytes` marshalling), B1 (the export
  `trusted_base_delta` → `P`), and Sec2 (capability gating, `62`). The
  `declare_postulate` → `trusted_base()` path and the `36 §1.4` escape check are
  on `main`; AC2/AC3 route through the real export, AC5/AC6 through the real
  escape + cap gates.
- Literal `foreign`/`symbol`/`library`/`pure`/`visits` keyword spellings are
  `(oracle)` (`OQ-syntax`, `38 §2.1`) — cases pin the **structure** and tag the
  token, never freezing it (assert-at-locked-granularity).
- The `pure`-but-effectful residual (E2) is the **one** soundness gap the type
  discipline names but cannot mechanically close (`38 §3.4`, `64`) — a reviewer
  flag, not a netted verdict; no build is expected to "fix" it.
