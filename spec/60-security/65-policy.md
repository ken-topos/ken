# Security policy — authored by compliance, enforced by the compiler

> Status: **Normative** for the policy *shape* and the *binding guarantee* (what
> ADR 0007 commits). The concrete policy **syntax** is `OQ-policy` (open): this
> chapter pins the **semantics** and **defers the spelling** (§6) — it does not
> resolve `OQ-policy`. The **separately-authored, mandatory, static**
> security-policy surface (ADR 0007) instantiates the lattice-parametric
> information-flow discipline (`61-information-flow.md`) and is the thing a CISO
> authors: **policy as code, enforced by the type-checker, separated from — and
> binding on — the implementation.** Conformance:
> `../../conformance/security/policy/`.

Each contract below is stated with its **trust level**, matching the trust-model
discipline (`64`): a **`[landed producer]`** contract bottoms out in a real
elaborator function whose accept/reject a consumer observes; a
**`[structural/by-construction]`** contract is a mechanically-auditable
invariant of the existing discipline, not a new proof; a
**`[deferred: <resolver>]`** surface pins the *semantics* and oracle-tags the
*spelling*, naming its resolver. **The load-bearing honesty (§4):** the policy
layer costs no new metatheory *because* it is the configuration the
lattice-parametric non-interference meta-theorem (`61 §5`) already covers — but
that theorem is **trusted by typing** (labels are erased before the kernel,
`61 §5`/§H), so a policy's guarantee rides `P`/`tested`, **never**
kernel-certified `Q`. Policy is cheap on soundness, not kernel-backed.

## 1. Why a policy layer (role separation, not a new engine)

Classification has **two authors with different authority**. **Security /
compliance** define *what the policy is* — what counts as PII/secret, the
principal/compartment lattice, which sinks are cleared for what, which
declassifications are permitted under what proven condition, which ingestion
points may classify. **Implementation teams** write code that must *conform*.
Putting all of that in inline labels (`61 §3`) puts classification authority in
the wrong hands: an implementer must not be able to declare PII `Public`, and
must not have to re-derive the org lattice.

So Ken has a **policy layer**. It is **part of Ken, not a sibling** (contrast
Ward, `../70-behavioral/`): policy is **static**, enforced by **Ken's own
type-checker**, at the same time as everything else — it is **not a second
engine, it is a second *input* to the one engine**. What is separate is the
**authoring role**, mirroring the agent-team federation: security/compliance own
the policy as the team constraint, implementers are bound by it. This
one-engine-two-inputs framing is the reason the layer costs no metatheory (§4)
and the reason enforcement is the *same* landed IFC check whatever the label's
author (§3).

## 2. What a policy declares

A **policy module** (authored by security/compliance) supplies the concrete
instantiation of the IFC discipline (`61 §2`, lattice-parametric):

- **The lattice `ℒ`** — principals, compartments, data-residency regions, and
  any level sugar (the DLM standard, `61 §2`). It must be a **bounded lattice**
  (`61 §2.1`: `⊥`, `⊤`, `join`, `meet`) — the interface the generic proof
  requires; a policy supplying a non-lattice is rejected at policy admission.
- **Classification rules** — which types/data carry which labels ("`Email` is
  `String @ Secret[user]`"; "rows of table `customers` are PII"); structural and
  source-based.
- **Channel/sink clearances** — what each `Net`/`FS`/RPC/log/`space` sink (and
  each per-compartment sink, e.g. `bucket[Tenant X]`) is cleared for (`61 §3`).
- **Declassification edges** — which downgrades are permitted, **by whom** (the
  declassification capability, `62`), under what **proven precondition** (a
  `requires`, `61 §4`).
- **Trusted classification points** — which ingestion boundaries may assign
  **data-derived** labels (the audited dual of declassification, `61 §3`).
- **Constant-time requirements** — which data classes **must be handled
  constant-time** (e.g. "`KeyMaterial` is `@ct`", `61 §5a`). This has **two
  faces** (§65 AC5), which a policy spec must not conflate:
  - **Static face `[landed producer]`:** the requirement compiles to the static
    `@ct` leakage-sink discipline (`61 §5a`, landed, `l_ct_sink`) — a `@ct`
    value steering a branch / index / variable-time op is a **compile error**.
    Mind the order-dual `@ct` orientation (`61 §5a.1`): the discriminating pair
    is a **taint-carrying value rejected while a safe value accepts** on the
    same sink.
  - **Runtime face `[deferred: Ward / OQ-discharge-attestation]`:** the `Ward`
    runtime CT-validation, emitted into the `63 §5a` discharge attestation, is a
    **named deferred trigger** (the Ward runner) — *not* delivered by this WP.
    The static face is a necessary precondition; the timing guarantee itself is
    hardware/codegen-relative and delegated below Ken (`64 §4.2`, `61 §5a.6`).

## 3. Mandatory and non-weakenable (the load-bearing property)

A policy is **bound** to a build/package and **imposed program-wide as a
constraint the implementer cannot weaken** (ADR 0007 Decision 3). The three
binding contracts below all bottom out in the **same landed IFC enforcement** as
inline labels — policy adds *authority over the label*, not a new checker.

> **Contract POL-Bind (AC1) `[landed producer]`.** A value carrying a
> policy-`Secret` label written to a sink cleared only for `Public` **fails to
> type-check**: the flow-typing pass returns a violation (`FlowCtx::l_sink`
> rejects because `flows_to(value_label, clearance)` is false, `61 §3.1`) — a
> **compile error, not a review nit**. The conformant case (label ⊑ clearance)
> **accepts**. Non-degenerate pair on the real lattice check; producer:
> `l_sink`/`flows_to`, never a hand-rolled bool.

> **Contract POL-NonWeaken (AC2) `[landed producer]` enforcement +
> `[deferred: OQ-policy]` binding.** The label's **authority is the policy, not
> the implementer**. Because enforcement is a *second input to the one engine*
> (§1), the flow verdict is computed by the **identical** `flows_to`/`l_sink`
> path regardless of who authored the label — so an implementer who re-declares
> a policy-`Secret` value `Public` inline **cannot change the verdict**: the
> laxer inline label is either overridden by the bound policy label or is itself
> a policy violation. **Non-weakenability is therefore a property of the
> *landed* enforcement**, not of the deferred binding. The mechanism that
> *sources* the label from a policy module (rather than inline) — the concrete
> binding — is `OQ-policy`-deferred; pin that a policy **is bound** and enforced
> program-wide, defer *which scope* (per package/build/deployment) and the
> syntax. Do **not** fabricate a policy-parser producer; a conformance case
> stands in a policy-authored label for the enforcement check, oracle-tagged
> `[policy-binding deferred: OQ-policy]`.

> **Contract POL-Declassify (AC3) `[landed producer]`.** A downgrade
> (`Secret → Public`) **without** the policy-granted declassification capability
> (`62`), or outside a policy-permitted edge, is **rejected**
> (`check_declassify` returns `Reject`: missing capability or edge mismatch);
> **with** the capability **and** under its proven precondition (`requires`,
> `61 §4`) it is **admitted**. Declassification is the *only* downgrade,
> explicit and audited (`61 §4`), and the exercised authority is recorded (§5).
> Non-degenerate pair on the landed declassify + capability machinery, never a
> stub gate.

Policy is **orthogonal** to implementation: one policy governs many modules,
authored once and applied everywhere it is bound. Implementers write ordinary
code; the policy is the cross-cutting constraint they cannot escape. This is
what a CISO wants and a scanner cannot give: classification authority held by
security, enforcement by the compiler, no bypass.

## 4. Policy is the *instantiation*, so it costs no metatheory

Because the IFC discipline is **lattice-parametric** (`61 §2`, non-interference
proved once for any bounded `ℒ`), a policy is precisely the **configuration the
generic proof was waiting for** — the lattice + classifications + clearances +
edges. Adding the policy layer therefore **adds nothing to the metatheory and
nothing to the kernel**.

> **Contract POL-NoCost (AC4) `[structural/by-construction]`.** A policy
> introduces **no new kernel feature and no new metatheorem**: the same
> by-typing non-interference theorem (`61 §5`) holds for whatever bounded `ℒ`
> the policy supplies, exercising the **existing** IFC lattice ops
> (`join`/`meet`/ `flows_to`) — **no** new kernel emission, **no** new trusted
> primitive. The **metatheory delta is empty**; the discriminating check is that
> a policy over a valid bounded `ℒ` type-checks via the *unchanged* IFC path.
> This is a **structural / by-construction** invariant (the `64 §3.1` analog) —
> policy is `data + binding`, not new logic (reflect-don't-extend /
> subsume-don't-proliferate).

**The trust level this inherits — state it, do not launder it.** "Costs no
metatheory" is a claim about *cost*, not about *strength*. The theorem the
policy rides — well-labeled ⇒ non-interfering (`61 §5`) — is a **trusted
meta-theorem about the discipline**: IFC labels are **erased before the kernel**
(`61` banner, `§5`/§H), so the kernel never re-checks the label flow. A policy's
non-interference guarantee is therefore **trusted-by-typing**, and it projects
to `P`/`tested` — **never** kernel-certified `Q`
([[trusted-by-typing-guarantee-is-not-kernel-proved-Q]]). The **discriminating
conformance corpus, not the kernel, is the net** for it (`61 §5a.6`/§H). So the
correct reading of AC4 is: policy is **cheap on soundness** (empty metatheory
delta) *and* its guarantee is **trusted, not kernel-backed** — both, honestly,
and a policy spec that presents "no metatheory cost" as "kernel-guaranteed"
over-claims exactly where a CISO reads the assurance.

## 5. Org-scale governance → the supply-chain story

The **mechanism** is in Ken (this chapter). **Governance at organisational
scale** — versioning policies, distributing them across repos, and *attesting*
that a build provably conformed to policy `vX` — is the **supply-chain** concern
(`63-supply-chain.md`).

> **Contract POL-Governance (AC6) — cross-ref contract face.** The governing
> policy's **hash/version** and any **declassification authority exercised**
> appear in the package's **`trusted_base_delta`** (the landed enumeration
> surface, `64 §1`/`25 §3`) and its **provenance** (`63`, transport deferred).
> Policy (what is allowed) + per-package attestation (what was used) together
> give **"this build provably honoured org policy `vX`."** This chapter pins
> that policy governance is the **payload** of the `63` attestation — it
> **cross-links, it does not re-specify** the `63` attestation shape (that is
> Sec3's locus). The `trusted_base_delta` face reuses the `64` landed
> enumeration; the provenance transport is `63`-deferred (`OQ-provenance`).

## 6. What is committed vs open

- **Committed (ADR 0007), normative:** a separately-authored, mandatory, static
  policy surface, **in Ken** (not a sibling); it instantiates the
  lattice-parametric discipline; implementers are **bound and cannot weaken** it
  (§3); org-scale governance rides the supply-chain attestation (§5).
  `OQ-policy` itself is **decided** (ADR 0007) — what remains open are
  sub-decisions *within* the committed design.
- **Open (`OQ-policy` sub-decisions, `../90-open-decisions.md`) — deferred by
  spelling, pinned in semantics, each with a named resolver:**
  - **Concrete policy syntax** — the surface form of a policy module. Semantics
    pinned (§2: a policy supplies lattice + classifications + clearances + edges
    + ingestion points); tokens oracle-tagged. Resolver: `OQ-policy`.
  - **The binding mechanism** — per package / per build / per deployment.
    Pinned: a policy **is bound** and enforced program-wide (§3); *which scope*
    deferred. Resolver: `OQ-policy`.
  - **Policy composition** — org → team → service overrides,
    **monotone-tightening only** (a sub-policy may be **stricter, never laxer**
    — it mirrors the `63` step-6 policy-compatibility discipline). The
    monotone-tightening *discipline* is pinned (it is the stability guarantee a
    client relies on); the compose operator's *spelling* is deferred. Resolver:
    `OQ-policy`.
  - **The `Ward` runtime CT-validation emission** (§2 CT requirement, →
    `63 §5a`) — the runtime face is `Ward`-deferred. The **static** `@ct` face
    is landed (§2); the runtime emission is a **named trigger** (the Ward
    runner), never a buried TODO. Resolver: `OQ-discharge-attestation`.
  - **The `63` provenance transport** for the governing policy hash/version
    (§5). Resolver: `63` / `OQ-provenance`.

## 7. What WS-V / WS-L must deliver here

- **Delivered (landed producers):** the binding guarantee — relabel /
  over-clearance fails to compile (POL-Bind/AC1, `l_sink`/`flows_to`); the
  policy-authority-not-implementer enforcement path (POL-NonWeaken/AC2,
  enforcement landed / binding `OQ-policy`-deferred);
  declassification-capability gating (POL-Declassify/AC3, `check_declassify`);
  the **static** `@ct` face (AC5-static, `l_ct_sink`). Every landed contract
  bottoms out in the **existing** IFC / capability producers — **no new kernel
  feature, no new metatheory** (POL-NoCost/AC4 is itself the statement of that),
  and every guarantee inherits `61`'s **trusted-by-typing** level, not `Q` (§4).
- **Deferred (oracle-tagged, named triggers):** the `OQ-policy` concrete syntax
  + binding mechanism + composition operator; the `Ward` CT runtime emission
  (`OQ-discharge-attestation`); the `63` provenance transport. None fabricated —
  each pinned in semantics, deferred in spelling, with a named resolver (§6).

Acceptance: an implementation that relabels policy-`Secret` data as `Public`, or
writes above a policy clearance, or declassifies without the policy capability,
**fails to compile** (POL-Bind/POL-NonWeaken/POL-Declassify); a policy over a
valid bounded `ℒ` type-checks through the unchanged IFC path with an **empty
metatheory delta** (POL-NoCost); a conformant build records its governing policy
in `trusted_base_delta` + provenance (POL-Governance). The security-critical
contracts drive the **real** landed IFC producers; the policy guarantee is
**trusted-by-typing**, cheap on soundness and honestly not kernel-backed.

