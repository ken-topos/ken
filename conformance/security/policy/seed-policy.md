# Security policy conformance — seed cases (Sec5)

Format: `../../README.md`. These pin the **policy-as-code** layer of
`spec/60-security/65-policy.md` (Sec5, DRAFT v0 → normative; ADR 0007): a
**separately-authored, mandatory, static** policy surface — *authored by
compliance, enforced by the compiler*. The load-bearing property is the
**mandatory, non-weakenable binding guarantee** (§65 §3): relabeling a
policy-`Secret` value `Public`, writing above a sink's clearance, or
declassifying without the policy-granted capability all **fail to compile**.
The soundness anchor is that policy **costs no metatheory and no kernel** (§65
§4): it is the *configuration the lattice-parametric non-interference proof
was waiting for* (`61 §2`/`§5`), so it is **data + binding, not new logic**.
They sit beside the Sec1 IFC (`../ifc/seed-ifc.md`), Sec1ct CT (`../ct/`),
Sec2 authority (`../capabilities/`), and Sec4 trust-model (`../trust-model/`)
seeds.

Grounding (landed `§`-bodies + landed code on this branch, content-reconciled
— not the plan): `65 §1`–`§7` (role separation; policy is a second **input**
to the one engine; the non-weakenable binding; no-metatheory-cost; governance
rides the supply-chain), ADR 0007 (the five locked commitments), `61 §2` (the
lattice-parametric IFC discipline), `61 §5` (by-typing non-interference — a
**trusted** meta-theorem over **erased** labels, `61 §9 N1`), `61 §4`
(declassification, capability-gated), `61 §5a` (`@ct`). **Landed code pinned
against** (`crates/ken-elaborator/src/ifc.rs`): `flows_to` (`:77`,
componentwise `conf≤ ∧ integ≤ ∧ (!ℓ.ct||κ.ct)`), `FlowCtx::l_sink` (`:239`,
`flows_to(join(value,pc), clearance)` — the `pc`-join catches implicit flows),
`join`/`meet` (`:55`/`:64`), `l_ct_sink` (`:260`, `@ct` leakage-sink),
`LeakageSink` (`:100`, the sealed three-member sink set),
`DeclassifyCap`/`is_valid`/`check_declassify` (`:301`/`:309`/`:323`),
`check_declassify_in_delta` (`:349`), and the `Label` note at `:28` ("Labels
are **erased** before the kernel; no kernel primitive introduced"). The `@ct`
runtime timing face is `CtHook.deferred_timing` (`:132`).

## Reading these cases — the Sec5-specific disciplines

**Policy is a second INPUT to the one engine, so enforcement IS the landed IFC
(`65 §4`).** A policy supplies the lattice + classifications + clearances +
edges + ingestion points; Ken's *own* type-checker enforces them via the
**same** `flows_to`/`l_sink`/`check_declassify` the inline discipline uses
(`61 §3`/`§4`). So the security-critical ACs drive **real, landed producers**
— a policy-`Secret` value written to a `Public`-cleared sink Rejects because
`flows_to(SECRET, PUBLIC)` is `false` (`2 ≤ 0` fails), not because of a new
policy engine. What Sec5 *adds* over Sec1 is the **binding** (a policy is
bound program-wide, its labels non-weakenable by the implementer) — and the
binding **mechanism** is `OQ-policy`, **deferred**.

**Defer the spelling, pin the semantics (`OQ-policy`,
[[contract-spec-defer-spelling-not-concept]]).** The concrete policy syntax,
the per-package/build/deployment binding mechanism, and the composition
operator (org → team → service, **monotone-tightening only**) are open
(`65 §6`). Cases that exercise the *deferred binding* drive the **landed
enforcement** with a label standing in for a policy-sourced one,
**oracle-tagged** `[policy-binding deferred: OQ-policy]` — they pin the
**enforcement-identical** semantics and the **non-weakenable** invariant,
never a fabricated policy parser. Over-freezing the spelling would falsely
fail a valid `OQ-policy` resolution.

**Trust faces — get the LEVEL right, do not over-claim "kernel-backed."** The
policy layer's guarantees sit at three levels, tagged per case:
- **`[landed producer]`** — AC1 (§A, `l_sink`/`flows_to`), AC3 (§C,
  `check_declassify`), AC5-static (§E, `l_ct_sink`), AC6-delta (§F,
  `check_declassify_in_delta` + the Sec4 `trusted_base_delta`): a real IFC/
  capability producer returns the accept/reject verdict.
- **`[structural / by-construction]`** — AC2's non-weakenability *enforcement*
  (§B: `flows_to`/`l_sink` take a `Label` with no author/provenance field, so
  authority cannot be weakened at the check) and AC4 (§D: policy reuses the
  existing lattice ops over an erased-label discipline, so the
  kernel/metatheory delta is **empty**). **NOT kernel-proved:** the by-typing
  non-interference is a **trusted** meta-theorem over **erased** labels
  (`61 §9 N1`), so it projects to **`P`/`tested`, never kernel-certified `Q`**
  ([[trusted-by-typing-guarantee-is-not-kernel-proved-Q]]).
- **deferred (oracle-tagged, named triggers)** — the `OQ-policy`
  binding/syntax/ composition (AC2), the `Ward` runtime CT-validation
  (AC5-runtime, → `63 §5a`, trigger `CtHook.deferred_timing`), and the `63`
  provenance transport (AC6). Each pinned in semantics, deferred in spelling;
  none fabricated. A policy corpus that labeled the binding guarantee
  "kernel-backed" would over-claim exactly where a CISO reads it: the
  enforcement is by-typing (trusted), the *soundness* is that this costs no
  new trust (AC4), not that the kernel re-proves the policy.

**The `@ct` axis is order-dual — the distinguishing pair is the sole net
(`61 §5a`, [[taint-axis-orientation-needs-distinguishing-pair]]).** `ct⊥` is
safe, a leakage sink demands `ct⊥`, and `@ct` (`ct⊤`) must never steer it. A
flipped orientation silently inverts accept↔reject, and a single reject case
cannot net it — so AC5 authors the **pair on one sink shape** (a `@ct` value
rejects **while** a safe value accepts). Labels are erased before the kernel,
so the pair is the **sole** net.

## A. The binding guarantee (AC1 ★) — relabel/over-clearance fails to compile

> The ★ load-bearing property (§65 §3 / §65 §7 acceptance): an implementation
> that writes a policy-`Secret` value to a `Public`-cleared sink **fails to
> compile**. The non-degenerate pair is **{A1, A2}** on the real `l_sink`;
> **A3** pins the `pc`-join (implicit flows).

### security/policy/policy-secret-to-public-sink-rejected
- spec: `65 §3` (POL-Bind), `65 §7`, `61 §3.1` (L-SINK)
- given: a value carrying a policy-`Secret` label (`conf=2`) written to a sink
  cleared only for `Public` (`conf=0`) — `l_sink(SECRET, PUBLIC, site)` in a
  pure `pc` (ifc.rs:239)
- expect: **rejects** — `FlowResult::Reject(L-SINK)`;
  `flows_to(SECRET, PUBLIC)` is `false` (`2 ≤ 0` fails). A compile error, not
  a review nit
- why: (soundness ★) AC1, the binding guarantee. Producer: the landed
  `l_sink`/`flows_to` (ifc.rs:239/77) — grep the real IFC check, **not** a
  hand-rolled bool. **Flip:** case A2 (`Internal ⊑ Secret` clearance) accepts
  the same shape. **Trust level: `[landed producer]`.**

### security/policy/label-within-clearance-accepted
- spec: `65 §3` (POL-Bind), `61 §3.1`
- given: a value at `Internal` (`conf=1`) written to a sink cleared for
  `Secret` (`conf=2`) — `l_sink(INTERNAL, SECRET, site)`
- expect: **accepts** — `FlowResult::Accept`; `flows_to(INTERNAL, SECRET)`
  holds (`1 ≤ 2`)
- why: (soundness) AC1, the accepting half of the pair. Same producer as A1;
  the verdict flips on the real product-order check, not a synthetic gate.
  **Trust level: `[landed producer]`.**

### security/policy/implicit-flow-via-pc-rejected
- spec: `65 §3` (POL-Bind), `61 §3.1` (the `pc`-join)
- given: a `Public` value written to a `Public`-cleared sink **inside a branch
  guarded by a `Secret` scrutinee** —
  `FlowCtx::with_pc(SECRET).l_sink(PUBLIC, PUBLIC, site)`
- expect: **rejects** — `l_sink` joins `pc`:
  `flows_to(join(PUBLIC, SECRET) = SECRET, PUBLIC)` is `false`, so the
  implicit flow is caught (L-SINK)
- why: (soundness) AC1. The policy clearance binds **implicit** flows too — a
  dropped `pc`-join would leak the secret scrutinee through control flow.
  **Discriminating:** the value itself is `Public` (accepts with a pure `pc`,
  cf. A2-style), so the reject is driven **solely** by the `pc`-join — a
  producer that dropped it passes A1/A2 but leaks here. **Trust level:
  `[landed producer]`.**

## B. Non-weakenable: the label's authority is the policy (AC2 ★)

> The security-critical face of §3: the governing label is **policy-sourced**,
> so an implementer cannot weaken it. **B1** is the landed structural net
> (enforcement is provenance-blind); **B2** pins the non-weakenable *binding*
> semantics, whose mechanism (`OQ-policy`) is **deferred** — oracle-tagged,
> not fabricated.

### security/policy/enforcement-is-provenance-blind
- spec: `65 §3` (POL-NonWeaken), `65 §4`
- given: two `l_sink` calls with an identical `Secret` value label at a
  `Public` sink — one label conceptually policy-sourced, one inline — through
  the landed `flows_to(label: Label, clearance: Label)` (ifc.rs:77)
- expect: **identical verdict** (both Reject) — `flows_to`/`l_sink` take a
  `Label` with **no author / provenance field**, so the enforcement path
  cannot distinguish (or be weakened by) a label's origin
- why: (soundness ★) AC2, the **structural** non-weakenability net. There is
  no provenance channel by which an implementer's re-declaration reaches the
  check (the Sec4-AC3 analog: the enforcement signature admits no authority
  input). A hypothetical enforcement that consulted label-provenance
  (accept-if-implementer- lowered) is **inexpressible** at `flows_to`'s
  signature. **Trust level: `[structural / by-construction]`.**

### security/policy/implementer-inline-relabel-cannot-override-policy-label
- spec: `65 §3` (POL-NonWeaken), `65 §6` (`OQ-policy`)
- given: a policy binds a value's label to `Secret`; an implementer inline-
  re-declares that value `Public` and writes it to a `Public` sink —
  `[policy-binding deferred: OQ-policy]` (the label is policy-authored, not
  inline; binding mechanism deferred)
- expect: **rejects** once the policy is bound — the policy-`Secret` label
  governs; the inline `Public` re-declaration does **not** change the flow
  verdict (the write is still `SECRET → Public` under the bound policy)
- why: (soundness ★) AC2. **Non-weakenability is a property of the *landed*
  enforcement** (B1: `flows_to`/`l_sink` do not consult authorship), NOT of
  the deferred binding — an implementer's inline `Public` re-declaration
  cannot change the verdict because the **identical** `l_sink` runs regardless
  of who authored the label (the laxer inline label is either overridden by
  the bound policy label or is itself a violation, §4 one engine).
  **Defer-spelling ([[contract-spec-defer-spelling-not-concept]]):** only the
  label-*sourcing* mechanism (which scope, the concrete syntax) is
  `OQ-policy`-deferred — oracle-tag it; the enforcement is landed and
  identical. **Not a fabricated policy parser.** **Trust level: enforcement
  `[landed producer]` (B1) / label-sourcing `[deferred: OQ-policy]`.**

## C. Declassification requires the policy-granted capability (AC3)

> The pair is **{C1, C2}** on the real `check_declassify` — no capability
> rejects, the policy-granted capability admits. **C3** pins the downgrade
> *direction* (`is_valid`: `to ⊑ from`, strictly lower).

### security/policy/declassify-without-capability-rejected
- spec: `65 §3` (POL-Declassify), `61 §4`, `62 §5`
- given: a downgrade `Secret → Public` attempted with **no** declassification
  capability — `check_declassify(None, SECRET, SECRET, PUBLIC)` (ifc.rs:323)
- expect: **rejects** —
  `DeclassifyResult::Reject { "missing Cap_declassify" }`
- why: (soundness) AC3. A downgrade is the **only** way a label moves down,
  and only under the policy-granted capability (`62`). **Flip:** case C2
  (valid capability) admits it. Producer: the landed `check_declassify`, not a
  stub gate. **Trust level: `[landed producer]`.**

### security/policy/declassify-with-policy-capability-accepted
- spec: `65 §3` (POL-Declassify), `61 §4`
- given: the same downgrade with a valid `Cap_declassify[Secret→Public]` in
  scope, matching edge and value —
  `check_declassify(Some(cap), SECRET, SECRET, PUBLIC)`
- expect: **accepts** —
  `DeclassifyResult::Accept { downgraded_label: PUBLIC }`
- why: (soundness) AC3, the accepting half. The policy-granted capability is
  the authority; the `requires` **proven precondition** (`61 §4`) is the Sec1
  face (cross-ref, landed there — not re-specified here). **Trust level:
  `[landed producer]`.**

### security/policy/upgrade-masquerading-as-declassify-rejected
- spec: `65 §3` (POL-Declassify), `61 §4` (`is_valid`: genuine downgrade)
- given: a capability whose edge is an **upgrade** `Public → Secret`
  (`to=Secret` ⊋ `from=Public`) —
  `DeclassifyCap::new(PUBLIC, SECRET).is_valid()`
- expect: **rejects** — `is_valid` is `false` (`flows_to(SECRET, PUBLIC)`
  fails, so `to ⊑ from` does not hold); `check_declassify` → Reject "invalid:
  to ⊋ from"
- why: (soundness) AC3, the **direction** guard. Declassification is monotone-
  **down**; a reversed `is_valid` would admit an upgrade-as-declassify (a
  privilege escalation). **Discriminating on direction** — the non-degenerate
  endpoint (`Public ≠ Secret`), so a flipped order does not pass by refl.
  **Trust level: `[landed producer]`.**

## D. No new metatheory / no new kernel (AC4 ★) — the soundness anchor

> The Sec4-AC4 analog: adding a policy introduces **no new kernel feature and
> no new metatheorem**. `[structural / by-construction]`, **NOT**
> "kernel-backed."

### security/policy/policy-lattice-runs-through-unchanged-ifc-path
- spec: `65 §4`, ADR 0007 §4, `61 §2` (lattice-parametric), `61 §5` (by-typing
  NI)
- given: a policy supplying a valid bounded lattice `ℒ` (the DLM product
  order, `61 §2`) with its classifications and clearances
- expect: it type-checks via the **existing** IFC ops
  (`join`/`meet`/`flows_to`, ifc.rs:55/64/77) with **no new kernel emission**
  and **no new trusted primitive**; the metatheory delta is **empty** — the
  same by-typing non- interference (`61 §5`) holds for the policy's `ℒ`
- why: (soundness ★) AC4, the structural soundness anchor. Labels are **erased
  before the kernel** (ifc.rs:28) — a policy is *data + binding*, so it
  reaches the kernel as nothing. **Not a verdict-flip** but a structural
  invariant ([[soundness-AC-static-vs-runtime-face]] / the Sec4-AC4 form): the
  bug it guards is a "policy" that demanded a new kernel rule or a new
  metatheorem; the conformant policy adds **neither**. **Trust level:
  `[structural / by-construction]` — the by-typing NI is a *trusted*
  meta-theorem over erased labels (`61 §9 N1`), projecting to `P`/`tested`,
  never kernel-certified `Q`
  ([[trusted-by-typing-guarantee-is-not-kernel-proved-Q]]); NOT
  "kernel-backed."**

## E. CT requirement binds a data class (AC5) — static landed / runtime deferred

> A policy "`KeyMaterial` is `@ct`" compiles to the **static** `@ct`
> leakage-sink discipline (Sec1ct, landed). The pair is **{E1, E2}** on
> **one** sink shape (the order-dual net). The **runtime** Ward CT-validation
> face is a **named deferred trigger**, not delivered here.

### security/policy/ct-value-steering-leakage-sink-rejected
- spec: `65 §2` (AC5, CT static face), `61 §5a.3` (L-CT-SINK)
- given: a policy-`@ct` value (`ct=true`, e.g. `KeyMaterial`) steering a
  leakage sink — `l_ct_sink(CT_TOP, BranchGuard, site)` (ifc.rs:260), a
  control-flow branch scrutinee
- expect: **rejects** — `FlowResult::Reject(L-CT-SINK)`; `value.ct || pc.ct`
  is `true`
- why: (soundness ★) AC5-static. A `@ct` value must never steer a
  branch/index/ var-time op (`61 §5a`). **Flip:** case E2 (safe value, same
  sink) accepts — the **order-dual distinguishing pair**; a flipped `ct`
  orientation would invert both, so the pair on **one** sink shape is the sole
  net ([[taint-axis-orientation-needs-distinguishing-pair]]). **Trust level:
  `[landed producer]` (static face). Runtime Ward CT-validation → `63 §5a` is
  a named deferred trigger (`CtHook.deferred_timing`, ifc.rs:132),
  oracle-tagged — not delivered ([[soundness-AC-static-vs-runtime-face]]).**

### security/policy/ct-safe-value-at-same-sink-accepted
- spec: `65 §2` (AC5), `61 §5a.3`
- given: a non-`@ct` value (`ct=false`) at the **same** leakage sink —
  `l_ct_sink(CT_BOT, BranchGuard, site)`
- expect: **accepts** — `FlowResult::Accept`; `value.ct || pc.ct` is `false`
- why: (soundness) AC5-static, the accepting half of the order-dual pair on
  one sink shape. Same producer as E1; the verdict flips on the `ct` taint
  alone — the discriminator that a single reject case could not net. **Trust
  level: `[landed producer]` (static face).**

## F. Governance rides the supply-chain (AC6) — cross-ref contract face

> Not a Sec5-local mechanism: the governing policy + exercised
> declassification authority are the **payload** of the `63` supply-chain
> attestation. Cross-link, do **not** re-specify `63`.

### security/policy/policy-and-declassify-authority-recorded-in-delta
- spec: `65 §5` (POL-Governance), `63 §5`, `25 §3` (`trusted_base_delta`)
- given: a build governed by policy `vX` that exercised a declassification
  authority — the authority id and the policy hash/version recorded for the
  package's `trusted_base_delta`
- expect: the exercised declassification authority **surfaces** in the delta —
  `check_declassify_in_delta(authority_id, delta)` is `true` (ifc.rs:349); a
  package that omits it silently hides the downgrade (the delta-completeness
  backstop, `25 §3`, reusing the Sec4 landed enumeration)
- why: (fidelity, contract-posture) AC6. Policy (what is allowed) +
  per-package attestation (what was used) = "this build provably honoured org
  policy `vX`." **Cross-ref only:** the provenance **transport**
  (signature/SLSA) is `63`'s, **deferred** — this pins the
  `trusted_base_delta` payload face (landed), not the `63` attestation shape.
  **Trust level: `[landed producer]` for the delta face / `63` provenance
  transport `[deferred]`.**
