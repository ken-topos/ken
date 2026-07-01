# Sec5 — policy-as-code (§65): authored by compliance, enforced by the compiler

**Steward frame.** WP `Sec5`. Enclave WP (spec-hardening + conformance), `§2c`
process. Owner on release: spec-leader → spec-author (`/spec`) +
conformance-validator (`/conformance`). Reviewers: Architect (soundness), CV
(Spec on `/spec`), spec-author (Fidelity on `/conformance`).

## Why this WP, and why now

`Sec5` rounds out the G5 security story (Sec1 IFC · Sec2 authority · Sec4
TCB → **Sec5 policy**). It hardens `spec/60-security/65-policy.md` and seeds its
conformance. It is **independent** and — critically — **cheap on soundness**:
the policy layer is *"not a second engine, it is a second **input** to the one
engine"* (§65 §1). Its binding guarantee bottoms out in the **landed IFC
discipline** (`crates/ken-elaborator/src/ifc.rs` — `flows_to` `:77`, `l_sink`
`:239`), so the security-critical conformance drives **real producers**, unlike
a purely-deferred contract spec. The enclave is warm on the security corpus and
competent at contract specs with oracle-tagged deferred surfaces (the B-series).

## Locus

- **Spec:** `spec/60-security/65-policy.md` — harden **DRAFT v0 → normative** for
  the **shape + binding guarantee** (what ADR 0007 commits). The concrete policy
  **syntax** is `OQ-policy` (open) — this WP pins the *semantics* and
  **defers the spelling** (see Pinned/Deferred below), it does **not** resolve
  `OQ-policy`.
- **Conformance (new):** `conformance/security/policy/` — does not exist yet
  (security subtree has `capabilities/`, `ct/`, `ifc/`, `trust-model/`). Author
  the seed + discriminating cases following `conformance/security/seed-security.md`
  house style.

## Pinned decisions — settled (ADR 0007), do NOT reopen

1. **Role separation** (§65 §1): security/compliance author *what the policy is*
   (lattice, classifications, clearances, declassification edges, ingestion
   points); implementers write conforming code. Classification authority is
   **not** the implementer's — an implementer cannot declare PII `Public`.
2. **Policy is IN Ken, static, one engine** (§65 §1, §4): the policy layer is a
   **second input** to Ken's own type-checker, enforced at compile time — **not**
   a sibling (contrast Ward), **not** a second engine. What is separate is the
   **authoring role**, mirroring the federation.
3. **★ Mandatory + non-weakenable binding guarantee** (§65 §3, the load-bearing
   property): a policy binds a build/package and is imposed **program-wide as a
   constraint the implementer cannot weaken**. Relabeling a policy-`Secret` value
   `Public`, writing to a sink above its policy clearance, or declassifying
   without the policy-granted capability all **fail to type-check** — a compile
   error, not a review nit.
4. **★ No new metatheory, no new kernel** (§65 §4, the soundness anchor): because
   the IFC discipline is **lattice-parametric** (§61 §2, non-interference proved
   once for any bounded `ℒ`), a policy is exactly *the configuration the generic
   proof was waiting for*. The policy layer adds **nothing** to the metatheory
   and **nothing** to the kernel — the same by-typing non-interference theorem
   (§61 §5) holds for whatever lattice the policy supplies. Policy is **data +
   binding, not new logic** (reflect-don't-extend / subsume-don't-proliferate).
5. **Org-scale governance rides the supply-chain** (§65 §5, §63): the governing
   policy + any declassification authority exercised appear in the package's
   `trusted_base_delta` + provenance (`25 §3`). Policy (what is allowed) +
   per-package attestation (what was used) = "this build provably honoured org
   policy `vX`." This couples to Sec3 (`§63`, supply-chain) — **cross-reference
   only; do not re-pin the §63 attestation shape here.**

## Deferred — oracle-tag via defer-spelling, do NOT invent

`OQ-policy` is open (§65 §6). Defer the **spelling**, pin the **semantics**
([[contract-spec-defer-spelling-not-concept]]):

- **Concrete policy syntax** — the surface form of a policy module. Oracle-tag
  the tokens; the binding *semantics* (a policy supplies lattice +
  classifications + clearances + edges + ingestion points, §65 §2) are pinned.
- **The exact binding mechanism** — per-package / per-build / per-deployment.
  Pin that a policy **is bound** and enforced program-wide; defer *which* scope.
- **Policy composition** — org → team → service overrides, **monotone-tightening
  only** (a sub-policy may be stricter, never laxer). Pin the monotone-tightening
  *discipline* (it mirrors §63 step-6 policy compatibility); defer the compose
  operator's spelling.
- **The `Ward` runtime CT-validation emission** (§65 §2 CT requirements, → `63
  §5a` discharge attestation) — the runtime face is `Ward`-deferred
  (`OQ-discharge-attestation`). Pin the **static** `@ct` face (landed); oracle-tag
  the runtime emission as a **named trigger** (Ward runner), never a buried TODO.

## Acceptance criteria (discriminating pairs; producer-grep gated)

AC1–AC3 + AC5-static drive **landed** producers; AC4 is structural; AC6 is a
cross-ref contract face.

- **AC1 — ★ binding guarantee: relabel/over-clearance FAILS to compile
  (LANDED).** A value carrying a policy-`Secret` label written to a sink cleared
  only for `Public` → the flow is **rejected** (`l_sink`/`flows_to` returns a
  violation → compile error); the conformant case (label ⊑ clearance) → accepts.
  Non-degenerate pair on the real lattice check. Producer:
  `FlowCtx::l_sink(value_label, clearance, site)` (ifc.rs:239) / `flows_to`
  (ifc.rs:77) — grep the real IFC producer, **not** a hand-rolled bool. (This is
  the §65 §7 acceptance "relabels policy-Secret as Public … fails to compile.")

- **AC2 — ★ non-weakenable: the label's authority is the policy, not the
  implementer.** The security-critical face of §3. The enforcement is the **same
  landed `flows_to`/`l_sink`** as AC1; what Sec5 adds is that the governing label
  is **policy-sourced**, so an implementer's attempt to *re-declare* it laxer
  does not change the flow verdict. **Defer-spelling note:** the policy-sourced
  *binding* mechanism is `OQ-policy` (deferred) — so the case drives the landed
  enforcement with a label standing in for a policy-sourced one, **oracle-tagged**
  "`[policy-binding deferred: OQ-policy]` — the label is policy-authored, not
  inline; the *enforcement path is identical* because policy is a second input to
  the one engine (§4)." Pin that the verdict is the kernel/IFC check, and that an
  implementer inline-relabel cannot override a policy label once the binding
  lands. Do **not** fabricate a policy-parser producer.

- **AC3 — declassification requires the policy-granted capability (LANDED).** A
  downgrade (`Secret → Public`) **without** the declassification capability
  (§62) / outside a policy-permitted edge → rejected; **with** the capability +
  under its proven precondition (`requires`, §61 §4) → admitted. Discriminating
  pair on the real capability/declassify machinery (Sec1/Sec2 landed). Producer:
  the landed declassify + capability check, **not** a stub gate.

- **AC4 — ★ no new metatheory / no new kernel (structural, soundness anchor).**
  The policy layer introduces **no new kernel feature** and **no new
  metatheorem**: the same lattice-parametric by-typing non-interference (§61 §5)
  holds for the policy's `ℒ`. Mechanically: adding a policy exercises the
  **existing** IFC lattice ops (`join`/`meet`/`flows_to`) over a
  policy-supplied bounded lattice — **no** new kernel emission, **no** new
  trusted primitive. Discriminating: a policy over a valid bounded `ℒ`
  type-checks via the *unchanged* IFC path; the metatheory delta is **empty**.
  State as a structural/by-construction invariant (the Sec4 AC4 analog) — policy
  is `data + binding`. Correctly `[structural/by-construction]`, **NOT**
  "kernel-backed" (the [[trust-level-prose-vs-locked-adr]] discipline — the
  by-typing non-interference flow rule is a *trusted* meta-theorem, so this
  projects to trusted, never kernel-certified `Q`;
  [[trusted-by-typing-guarantee-is-not-kernel-proved-Q]]).

- **AC5 — CT requirement binds a data class (static face LANDED, runtime face
  Ward-deferred).** A policy "`KeyMaterial` is `@ct`" compiles to the **static**
  `@ct` leakage-sink discipline (Sec1ct, landed — a `@ct` value steering a
  branch/index/var-time op is a compile error): discriminating pair on the real
  `@ct` sink check (taint-value rejects while safe-value accepts — mind the
  order-dual orientation, [[taint-axis-orientation-needs-distinguishing-pair]]).
  The **runtime** face (Ward CT-validation into the `63 §5a` discharge
  attestation) is a **named deferred trigger** (Ward runner), oracle-tagged — not
  delivered here. State precisely which face is landed
  ([[soundness-AC-static-vs-runtime-face]]).

- **AC6 — governance rides supply-chain (cross-ref contract face).** The
  governing policy hash/version + exercised declassification authority appear in
  the package `trusted_base_delta` (landed `trusted_base()` surface) + provenance
  (§63, deferred). Doc/contract-posture fidelity: pin that policy governance is
  the §63 attestation's payload (not a Sec5-local mechanism); cross-link, don't
  re-specify §63. The `trusted_base_delta` face reuses the Sec4 landed
  enumeration.

## Trust faces — what this WP delivers

- **Delivered (landed producers):** the binding guarantee (AC1), the
  policy-authority-not-implementer enforcement path (AC2, enforcement landed /
  binding deferred), declassification-capability gating (AC3), the static `@ct`
  face (AC5-static). Every landed AC bottoms out in the **existing** IFC /
  capability producers — **no new kernel feature, no new metatheory** (AC4 is
  itself the proof of that). Policy = data + binding over the landed engine.
- **Deferred (oracle-tagged, named triggers):** the `OQ-policy` concrete syntax +
  binding mechanism + composition operator; the Ward CT runtime emission
  (`OQ-discharge-attestation`); the §63 provenance transport. None fabricated —
  each pinned in semantics, deferred in spelling, with a named resolver.

## Producer-grep gate (HIGH-signal)

The landed ACs must drive **real** producers:
- AC1/AC2 → `flows_to` (ifc.rs:77) / `FlowCtx::l_sink` (ifc.rs:239) over a
  bounded lattice. **Not** a hand-rolled bool, **not** a fabricated policy parser.
- AC3 → the landed declassify + capability check (Sec1 §61 §4 / Sec2 §62).
- AC5-static → the landed `@ct` leakage-sink check (Sec1ct).
- AC4 → the **absence** of new kernel emission for policy (structural).

Grep the producer, not the test. A case that hand-feeds a policy label or a
verdict is green-vs-green.

## Process (`§2c` / `§14`)

1. spec-leader routes to spec-author (`/spec §65` hardening) + CV
   (`/conformance/security/policy/`), independence preserved.
2. Merge Decision, three gates: **Architect — soundness** (the no-new-metatheory/
   no-new-kernel anchor AC4 + the binding-guarantee enforcement + the trust-level
   stamps: AC4 `[structural]`, AC5 static-vs-runtime split, nothing over-claimed
   "kernel-backed"); **CV — Spec** on `/spec §65`; **spec-author — Fidelity** on
   the `/conformance` seed.
3. Integrator merges on green (spec + conformance + docs only, no crates).
4. Retros in → Steward. With Sec5, the enclave's G5 security spine is Sec1/Sec2/
   Sec4/Sec5 (+ Sec3 supply-chain, which carries the 33↔63 package-format locus —
   sequence with L4b / operator input, flagged separately).

## References (verify targets, don't launder)

- `spec/60-security/65-policy.md` (the chapter; §1–§7); `docs/adr/0007-*`
  (locked design); `spec/90-open-decisions.md` (`OQ-policy`).
- `spec/60-security/61-information-flow.md §2` (lattice-parametric IFC), `§5`
  (by-typing non-interference), `§4` (declassification), `§5a` (`@ct`).
- `crates/ken-elaborator/src/ifc.rs` — `flows_to` (:77), `l_sink` (:239),
  `join`/`meet` (:55/:64), `classify_vis_op` (:117) — the landed enforcement.
- `spec/60-security/63-supply-chain.md §5` (governance attestation, cross-ref);
  `spec/20-verification/25 §3` (`trusted_base_delta`).
