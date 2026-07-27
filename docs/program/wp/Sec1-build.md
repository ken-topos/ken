# WP Sec1-build — IFC by typing, the implementation (WS-Sec, tier-1)

> **Status:** Steward frame — **release to Team Verify** (operator-decided:
> WS-Sec build is a scope extension of Verify, not a new team). Builds directly
> from the **landed** spec + conformance; no further spec-leader elaboration
> round (the enclave already elaborated `61` impl-ready + the seed).
>
> **Deps:** **L5** (`36` ITree denotation — labels ride `Vis` nodes, **built**:
> `ken-elaborator/src/effects/{lower,extract}.rs`, `capabilities.rs`),
> **K1.5** (kernel admission, `f037451`, **ancestor of `origin/main`**),
> **Sec1 spec** (`61`, blob **`e6c91f50`**, incl. the N1/N2 honesty fold) + the
> **`conformance/security/ifc/seed-ifc.md`** seed (blob **`45160418`**, **16**
> Sec1 cases). ⚠ **`V3` (`23` prover) is a dep of increment 2 ONLY and is NOT
> re-verified in this frame** — see §Slicing. **Size:** L, released as **two
> increments** · **Risk: tier-1 security, TWO trusted surfaces** (see Trust
> boundary). ► First implementation of WS-Sec → **G-Sec**.
>
> ⛔ **Re-derive every blob at point of use.** The pins above were measured at
> `origin/main = 4be827eb`; `61` has already moved **once** under this frame
> (see §The `@ct` boundary) and a stale pin is what caused that defect.

## Objective

Implement IFC-by-typing in the elaborator per the landed `61`: the label
lattice + DLM instance, the flow-typing pass (the four `§3` rules), the
no-laundering guarantee on `Vis` nodes, and
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
  meta-theorem + the discriminating flip cases **{A1–A4, C1}** — never the
  kernel. Treat the flow pass with trust-root discipline.
  ⚠ **`F1` was in this net and is no longer available to it** — the seed moved
  F1 to Sec1ct (`CT-A1/A2/A3` + `CT-A4`). ⛔ Do **not** substitute a Sec1ct case
  into a Sec1 trust-root net: the net has **five** members now, not six, and a
  net that cites a case living in another WP's seed is unrunnable here.
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
- **§H/§9** — honest-limits behavior surfaced: the four-way status shows the
  termination-(in)sensitivity choice; no kernel former, no new level rule; the
  `ℓ_carrier ≤ ℓ_ITree` side-condition on the parametric `Lattice`.

**OUT — deferred, carry the reify-trigger, NOT this WP:**
- ⛔ **THE WHOLE `@ct` DISCIPLINE, INCLUDING THE LABEL AND THE SINK RULE** →
  `[Sec1ct]`. ⚠ This is a **change from this frame's earlier scope**, forced by
  a spec move — see §The `@ct` boundary below. ⛔ Not the label's parsing, not
  the `L-CT-SINK` rule, not the sealed `LeakSink` set, not the CT-promise/`P`
  export, not declassify-ends-span. **None of `§5a` is Sec1's.**
  **runtime timing validation** → `[Ward]`.
- **Heavy value-dependent product-program machinery** (seed D3/D4) →
  `[rel-deferred]`. Land the basic mechanism (D1/D2/D5); the value-dependent
  relational cases stay deferred with their existing trigger.
- **authority/capabilities** (`62`, Sec2); **policy-as-code** (`65`);
  **supply-chain** (`63`).

## ⛔ The `@ct` boundary — a spec section moved under this frame

⚠ **Read this before scoping anything `@ct`.** This frame previously took
"`§5a` `@ct` **hook** only" as IN scope. That is now wrong, and the reason is
worth stating because it is the failure mode, not just the fact:

| | |
|---|---|
| the frame pinned | `61` @ `a5c82ea` (blob `f2590c40`) |
| `61` is now | blob `e6c91f50` — **+304 / −67** |
| every `§2`–`§4` heading | **byte-identical** across that drift |
| the **one** heading that changed | `### 5a. Constant-time — the @ct` ***hook*** *(Sec1 lands the label, not the timing)* → `### 5a. Constant-time — the @ct` ***discipline*** *(by typing; timing delegated)* |

⇒ `§5a` is **no longer a hook**. It is the full elaborated Sec1ct discipline —
the `CT` axis, the sealed `LeakSink` set, the `L-CT-SINK` rule, the
CT-promise/`P` export, declassify-ends-span. The seed agrees and has already
moved: its group `F` is now a **pointer**, stating F1/F2 are *"superseded by the
Sec1ct `@ct` discipline"*, F1 splitting into `CT-A1/A2/A3` + `CT-A4` and F2
absorbed into `CT-E1`.

⭐ **The generalizable defect: this frame named a spec SECTION and let that
section's TITLE carry a WP BOUNDARY.** The title said "hook", so "hook only"
read as a scope limit. When the enclave re-elaborated the section, the title
changed and the boundary silently inverted — the pointer now resolves to the
*whole* discipline. ⛔ A section reference is an **address**, never a **limit**.
⇒ So the boundary is now stated in this frame's own words, in the OUT list,
naming the five mechanisms rather than deferring to `§5a`'s heading.

⚠ **What this would have cost if released unrepaired:** an implementer following
"`§5a` hook only" into the current `§5a` finds the complete discipline and can
reasonably build `L-CT-SINK` + the sealed sink set — duplicating Sec1ct, in a
WP whose own AC cites two seed cases (F1/F2) that no longer exist in its seed.

## Slicing — TWO increments, cut on the TRUST-SURFACE seam

⭐ **The cut is the N1/N2 boundary**, so each increment carries **exactly one
trusted surface** and its trust-root discipline has a single owner:

| increment | scope | trusted surface | seed cases | dep risk |
|---|---|---|---|---|
| **1 — by-typing** (M) | `§2` lattice + DLM · `§3` flow-typing (4 rules) · `§3.2` no-laundering · `§H`/`§9` honest limits · E1's **forged-label** half | **N1 only** | A1–A4, B1–B3, C1, G1, G2, E1(label) | ⭐ **none** — needs only L5 + K1.5, both verified on `main` |
| **2 — by-proof** (M) | `§5.3` product-program + V3 kernel-re-checked verdict mapping · E1's **forged-cert** half | **N2 only** | D1, D2, D5, E1(cert) | ⚠ **needs `V3`** |

⛔ **Release increment 1 only.** ⚠ **`V3`'s delivered state is NOT verified in
this frame.** The predecessor frame asserted "V3 … all on `main`" and I did not
re-derive it; a keyword probe is not evidence, and increment 1 is deliberately
built to not depend on the answer. ⇒ **Increment 2 is `not-ready` until someone
measures V3 against `23` and reports it.** ⛔ Do not infer V3's state from this
frame, from the DAG table, or from a tracker row.

⭐ **Why this cut and not a smaller one:** `§3` flow-typing without `§2`'s
lattice has no `⊑` to check, and `§3.2` no-laundering is the case (C1) that N1
is *most* blind to — splitting it out would ship the verifiable half of a
trusted surface and leave its load-bearing case for later.

## Acceptance (testable — the seed is the contract)

The `conformance/security/ifc/seed-ifc.md` **16** Sec1 cases, grounded on
landed `61`:
1. **By-typing flips (A1–A4, B1–B3, C1)** — accept/reject; each goes
   green-vs-red under its named bug. **C1 is doubly load-bearing** (N1: kernel
   is blind). NI is **never** authored as a unary `ensures`.
2. **By-proof verdict mapping (D1, D2, D5)** — related → `proved`,
   distinguishing → `disproved`-with-witness, interfering → `disproved` (D5).
   The trichotomy class agrees (a non-interfering program is never `disproved`;
   unprovable → `incomplete`, never false `proved`).
3. **Kernel re-check (E1)** — a forged label/cert is kernel-rejected.
4. ⛔ **NOT AN AC OF THIS WP.** Sec1's `AC4` (the `@ct` hook) was **elaborated
   into the Sec1ct discipline** and its cases F1/F2 now live in
   `conformance/security/ct/seed-ct.md` as `CT-A1/A2/A3`, `CT-A4`, `CT-E1`
   (Sec1ct's own `AC1`–`AC7` namespace). ⚠ **Different AC namespaces** — this
   row is Sec1 `AC4`; `../ct/` runs Sec1ct `AC1`–`AC7`. ⇒ **Discharge nothing
   here.** A green `@ct` result produced by this WP is out-of-scope work, not
   progress.
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
Architect is required regardless) → `propose_decision` → standalone Steward
`git_request` after the vote → retros.
