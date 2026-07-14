# Capabilities & authority conformance — seed cases (Sec2)

Format: `../../README.md`. These pin the **authority discipline** of
`spec/60-security/62-authority.md` (Sec2, elaborated impl-ready): no ambient
authority, capabilities as static/visible/least tokens, **monotone-downward
attenuation**, transitive revocation (static contract), statically-known audit
points, and authority+flow composition. They sit beside the Sec1 IFC seed
(`../ifc/seed-ifc.md`), with which it **composes** (AC6), and **supersede**
the two `authority/` placeholders in `../seed-security.md` (`attenuation-cannot-
amplify` → AC3, `no-ambient-authority` → AC1; retired there in this WP — see
"Placeholder absorption").

Grounding (landed `§`-bodies + landed code on this branch, content-reconciled —
not the plan): `62 §1`–`§9`/`§H` (no-ambient, the authority lattice, attenuation
as an emitted but direction-degenerate refinement obligation, revocation static
contract, audit, compose, trust-boundary table); `36 §2.5`/`§3`/`§3.1` (the
capability-passing
translation: `Cap E` is a value parameter via ordinary Π, minted by a handler;
the cross-workstream contract "capability `Cap E` → a value parameter (Π) → read
by Sec2 authority/attenuation `62`"); `36 §1.4` (the EFFECT-ESCAPE check — a no-
row `view` is inert); `34 §5`/`21 §2` (refinement = carrier + emitted obligation
— the attenuation encoding); `16 §1`/`61 §2.1` (the `Ω`-valued `⊑` order the
`Authority` lattice rides); `61 §3.1` (`L-SINK`, the flow half of AC6); `61 §4`
(declassify is capability-gated/audited); `25 §3`/`63` (`trusted_base_delta`).
**Landed code pinned against:** `CapParam { name, effect }` + `cap_set`
(`crates/ken-elaborator/src/effects/algebra.rs`), `cap_params` on `EffectSig`
(`effects/infer.rs`); `DeclassifyCap { from, to }` with `is_valid` (`to ⊑ from`
∧ strict), `check_declassify`, `check_declassify_in_delta`
(`crates/ken-elaborator/src/ifc.rs`) — the **declassification special case** of
the monotone-downward, validity-checked, delta-audited capability Sec2
generalizes.

**I-4 §C RESHAPE boundary.** This seed's attenuation cases name the semantic
runner/host operation realized by `capabilities.rs::attenuate`; they do **not**
place `attenuate` in Ken's source environment. The program-facing I-4 wrappers
only consume opaque capabilities minted from the parsed program header:
`readFile` is authority-polymorphic and `writeFile` requires `Cap AFull`. Ken
source exposes neither an opaque-`Cap` constructor nor any capability-producing
attenuation function. Section C therefore observes host-derived capability
values and their real authority checks; a Ken source call is not its fixture.

## Reading these cases — the Sec2-specific disciplines

**Capabilities are REAL Π values, but attenuation's bound is trusted-Rust +
conformance-netted (`62 §3.1`/`§H`, ADR-0017 §2).** A `Cap E` is a real Π
parameter (`36 §2.5`), so **cap presence** has a genuine independent kernel net:
a missing-capability `perform` denotes to an unbound variable and is rejected.
That does not make the attenuation bound independently kernel-proved. For
`authority c' ⊑ authority c ⊓ w`, `discharge_attenuation` emits an `Eq` + `Refl`
obligation over fresh opaque postulates, but the elaborator's Rust comparison
chooses whether `child` and `bound` name the same postulate. The kernel never
computes the meet or sees the authority lattice; its discharge mirrors the
elaborator's decision. The real net for the bound is the elaborator's `meet`/`⊑`
computation plus the non-degenerate {C1, C2} conformance pair. This gives the
bound the same trust class as declassify: trusted-by-typing and conformance-
netted, not an independent kernel proof. These cases assert **accept/reject of
the elaboration** and the emitted obligation structurally — never a synthetic
`is_weaker` predicate over literals (that guards nothing).

**The order-dual orientation — the `[Sec2-dual]` discipline (`62 §3.2`; the
`[Sec1-dual]` trap-class).** `⊑` on `Authority` is a **direction**: *more
authority is higher*, `attenuate` moves **down**, and a sink demands *at least*
its authority (`a ⊑ authority c`). Getting `⊑` **backwards** (the bound or the
sufficiency check as `⊒`) **silently inverts** attenuate-weakens into
attenuate-strengthens — privilege escalation. **The kernel obligation alone does
NOT net it:** the canonical witness `authority c' = authority c ⊓ w` discharges
**both** orientations by `⊑-refl` (the bound is **direction-degenerate at the
meet**). So the orientation is held **only** by the **non-degenerate
distinguishing pair** {C1, C2} on **strict** authorities (`authority c ⊓ w ⊏
authority c`): a weaker cap **accepts** at a weak sink **while** it **rejects**
at a sink demanding the parent's full authority. A single accept case is
green-vs-green and nets nothing — the same lesson as the taint-axis orientation
pair and the cast-direction non-degenerate-endpoints rule.

**Route REAL `Cap` values through the REAL `authority`-`⊑` check (the QA gate,
`62 §9`).** The AC3 pair must use real runner/host capability values and the
real authority order with non-degenerate authorities; never a boolean flag or a
Ken-callable constructor. A degenerate (meet-equal) instance collapses the
direction and passes green-vs-green under a flipped order.

**No amplification is an ABSENCE — assert no operation exists (`62 §3.2`).**
"Downward-only" rests on three facts: the trusted-Rust, conformance-netted
semantic bound (above),
the **enumerated absence** of any Ken-callable capability producer
(`attenuate`/`strengthen`/`amplify`/public-`Cap` constructor), and
**unforgeability** (`62 §2.2`: `Cap E` is opaque). The semantic host operation
still satisfies the `⊑` bound at `w = ⊤`; that does not expose it to Ken source.

**Static contract vs runtime face — the deferred runtime is `(oracle)`-tagged,
named not omitted (`62 §4`/`§5`/`§H`).** Revocation and audit each have a STATIC
face (delivered here: the typed interface + the transitivity/boundary property)
and a RUNTIME face (DEFERRED to `40-runtime`/`Ward`: the membrane that fails
closed at eval; the record emission/tamper-evidence). Each case states **which
face it pins** and `(oracle)`-tags the deferred face — never asserts the runtime
guarantee as if delivered (that would over-claim past Sec2's locked scope).

**Tags.** `(soundness)` = a real authority commitment that must never regress
(AC1, AC3, AC6, and the static contracts of AC4/AC5). `(oracle)` = confirm
against Ken's reference elaborator once it exists, and (defer-spelling-not-
concept) the literal source spellings `Cap_FS`/`using`/`revoke` stay
`OQ-syntax`-deferred. `attenuate` is only the semantic host-operation name, not
a deferred Ken token. Cases pin the **value-set + invariants** (a typed
unforgeable token; the `⊑`-bounded semantic derivation; the audited boundary),
not a capability-producing surface spelling; deferred runtime mechanisms remain
`(oracle)` likewise.

---

## A. No ambient authority (AC1) — the structural precondition

### security/capabilities/world-action-without-capability-rejected
- spec: `62 §1`, `36 §2.5`/`§7.3`
- given: a world-action (`write_at p d`, a `visits [FS]` op) in a `view` that
  declares **no** `Cap_FS` parameter (and/or no `FS` row)
- expect: **rejects** — the `perform_FS` denotes to an unbound `Cap_FS`
  reference; a missing-capability error (kernel-ill-typed; the elaborator
  surfaces the source-located diagnostic, `36 §7.3` class 2)
- why: (soundness) AC1, no ambient authority. **Kernel-backed flip:** the *same*
  body with the `Cap_FS` parameter present **accepts** — right=accept (cap +
  row) / wrong=reject (no cap), on the real Π-binding discriminator, not a
  synthetic gate. A no-row/no-cap `view` is provably inert (`ITree 𝟘 ≅ B`,
  `36 §2.4`).

### security/capabilities/no-row-view-is-inert
- spec: `62 §1`, `36 §1.4`/`§2.4`
- given: a `view classify (x) : Tag` with **no** effect row, whose body attempts
  any effect
- expect: **rejects** — EFFECT-ESCAPE (`ρ_inf ⊄ ρ_decl = ∅`), naming the
  escaping effect + a witness perform/call
- why: (soundness) inert-by-type precondition every authority claim rests on.
  Flip: declaring the row (and holding the cap) accepts; the bare `view` cannot
  perform. (Distinct from A1: here the **row** is absent, not just the cap — the
  two halves of "no ambient".)

## B. Least by default (AC2)

### security/capabilities/uses-unpassed-capability-rejected
- spec: `62 §2`, `36 §2.5` (`cap_params`)
- given: a `view` whose body uses a `Cap_Net` it was **not** passed (no
  `Cap_Net` in `cap_params`, no enclosing handler providing it)
- expect: **rejects** — default authority is `∅`; the capability is an unbound
  reference
- why: (soundness) AC2, least by default (PoLA). Flip: passing `Cap_Net` (adding
  the `CapParam`) **accepts** the same body — right=accept / wrong=reject on
  whether the cap is in scope. A function holds **exactly** the caps it is
  passed, never ambient ones.

## C. Attenuation — monotone-downward (AC3, THE headline)

> The order-dual distinguishing pair is **{C1, C2}** on the **same** attenuated
> cap, strict authorities (`authority c ⊓ w ⊏ authority c`): verdict **flips**
> on the sink's demand. C3 pins the emitted-but-degenerate discharge mechanism;
> C4 pins the **absence**.

### security/capabilities/attenuated-cap-at-weak-sink-accepts
- spec: `62 §3`/`§3.1`
- given: the runner/host derives `c_tmp` from `c` and window `dir "/tmp"` via
  its semantic attenuation operation (so `authority c_tmp = authority c ⊓
  "/tmp" ⊏ authority c`), then uses it at a sink demanding only
  `authority c_tmp`'s scope (`a_weak ⊑ authority c_tmp`)
- expect: **accepts** — the child's reduced demand is met (`a_weak ⊑ authority
  c_tmp`)
- why: (soundness) the **accept** half of the order-dual pair. **Necessary but
  degenerate alone** — green under *both* `⊑` orientations (the meet-witness
  satisfies `a_weak ⊑ authority c_tmp` and its reverse), so this case **cannot**
  net the orientation by itself. Pairs with C2.

### security/capabilities/attenuated-cap-at-strong-sink-rejects
- spec: `62 §3.2`/`§3`/`§7`
- given: **same** `c_tmp` (`authority c_tmp ⊏ authority c`, strict) used at a
  sink demanding the **parent's full** authority `authority c` (e.g.
  `write_at c_tmp (path "/etc/passwd")`)
- expect: **rejects** — sufficiency needs `authority c ⊑ authority c_tmp`, false
  by strictness; the weakened cap is **insufficient**
- why: (soundness) **THE net.** This is where the order-dual orientation lives:
  under a **backwards** `⊑` (sufficiency `authority c_tmp ⊑ authority c`) this
  would **wrongly accept** — a weakened cap passing a strong sink (privilege
  escalation). The pair C1↔C2 flips green↔red on exactly the orientation bug; a
  single accept case (C1) is green-vs-green. **Non-degenerate** authorities are
  required (a meet-equal `c_tmp` collapses it). Real `Cap` through the real
  `authority`-`⊑` check (QA gate), never a synthetic flag.

### security/capabilities/attenuate-bound-discharge-mirrors-elaborator
- spec: `62 §3.1`/`§H`, `34 §5`, `21 §2`, `23 §1`, `18 §4`
- given: the runner/host semantic `capabilities.rs::attenuate(c, w)` derives a
  child and its **emitted obligation**; observe the discharge core for
  `authority c' ⊑ authority c ⊓ w`. No Ken source expression constructs `c'`.
- expect: the elaborator **emits an obligation** for
  `authority c' ⊑ authority c ⊓ w` (`22 §2.1`) as `Eq(child, bound)` over fresh
  opaque postulates. For the canonical child, the elaborator chooses the same
  postulate for both sides and `Refl` discharges. For a too-strong child it
  chooses distinct postulates, so `Refl` yields `Unknown` and the obligation is
  **undischargeable**. The kernel never computes the meet or `⊑` relation.
- why: (soundness) **trust-boundary assertion**, not an independent kernel net.
  The emitted `Eq` + `Refl` mechanism is real, but its postulate identities are
  chosen by the elaborator's Rust comparison. Therefore the discharge mirrors
  the elaborator's decision and is direction-degenerate; the elaborator's
  `meet`/`⊑` computation plus the non-degenerate {C1, C2} pair nets the bound.

### security/capabilities/no-amplifying-operation-exists
- spec: `62 §3.2`/`§2.2`
- given: a holder of `c : Cap E` seeking a `c'' : Cap E` with `authority c'' ⊐
  authority c` — enumerate Ken's source environment for `attenuate`/
  `strengthen`/`amplify`/a public `Cap` constructor, while separately applying
  the runner/host semantic attenuation at `w = ⊤`
- expect: **no Ken-callable capability producer exists** and `Cap E` exposes no
  constructor (`62 §2.2`). The separate semantic host derivation still yields
  `authority c' ⊑ authority c` and cannot exceed the parent even at `w = ⊤`.
- why: (soundness) **assert absence** — "downward-only" is the conjunction of
  the trusted-Rust + conformance-netted semantic bound ({C1, C2, C3}), this
  enumerated Ken-surface absence, and unforgeability (`62 §2.2`). The kernel
  cannot witness a missing source
  operation, so the **absence** is the guard. (Absorbs `../seed-security.md`'s
  `attenuation-cannot-amplify` — a child cannot exceed `w`.)

## D. Revocation — transitive (AC4, static contract)

### security/capabilities/revoke-is-transitive-static-contract
- spec: `62 §4`/`§H`, `36 §4`/`§4.4` (`(oracle)` runtime)
- given: the runner/host derives `c_child` from `c` and `w`, delegates the
  child, then applies the semantic `revoke c` operation
- expect: **(static face — delivered)** the typed contract holds: `c_child`'s
  validity is **derived from** `c`'s, so "`c` revoked ⇒ `c_child` (and its
  descendants) revoked" follows from the delegation tree — transitive, fail-
  closed. **(runtime face — `(oracle)`/DEFERRED)** the membrane that *realizes*
  fail-closed at evaluation (forwarder / validity cell flip, `36 §4`) is
  `40-runtime`/`OQ-Space` — **named, not asserted here**
- why: (soundness, static face) AC4. **Transitivity is the discriminator:** a
  non-transitive impl revoking only `c` (leaving `c_child` live) would pass a
  parent-only check — so the case asserts the **child** fails closed too. The
  runtime fail-closed behavior is the deferred face, `(oracle)`-tagged — Sec2
  pins the contract, not the mechanism (static-vs-runtime-face split, named).

## E. Audit points statically known (AC5)

### security/capabilities/unaudited-boundary-effect-is-impossible
- spec: `62 §5`, `36 §1.4`/`§3.1`
- given: a trust-boundary effect (`space` / FFI / declassify / delegation)
  attempted **without** a declared audit point (no row declaring it)
- expect: **rejects** — an un-audited boundary effect is impossible: you cannot
  perform an effect the row did not declare (`36 §1.4`), and the boundary set =
  the `Vis` nodes the type declares (`36 §3.1`)
- why: (soundness, static face) AC5 — the audit points are **statically known**
  because they are the type's `Vis` nodes. The runtime record **emission**
  (serialization / tamper-evidence) is the deferred face (`(oracle)`,
  runtime/`Ward`). Flip: a boundary effect *with* its declared row/point is
  performable-and-audited; without it, impossible.

### security/capabilities/declassify-every-use-audited-and-in-delta
- spec: `62 §5`, `61 §4`, `25 §3`, `63`; landed `check_declassify_in_delta`
- given: a package performing an authorised `declassify` (a capability whose
  every use is an audit point)
- expect: the declassification authority appears in `trusted_base_delta`
  (`check_declassify_in_delta` true); **rejects as an honesty violation** if the
  package downgrades but **omits** the authority from the delta
- why: (soundness) declassify is the capability whose every use is audited
  (`62 §5`); the delta-completeness backstop is the sole net for a hidden
  downgrade (the same guard as Sec1 B3, here owned by `62`). Flip on
  presence/absence in the delta. (Reuses the landed Sec1 `ifc.rs` check — Sec2
  states it as the audit tie, not a new mechanism.)

## F. Authority + flow compose (AC6)

### security/capabilities/net-write-needs-capability-and-clearance
- spec: `62 §6`, `61 §3.1` (`L-SINK`)
- given: `send c s msg` to `s : Socket Public`, exercised three ways — (i) `c :
  Cap_Net` present **and** `msg : Bytes @ Public`; (ii) `c` present **but** `msg
  : Bytes @ Secret`; (iii) `msg @ Public` **but** no `Cap_Net`
- expect: (i) **accepts** (both concessions); (ii) **rejects** — `Secret ⊔ pc ⋢
  Public`, IFC-FLOW error **despite** holding `Cap_Net`; (iii) **rejects** — a
  missing-capability error **despite** a clean flow
- why: (soundness) AC6 — authority and flow are **independent** concessions and
  **both** are required; dropping **either** rejects. Authority does not buy
  clearance, clearance does not buy authority. **Two flips** on one sink
  shape: the cap discriminator (kernel-backed, `62 §1`) and the flow
  discriminator (erased-label/trusted, `61 §H`) — composes Sec1 + Sec2, neither
  green-vs-green.

---

## Coverage map (AC → cases)

- **AC1** no ambient → A1 (`world-action-without-capability-rejected`),
  A2 (`no-row-view-is-inert`).
- **AC2** least by default → B1 (`uses-unpassed-capability-rejected`).
- **AC3** attenuation monotone-downward (THE headline) → **C1↔C2** (the order-
  dual non-degenerate pair), C3
  (`attenuate-bound-discharge-mirrors-elaborator`, emitted-but-degenerate
  mechanism), C4 (`no-amplifying-operation-exists`, the absence).
- **AC4** revocation transitive → D1 (static contract; runtime `(oracle)`).
- **AC5** audit points static → E1 (`unaudited-boundary-effect-is-impossible`),
  E2 (`declassify-every-use-audited-and-in-delta`).
- **AC6** authority + flow compose → F1 (`net-write-needs-capability-and-
  clearance`).

## Cross-case consistency sweep (pre-handoff gate)

- **No-ambient / least class {A1, A2, B1}** — agree: every world-action needs an
  explicit `Cap E` **and** a declared row; default authority is `∅`; a no-cap or
  no-row `view` is inert. Each a clean verdict flip (cap/row present → accept;
  absent → reject), kernel-backed (real Π binding), none green-vs-green.
- **Order-dual orientation {C1, C2}** — the load-bearing pair: the **same**
  attenuated cap **accepts** at a weak sink **while** it **rejects** at a sink
  demanding the parent's full authority, on **non-degenerate** authorities
  (`authority c ⊓ w ⊏ authority c`). A flipped `⊑` inverts **both** verdicts, so
  the pair (not a single case) holds the orientation. The `[Sec2-dual]` net,
  the same shape as `[Sec1-dual]` integrity/`@ct` (`../ct/`) and the cast-
  direction non-degenerate-endpoints rule.
- **Two faces of the bound {C2, C3}** — agree without merging: C3 pins that an
  `Eq` + `Refl` obligation is emitted and that a too-strong child becomes
  undischargeable, while also pinning that the elaborator chooses the postulate
  identities. Therefore the discharge is **not an independent kernel proof** of
  `authority c' ⊑ authority c ⊓ w`; it mirrors trusted Rust. C2 supplies the
  non-degenerate conformance net for the orientation that C3 cannot distinguish
  at the meet. Treating C3 as a stronger kernel net, or C2 as redundant, is the
  bug this pair forecloses (`62 §3.2`/`§H`, ADR-0017 §2).
- **Semantic operation vs Ken surface {C1–C4}** — all four attenuation cases
  observe the runner/host operation and real `Cap` values. C4 independently
  requires the Ken source environment to expose neither `attenuate` nor any
  other `Cap` producer. A reading that turns the semantic operation into a
  source wrapper contradicts the I-4 §C RESHAPE and fails this class.
- **Static-vs-runtime faces {D1, E1, E2}** — agree on scope: Sec2 delivers the
  **static** contract (typed interface, transitivity, type-determined boundary
  set, delta-completeness); the **runtime** faces (revocation membrane fail-
  closed, audit-record emission) are `(oracle)`/DEFERRED to `40-runtime`/`Ward`,
  **named not omitted**. No case asserts a runtime guarantee as delivered — that
  would over-claim past the locked Sec2 scope (`62 §H`).
- **Compose class {F1}** — both concessions independent and required; the cap
  discriminator (kernel-backed) and the flow discriminator (trusted-by-typing)
  each flip on the same sink shape — composes Sec1 + Sec2.

## Placeholder absorption (reconcile note)

This seed supersedes the two `authority/` cases in `../seed-security.md`:
`attenuation-cannot-amplify` → **C4** (+ the C1↔C2 pair sharpening "cannot
amplify" into the order-dual orientation net); `no-ambient-authority` → **A1**
(+ A2 splitting the cap and row halves). Both retired from `seed-security.md`
in this WP to avoid a stale-sibling contradiction; the `supply-chain/` and
`trust/` placeholders there are Sec3+ and untouched.

## Build-sequencing note

AC1–AC3 + AC5-static + AC6 exercise the landed capability-passing,
authority-check, and refinement-obligation machinery (`36 §2.5`, `34 §5`/
`21 §2`). Section C reaches `capabilities.rs::attenuate` and its discharge
directly at the semantic host boundary; it deliberately has no Ken-callable
attenuation fixture. The **deferred runtime faces** — the revocation membrane
(D1) and audit-record emission (E1) — land in `40-runtime`/`OQ-Space` and carry
`(oracle)` reify-tags, **not** Sec2. The authority lattice `⊑` rides `Ω`
(`16 §1`, level 0 for finite carriers); the semantic refinement `{c' | …}`
lands at `level(Cap E) = ℓ_op` (predicative, same as the carrier, `21 §2`/
`62 §9`). Literal source spellings `Cap_FS`/`revoke` stay
`OQ-syntax`-deferred; cases pin value-sets + invariants, never a Ken-callable
`attenuate` token. Sec2 **unblocks B4** (the agentic boundary = Sec1 + Sec2
envelope) and contributes to **G-Sec**.
