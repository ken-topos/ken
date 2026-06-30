# WP Sec1-build — IFC by typing, the implementation (WS-Sec, tier-1)

> **Status:** Steward frame — **release to Team Verify** (operator-decided:
> WS-Sec build is a scope extension of Verify, not a new team). Builds directly
> from the **landed** spec + conformance; no further spec-leader elaboration
> round (the enclave already elaborated `61` impl-ready + the 18-case seed).
>
> **Deps (all on `main`):** **L5** (`36` ITree denotation — labels ride `Vis`
> nodes), **K1.5** (kernel admission), **V3** (`23` prover + kernel re-check —
> the by-proof path), **Sec1 spec** (`61` @ `a5c82ea`, incl. the N1/N2 honesty
> fold) + the **`conformance/security/ifc/` seed** (18 cases). **Size:** L ·
> **Risk: tier-1 security, TWO trusted surfaces** (see Trust boundary). ► First
> implementation of WS-Sec → **G-Sec**.

## Objective

Implement IFC-by-typing in the elaborator per the landed `61`: the label
lattice + DLM instance, the flow-typing pass (the four `§3` rules), the
no-laundering guarantee on `Vis` nodes, the `@ct` hook (precondition only), and
the **basic by-proof relational path** (product-program → V3 kernel-re-checked
verdict) for the non-value-dependent cases. Make the `ifc/` seed pass. The
settled design is locked — **do not reopen** `OQ-ifc` (lattice-parametric + DLM)
or `OQ-relational` (by-proof = re-checked product programs, progress-sensitive;
heavy machinery deferred).

## The trust boundary — read this first (it sets the whole WP's risk)

Sec1 has **two trusted surfaces** the kernel does **not** backstop. This is the
security analog of the V2/V3 two-soundnesses carry (the kernel re-checks what a
layer *supplies*, never what it *omits*). Both are stated normatively in
`61 §9`/`§H` (the N1/N2 fold) and pinned by discriminating conformance:

- **N1 — the by-typing flow rules are TRUSTED.** IFC labels are **erased**
  before the kernel (`§3`: "at the kernel it *is* `A`"). So a flow-typing bug
  (wrong `⊑` in `L-SINK`, a dropped `pc`-join, a label-dropping `bind`/`incl`)
  emits a **well-typed core term the kernel accepts** while non-interference is
  violated. The kernel is **blind** to it. The **sole net** is the §H
  meta-theorem + the discriminating flip cases **{A1–A4, C1, F1}** — never the
  kernel. Treat the flow pass with trust-root discipline.
- **N2 — the by-proof product-program reduction is TRUSTED.** The kernel
  re-checks the certificate **for the obligation it is handed**, not that the
  obligation *faithfully encodes 2-safety*. A wrong reduction (too-weak
  `Φ_post`, a dropped `coterminates_ζ`) yields a **kernel-valid cert for a
  non-NI claim** — a false `proved` the forged-cert reject (E1) does **not**
  cover. `cert-recheck ≠ reduction-faithfulness`. The **sole net** is the
  positive-soundness case **D5** (a known-interfering program must reduce to
  `disproved`).

**Over-claiming is itself the security failure** (`64 §4`). State proven vs.
delegated exactly; never assert a delegated guarantee.

## Scope

**IN:**
- **§2 lattice** — the `Lattice` record interface (carrier + `⊑`/`⊔`/`⊓`/`⊥`/`⊤`
  + laws-as-Ω-obligations) and the **DLM instance** (confidentiality =
  reader-sets by reverse inclusion, `⊔ = ∩`; integrity = the order-dual,
  `Trusted = ⊥ ⊑ Untrusted = ⊤`; products componentwise; levels as sugar).
  Lattice-parametric; the concrete instance is policy (`65`).
- **§3 flow-typing pass** — the erasable index `A @ ℓ`; the explicit `pc`-label
  (implicit flows); the four rules **`L-PURE`/`L-COMBINE`/`L-OBSERVE`/`L-SINK`**
  (`L-SINK` joins `pc`: `(ℓ ⊔ pc) ⊑ κ`). Labels **erased** before the kernel.
- **§3.2 no-laundering** — the label rides the `Vis` op/resp (`36 §3.1`);
  `bind`/`incl` reconstruct the **same** `Vis e` node (`36 §2.2/§2.4`) and must
  **preserve** the index. The exact bug C1 flips on: a label-dropping
  `bind`/`incl`/handler at the `Vis` boundary.
- **§5.3 by-proof, basic path** — the product-program construction + the V3
  kernel-re-checked obligation, for the non-value-dependent cases **D1/D2/D5**:
  related → `proved`; distinguishing → `disproved`-with-witness; interfering →
  `disproved` (D5, the N2 reduction-faithfulness backstop). Verdict mapping
  pinned at source (never a false `proved`; unprovable → `incomplete`-hole).
  Progress-sensitive default (the `coterminates_ζ` conjunct).
- **§5a `@ct` HOOK only** — the opt-in label **parses, is carried, and a `@ct`
  value reaching a leakage-relevant sink (branch/index/var-time) is a type
  error** (the source precondition `Q`). Carry the reify-trigger.
- **§H/§9** — honest-limits behavior surfaced: the four-way status shows the
  termination-(in)sensitivity choice; no kernel former, no new level rule; the
  `ℓ_carrier ≤ ℓ_ITree` side-condition on the parametric `Lattice`.

**OUT — deferred, carry the reify-trigger, NOT this WP:**
- **`@ct` timing enforcement** → `[Sec1ct]`; **runtime timing validation** →
  `[Ward]`.
- **Heavy value-dependent product-program machinery** (seed D3/D4) →
  `[rel-deferred]`. Land the basic mechanism (D1/D2/D5); the value-dependent
  relational cases stay deferred with their existing trigger.
- **authority/capabilities** (`62`, Sec2); **policy-as-code** (`65`);
  **supply-chain** (`63`).

## Acceptance (testable — the seed is the contract)

The `conformance/security/ifc/seed-ifc.md` 18 cases, grounded on landed `61`:
1. **By-typing flips (A1–A4, B1–B3, C1, F1)** — accept/reject; each goes
   green-vs-red under its named bug. **C1 is doubly load-bearing** (N1: kernel
   is blind). NI is **never** authored as a unary `ensures`.
2. **By-proof verdict mapping (D1, D2, D5)** — related → `proved`,
   distinguishing → `disproved`-with-witness, interfering → `disproved` (D5).
   The trichotomy class agrees (a non-interfering program is never `disproved`;
   unprovable → `incomplete`, never false `proved`).
3. **Kernel re-check (E1)** — a forged label/cert is kernel-rejected.
4. **`@ct` hook (F1, F2)** — parses/carries/rejects-at-sink; timing
   `(oracle)`/defer-tagged, reify-trigger present.
5. **Honest limits (G1, G2)** — proven/assumed/delegated/deferred exact;
   deferred machinery (D3/D4) carries the `[rel-deferred]` trigger, not silent.
6. **No-regression** — the spine + kernel suites stay green; **no kernel
   enlargement** (labels are `Vis` indices).

## Disciplines (the spine carries transfer)

- **Exhaustive-by-construction** (COORDINATION §7): the lattice-op and
  flow-rule dispatch is a single no-`_=>` match; a new lattice/rule case is a
  compile error.
- **Trust-root testing on BOTH trusted surfaces** — for N1, verify the flip
  cases {A1–A4, C1, F1} genuinely flip under the exact flow bug (not
  green-vs-green); for N2, verify D5 (interfering → `disproved`) and that the
  reduction can't be massaged to make a leak look `proved`. Test at non-empty Γ
  and non-degenerate labels (not all-`⊥`/all-`⊤`).
- **Assert the emitted output** — for the flow pass, assert the accept/reject +
  the erased core shape; for by-proof, assert the verdict + the witness/hole,
  not just "it ran."
- **Placeholder lifecycle** — every deferred case names its reify-trigger
  (`[Sec1ct]`/`[Ward]`/`[rel-deferred]`), never a silent vacuous test.
- **Ground against landed `61`/`36`/`23`/`18`, the files, not status.**

## Sequencing

**Team Verify** (`verify-leader` `agt_37reqqf16g800` → `verify-implementer`
`agt_37reqfz3jnw00` → `verify-qa`). Deps L5+K1.5+V3+the Sec1 spec/seed — **all
on `main`**, no blocker. Carries straight from T1-build (same crate,
`ken-elaborator`; no team gap). **Unblocks** Sec1ct (`@ct` timing), the heavy
relational machinery, and B4 (agentic boundary). Build queries: security
semantics → Spec; trust-model/TCB → Architect. **Clean-room:** landed
`61`/`36`/`23` + first principles; copyleft security refs (jif, DCC, FaCT) are
**Spec-enclave-only, never vendored, never consulted by the implementer**
(`CLEAN-ROOM.md`). **Mechanism:** rebase → cut `wp/Sec1-build` off `origin/main`
→ assign implementer → ring → QA (verify BOTH trusted surfaces' flip cases) →
diff-scope (crates-only ⇒ Architect + CI, the trust model is load-bearing so
Architect is required regardless) → `propose_decision` → standalone Integrator
`git_request` after the vote → retros.
