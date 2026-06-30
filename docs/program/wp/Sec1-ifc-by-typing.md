# WP Sec1 — Information-flow control by typing (opens WS-Sec, tier-1)

> **Status:** Steward frame — **candidate post-spine WP** (deps met: L5 on `main`).
> **Opens WS-Sec** (security, a tier-1 workstream). spec-leader elaborates
> `spec/60-security/61-information-flow.md` (DRAFT → implementation-ready); build
> team TBD (Verify or a security enclave — see Sequencing).
>
> **Deps:** **L5** (`36`/effects: the **interaction-tree** denotation — IFC labels
> ride ITree nodes) + the kernel (the de Bruijn re-check). **Size:** L · **Risk:**
> **tier-1 security** — the guarantee *and its honest limits* are load-bearing
> (`64`: "a verified language that over-claims is itself a security risk"). ► First
> WP of WS-Sec → **G-Sec**; the IFC half of the security story (authority is Sec2).

## Objective

Elaborate `61-information-flow` (Sec1) — **lattice-parametric non-interference by
typing**: information-flow labels on **interaction-tree nodes** (riding L5's
denotation), with non-interference enforced **through the type system** and
**re-checked by the kernel** (the de Bruijn criterion — IFC is not a trusted
add-on). The settled design (do **not** reopen):
- **`OQ-ifc` DECIDED** — **lattice-parametric + DLM** (decentralized label model);
  a standard DLM lattice is the default instance.
- **`OQ-relational` DECIDED + narrowed** — non-interference is **by-proof =
  re-checked product programs**, **progress-sensitive**; the heavy relational
  machinery is **deferred** (honest-boundary it, don't fake it).
- **`@ct` constant-time** is an **opt-in label** (IFC to leakage sinks) — that's
  **Sec1ct**, a *separate* follow-on WP; Sec1 lands the label + the hook, not the
  full timing enforcement.

## The framing that sets the risk level

Tier-1, but the trust story is the subtle part: IFC **by typing** means the label
discipline is enforced where the type system already runs and **the kernel
re-checks the result** — so a Sec1 bug is a **missed or over-claimed
non-interference guarantee**, and *over-claiming is itself the security failure*
(`64`'s honest-limits mandate). Load-bearing: (1) the **non-interference theorem
is stated with its exact preconditions** (progress-sensitive, the product-program
re-check, what is proven vs. assumed) — no over-claim; (2) labels **compose
correctly through the effect/ITree structure** (a label can't be laundered by
routing a value through an effect); (3) the discipline is **kernel-re-checkable**,
not trusted.

## Scope

**IN:** the **label lattice** (lattice-parametric, DLM default) + label syntax on
types/effects; **labels on interaction-tree nodes** (how an effect's label flows
through L5's denotation); the **by-typing non-interference** discipline + the
**product-program re-check** mechanism (OQ-relational: progress-sensitive, the
re-checked relational obligation); the **`@ct` label *hook*** (the opt-in marker;
enforcement is Sec1ct); the **honest-limits** statement (proven vs. assumed,
deferred machinery named).

**OUT — other WPs:** **authority/capabilities** (`62`, Sec2); **`@ct` timing
enforcement** (Sec1ct); supply-chain (`63`), policy-as-code (`65`, Sec5); the
deferred heavy relational machinery (named, not built).

## The elaboration this needs (spec-leader → spec-author + Architect)

Elaborate `61` to builder rigor: the label lattice + DLM instance; label
placement on types/effects/ITree nodes; the non-interference statement **with
exact preconditions** and the product-program re-check as defensive pseudocode;
the `@ct` hook. **Ground against the *landed* L5 (the ITree denotation + `36`
effect labels) + the kernel re-check API — the files, not status.** **The
honest-limits section (`64`) is load-bearing — state proven-vs-assumed exactly;
do not over-claim** (the security analog of the metatheory-honesty discipline).
Conformance (`conformance/security/ifc/`): a leaking program is **rejected**
(a high-label value reaching a low sink — verdict-flip); a non-interfering program
**accepts**; a label laundered through an effect is **caught** (the ITree-node
label composition); the product-program re-check accepts a related pair and
**rejects** a distinguishable one; the deferred cases carry a **reify-trigger
placeholder** (not silent).

## Acceptance (testable)

1. **Non-interference by typing:** a high→low leak is **rejected**; a clean
   program accepts; the theorem's **preconditions are stated** (progress-sensitive,
   re-checked product program) — no over-claim.
2. **Labels compose through effects:** a value routed through an effect **keeps
   its label** (no laundering via the ITree); discriminating case.
3. **Kernel-re-checkable:** the IFC obligation is re-checked, not trusted (a
   forged label/proof is kernel-rejected).
4. **`@ct` hook present, enforcement deferred:** the opt-in label parses + is
   carried, with a reify-trigger placeholder to **Sec1ct** (not a silent gap).
5. **Honest limits:** the proven-vs-assumed boundary is explicit; deferred
   machinery is named with its trigger.

## Sequencing

**Opens WS-Sec** (tier-1). Deps L5 (done). Per `05`-DAG "keep security in-band —
sequence Sec1/Sec2 *with* the surface, not after." Unblocks **Sec1ct** (`@ct`),
**Sec5** (policy), **B4** (agentic boundary). **Build team:** the verification
enclave's discipline fits (re-checked obligations), or a dedicated security build
team if the operator stands one up. Build queries: security semantics → Spec;
trust-model/TCB → Architect. Clean-room: landed L5 + `60-security/` + first
principles; **copyleft security refs (jif, etc.) are Spec-enclave-only**, never
vendored (`CLEAN-ROOM.md`). **(Post-spine broadening — the operator's checkpoint:
Sec1 is the Steward's recommended highest-value ready WP; alternatives are B1
export / X3 scale / language breadth.)**
