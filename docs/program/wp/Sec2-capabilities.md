# WP Sec2 — capabilities (authority, attenuation, revocation, audit)

**Owner:** Team Verify (WS-Sec — operator-assigned). **Branch:**
`wp/Sec2-capabilities` (cut from `origin/main`). **Stream / gate:** WS-Sec →
**G-Sec** (with Sec1/Sec3/Sec5); **unblocks B4** (agentic boundary). **Depends
on:** L5 (effects + capability machinery, `36 §3`) — **merged**; Sec1
(IFC/labels)
— **merged**. **Spec source:** `spec/60-security/62-authority.md` (+ `36 §3`
capabilities, `36 §4` space, `61` flow/declassification).

> **Steward *frame*** — scope, settled-decision pinning, deliverable outline,
> acceptance, guardrails; the spec enclave elaborates `62` to team-ready rigor +
> conformance before Team Verify builds. **Perishable:** capabilities ride the
> **landed** L5 effect/row machinery (`36 §3`) — pin against the landed code,
> not
> this line.

## 1. Objective (one line)

Deliver Ken's **authority** discipline: **no ambient authority**, capabilities
as
**static, visible, least** typed tokens, **monotone-downward attenuation**,
**transitive revocation** (static contract; runtime membrane deferred), and
**statically-known audit points** — so "this AI-written helper cannot reach the
network" is a **compile-time fact**, not a review hope.

## 2. Settled inputs — FIXED, do not reopen

Per `62` (Committed §7; `OQ-8a`, `OQ-Space` Decided):

1. **No ambient authority (§1).** No global `open`, no implicit FS/network, no
   process-wide mutable singleton. A computation acts on the world **only** with
   an **explicitly-given capability** **and** via a **declared effect** (`36`).
   A
   `view` with no effect row + no capability args is, by its type, **inert**.
2. **Capabilities are static, visible, least (§2, `OQ-8a`).** A capability is an
   **unforgeable first-class value token** (`Cap_FS`, `Cap_Net`,
   `Cap_declassify[ℓ→ℓ']`, …) that is **part of the function's type** — the
   signature **is** the authority manifest, checked **statically**. **Least by
   default:** default authority is **none**; a function holds exactly the caps
   its
   callers pass. Distinct from logical `requires` (`36 §3`).
3. **Attenuation is monotone-DOWNWARD only (§3).** `attenuate : (c : Cap) (w :
   Authority) → { c' : Cap | authority c' ⊑ authority c ⊓ w }` — derives a
   **strictly weaker** capability; **NO operation amplifies authority**. A child
   cannot exceed the parent's delegated authority, **by construction**.
4. **Revocation is transitive (§4).** A revocable capability fails **closed**
   when
   revoked, **and so does everything attenuated from it**. The **runtime
   mechanism**
   (membrane / validity-indexed / region) ties to `OQ-Space` and is **deferred
   to
   `40-runtime`** — Sec2 pins the **static contract**, not the runtime
   realization.
5. **Audit points are statically known (§5).** A trust boundary (`space` edge,
   FFI, declassification, delegation) can emit a **tamper-evident audit
   record**;
   the audit points are **static** (you cannot perform an un-audited effect the
   type didn't declare). **Declassification (`61 §4`) is a capability whose
   every
   use is audited.**
6. **Authority + flow compose (§6).** A capability **gates an effect** and the
   sink
   it opens **carries a clearance label** (`61 §3`): one typed arrow expresses
   *may this code act* (capability) **and** *may this data flow* (label) —
   composes
   with Sec1.

## 3. Mandated deliverable outline (each ends in an implementable choice)

Deliver in the elaborator/type system (capabilities are static — the runtime
membrane is **out of scope**, deferred):

1. **No-ambient-authority enforcement.** A world-acting operation requires an
   explicit capability arg **and** its declared effect row; a no-cap/no-row
   `view`
   is **inert** (rejected if it attempts an effect). Pin the elaborator gate.
2. **Capability tokens in the type.** `Cap_FS`/`Cap_Net`/`Cap_declassify[…]` as
   first-class typed values; the signature-as-manifest; least-by-default (a
   function may use exactly the caps it is passed). Pin the `authority` lattice
   +
   the `⊑` order.
3. **Attenuation.** The typed `attenuate` producing `{c' | authority c' ⊑
   authority c ⊓ w}`; **enumerate the absence of any amplifying operation** (no
   `strengthen`, no cap-forging). Pin the `⊑`-on-`Authority` exactly.
4. **Revocation (static contract).** The typed revoke interface + the
   transitivity
   property (revoking `c` fails-closed everything `⊑`-derived from `c`); **defer
   the runtime membrane** to `40-runtime` (`OQ-Space`) — oracle-tag the
   mechanism,
   pin the contract.
5. **Audit points.** The statically-known boundary set + the audit-record shape
   (what authority, by whom, to what effect); declassification
   every-use-audited.

## 4. Testable acceptance criteria

- **AC1 (no ambient — discriminating)** A world-action **without** an explicit
  capability + declared effect is a **type error**; **with** them, accepts.
  Verdict flips.
- **AC2 (least by default)** A function using a capability it was **not** passed
  is
  rejected; passing it accepts. (Default authority = none.)
- **AC3 (attenuation monotone — THE headline, order-dual discriminating pair)**
  `attenuate` to a **weaker** cap **accepts** WHILE any path to a **stronger**
  cap
  **rejects / does not exist** — authored as the **non-degenerate pair on the
  same
  cap shape** (per the taint-axis order-dual lesson: a backwards `⊑` silently
  inverts attenuate-weakens into attenuate-strengthens, and a single accept case
  cannot net it). Route real `Cap` values through the real `authority`-`⊑`
  check.
- **AC4 (revocation transitive)** Revoking `c` fails-closed `c` **and** every
  cap
  attenuated from it (static contract; the runtime mechanism is oracle-tagged).
- **AC5 (audit points static)** Every trust-boundary effect has a **statically-
  known** audit point; an un-audited boundary effect is **impossible** (the type
  declares it). Declassification: every use audited.
- **AC6 (authority + flow compose)** A `Net` write requires `Cap_Net` **AND**
  the
  data `⊑` the sink's clearance (`61 §3`) — **both** concessions needed;
  dropping
  either rejects (composes with Sec1's IFC).
- **Conformance:** `conformance/security/capabilities/` — AC1–AC6, per-case
  verdict-flip + the AC3 order-dual pair + cross-case sweep (the no-ambient
  class
  agrees). **QA gate:** route **real** capability values through the **real**
  authority check; the attenuation pair must flip (weaker-accepts/stronger-
  rejects), never a synthetic flag.

## 5. Do-not-reopen guardrails

- **No ambient authority** (§1) — the structural precondition; never a global
  world-handle.
- **Attenuation monotone-DOWNWARD only** (§3) — **no amplifying operation
  exists**;
  this is the soundness core (assert the absence, order-dual pair).
- **Capabilities = first-class typed tokens**, distinct from logical `requires`
  (`OQ-8a`).
- **Revocation transitive** (§4); **runtime membrane DEFERRED** to `40-runtime`
  (`OQ-Space`) — build the static contract, not the runtime mechanism.
- **Audit points statically known** (§5) — no un-audited declared effect.
- **The security requirement is fixed regardless of construct form**
  (`OQ-Space`).

## 6. Sequencing notes

- Sec2 **unblocks B4** (agentic boundary = Sec1+Sec2 envelope) and contributes
  to
  **G-Sec** (with Sec1/Sec3/Sec5).
- **AC3 is the order-dual soundness net** — the same trap-class as Sec1's taint
  axis (`⊑` backwards inverts accept/reject; a single positive case guards
  nothing). The enclave must author the distinguishing pair at design time.
- The **runtime realization** (membrane/revocation) is a downstream `40-runtime`
  WP (`OQ-Space`) — keep the static contract clean so it rides.
- Standard §2c: frame → spec-leader elaborates `62` + conformance → merge
  (Architect + conformance-validator) → Team Verify compacted, then kicked off.
