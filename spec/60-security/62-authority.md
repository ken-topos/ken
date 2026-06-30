# Authority, capabilities, and least privilege

> Status: **Sec2 elaborated** — implementation-ready for Team Verify (WS-Sec).
> **Normative for the authority discipline**: no-ambient enforcement,
> in-type capability tokens, **monotone-downward attenuation**, **transitive
> revocation** (static contract; runtime membrane deferred), and
> **statically-known audit points** (§1–§6, §H). The concrete surface
> *spelling* stays proposal-level (`OQ-syntax`). This is the **authority** half
> of Ken's tier-1 security story (ADR 0004); the **flow** half is
> `61-information-flow.md`, with which it composes (§6). **Settled inputs (do
> *not* reopen): `OQ-8a` DECIDED** — capabilities are first-class value tokens,
> distinct from logical `requires` (`../30-surface/36 §3`); **`OQ-Space`
> DECIDED** — shared-nothing message-passing over `space` cells, **runtime**
> revocation realization deferred to `40-runtime` (`../30-surface/36 §4`).
>
> **No new kernel rule.** No-ambient is the L5 **capability-passing
> translation** (`36 §2.5`: a `perform_E` is ordinary Π/λ over the `Cap E`
> value); attenuation rides the **landed refinement types**
> (`../30-surface/34 §5`, `../20-verification/21 §2`) as a kernel-re-checked
> obligation (§3); revocation is a **static contract** (runtime membrane
> deferred); audit points are the **`Vis` nodes the type already declares**
> (§5). The kernel gains **nothing** — capabilities are ordinary typed values
> and the authority order is an ordinary `61 §2` lattice value (§2).
>
> **Perishable — pin against the *landed* L5/Sec1 capability machinery, not
> this banner.** Capabilities are already in the elaborator:
> `CapParam { name, effect }` + `cap_set`
> (`crates/ken-elaborator/src/effects/algebra.rs`) thread one `Cap E` per
> un-handled effect through an `EffectSig` (`effects/infer.rs`, `cap_params`);
> `DeclassifyCap { from, to }` with `is_valid` (`to ⊑ from` ∧ strict),
> `check_declassify`, and `check_declassify_in_delta`
> (`crates/ken-elaborator/src/ifc.rs`) is **already a monotone-downward,
> validity-checked, delta-audited capability** — the declassification special
> case of everything below. Sec2 **generalizes** this landed pattern to
> authority + attenuation; it does **not** re-implement it. Extends `36 §3`;
> ADR 0004 Decision 4.

## 1. No ambient authority

Ken has **no ambient authority**: there is no global `open`, no implicit
filesystem or network, no process-wide mutable singletons reachable from
anywhere. A computation can act on the world *only* with an authority it was
**explicitly given** (a capability) and *only* via an effect its type declares
(`../30-surface/36 §1`). A `view` with no effect row and no capability arguments
is, by its type, **inert** — it can compute, nothing else. This is the
structural precondition for every authority claim below.

**This is the L5 capability-passing translation (`36 §2.5`), not a new gate.** A
function of row `ρ` elaborates to take one `Cap E` parameter per **un-handled**
effect `E ∈ ρ_open` as a leading argument; a `perform_E op` is well-formed
**only if `Cap E` is in scope**. So a world-action with **no** matching `Cap E`
parameter elaborates to a term that **references an unbound capability** — which
the kernel rejects as ill-typed *and* the elaborator catches earlier as the
nicer **missing-capability** diagnostic (`36 §7.3` error class 2). The
no-ambient guarantee is therefore **kernel-backed at its core** (the `perform`'s
denotation needs the `Cap E` value; the value is a real Π binding, not an erased
index) — the elaborator supplies only the source-located diagnostic, not the
soundness. A no-cap/no-row `view` denotes to `ITree 𝟘 ⟦B⟧ ≅ ⟦B⟧` (`36 §2.4`): no
`Vis` node is constructible, so it is provably effect-free.

## 2. Capabilities are static, visible, and least

A **capability** is an unforgeable authority token a computation must hold to
perform the corresponding effect (`Cap_FS`, `Cap_Net`, `Cap_declassify[ℓ→ℓ']`,
…). Per `36 §2.5`/`§3`:

- **Static + visible.** A capability is part of a function's type (a `CapParam`
  in its `EffectSig`), so a function's signature *is* its authority manifest:
  you can read, per function, exactly what it is permitted to touch — authority
  is checked **statically**, not by a runtime-only gate.
- **Least by default.** Because authority is never ambient, the default
  authority of any function is **none**; it holds exactly the capabilities its
  callers pass (`cap_params`, `36 §2.5`). The principle of least authority
  (PoLA) is the *path of least resistance*, not a discipline to remember.

### 2.1 The authority lattice (`Authority`)

A capability carries an **authority** — the *scope* of what it permits (a set of
paths for `Cap_FS`, a set of hosts for `Cap_Net`, a clearance edge for
`Cap_declassify`, optionally a quota or a validity window). Authorities form a
**bounded lattice** `Authority` — the *same machinery* as the IFC label lattice
(`61 §2.1`), an **ordinary Ken value, not a kernel primitive**: a record of a
carrier plus `⊑`/`⊔`/`⊓`/`⊥`/`⊤` plus the lattice laws as `Ω`-valued obligations
(`../10-kernel/16 §1`), discharged once per instance.

```
authority : Cap E → Authority         -- the scope a capability confers; ⊑-comparable
⊥_auth = no authority (the least)      ⊤_auth = full authority for E (the most)
```

Data flows on the `61` lattice; **authority flows on this one**, and *more
authority is higher*: `a₁ ⊑ a₂` reads "`a₁` is **weaker than (or equal to)**
`a₂`." `attenuate` moves **down** toward `⊥_auth` (§3). The concrete authority
lattice for each effect — which paths, which hosts, which edges — is a
**policy** supplied separately (`65`, ADR 0007), exactly as the IFC lattice's
instance is (`61 §2.2`): the discipline is **lattice-parametric**, the instance
is policy.

### 2.2 Unforgeability (the abstraction boundary)

`Cap E` is an **abstract (opaque) type**: user code has **no constructor** for
it. A capability value enters scope by exactly three privileged routes and no
other:

1. **Minted by a handler** — a handler is a capability provider (`36 §5`): it
   supplies `Cap E` to the body it interprets, at the authority the handler
   itself holds.
2. **Passed** — threaded as an ordinary Π parameter from a holder (`36 §2.5`).
3. **Attenuated** — derived `⊑`-downward from a held capability (§3).

Unforgeability is **load-bearing for §3**: monotone-downward attenuation guards
nothing if user code can *fabricate* a `⊤_auth` capability and skip `attenuate`
entirely. The opaque-type boundary (no public introduction form; minting
confined to the handler/runtime primitives) is what makes "the only way to a
weaker cap is `attenuate`" true. This is an **abstraction-boundary** property
(§H) — the kernel rejects a user-side `Cap E` construction because no
constructor is in scope, but *which* code is privileged to mint is an
elaborator/module discipline, not a kernel rule.

## 3. Attenuation — hand a child a strictly weaker token (the headline)

A capability holder can derive a **weaker** capability to pass onward —
**attenuation** — and **never a stronger one**:

```
attenuate : (c : Cap E) (w : Authority)
          → { c' : Cap E | authority c' ⊑ authority c ⊓ w }
```

- Attenuation **narrows**: a smaller scope (one directory, not the filesystem;
  one host, not the network), a lower clearance, a tighter quota, a shorter
  validity window. The result authority is bounded by **both** the parent's
  authority **and** `w` — their meet `authority c ⊓ w` (`⊓-glb`, `61 §2.1`).
- A child therefore **cannot exceed** the authority its parent delegated, *by
  construction*. "This AI-generated helper must not reach the network beyond
  `api.example.com`" becomes a **compile-time fact**, not a code-review hope.

### 3.1 The encoding — a kernel-re-checked refinement obligation

The result type is a **landed refinement** (`34 §5`, `21 §2`): it elaborates to
the **carrier `Cap E`** with the predicate `authority c' ⊑ authority c ⊓ w`
tracked by the elaborator, and **producing the result emits the obligation**
`authority c' ⊑ authority c ⊓ w` (`22 §2.1`), discharged by the prover and
**re-checked by the kernel** (`23 §1`, `18 §4`). The canonical body computes
`authority c' = authority c ⊓ w`, so the obligation is `(authority c ⊓ w) ⊑
(authority c ⊓ w)` — discharged by `⊑-refl`.

**Kernel-backed *when the authority is kernel-visible* — not a uniform claim
(the honest split).** A capability is a **real Π value** (`36 §2.5`), when its
`Authority` ranges over **kernel-visible values** — FS paths, `Net` hosts, a
quota, a window (common case) — `authority c'`/`⊑` are **real terms** and the
bound is a real `Ω` obligation the kernel **certifies**, a genuine difference
from Sec1's flow rules (which are *trusted*: IFC labels are erased, conformance
the sole net, `61 §H`/§9 N1). **The exception: the declassify capability**
`Cap_declassify[ℓ→ℓ']` (`61 §4`): its authority **is an IFC label edge**, and
IFC labels are **erased before the kernel** (`61 §3`/§9 N1), so *its* monotone
bound (`ℓ' ⊑ ℓ`) is **trusted-by-typing** — exactly the landed elaborator check
`DeclassifyCap.is_valid` (`ifc.rs`), **not** a kernel obligation. The rule:
**real-value authority → kernel-backed obligation; label-mediated authority
(declassify) → trusted-by-typing**, mirroring `61 §H`. Filing a label-mediated
guarantee as kernel-certified over-claims (the `61 §9 N1` erasure boundary);
the safe, accurate split is the one above.

**The use-site dual is also a refinement obligation.** A world-action that needs
*at least* authority `a` declares its capability parameter **refined**:

```
view write_at (c : { c : Cap_FS | a ⊑ authority c }) (p : Path) … visits [FS]
```

so **calling** it with a capability `c''` emits the obligation `a ⊑ authority
c''` — kernel-re-checked. A capability is **sufficient** for an op iff its
authority is `⊒` the demand. Both the *production* of a weaker cap (attenuate)
and its *consumption* at a sufficiency-demanding sink ride the same
refinement-obligation machinery.

### 3.2 No amplification — assert the absence, and net the orientation

**There is no operation that amplifies authority.** No `strengthen`, no
`amplify`, no public `Cap` constructor (§2.2 unforgeability) — the only typed
arrow into `Cap E` from a held `Cap E` is `attenuate`, and its codomain is
`⊑`-bounded. **Soundness of "downward-only" is the conjunction of three facts**:
(a) the attenuate bound (§3.1, kernel-backed); (b) the **enumerated absence** of
any amplifying operation (nothing to call); (c) **unforgeability** (§2.2, you
cannot sidestep `attenuate`).

**The order-dual soundness net (`[Sec1-dual]` trap-class).** `⊑` on `Authority`
is a **direction** — getting it **backwards** (writing the bound or the
sufficiency check as `⊒`) **silently inverts** attenuate-weakens into
attenuate-strengthens, exactly the taint-axis orientation hazard of Sec1's
integrity / `@ct` axes (`61 §5a.1`/§H). And the kernel obligation **alone does
not net it**: the canonical witness `authority c' = authority c ⊓ w` discharges
**both** `authority c' ⊑ authority c ⊓ w` **and** reversed `authority c ⊓ w ⊑
authority c'` by `⊑-refl` — the bound is **direction-degenerate at meet**, so
a backwards rule still type-checks. The orientation is held **only** by a
**non-degenerate distinguishing pair** (conformance AC3) on **strict**
authorities (`authority c ⊓ w ⊏ authority c`):

- **weaker cap at a weak sink — ACCEPTS** (`authority c ⊓ w ⊑ authority c'` =
  refl): the child's reduced demand is met. *(Necessary, but degenerate alone —
  green under both orientations.)*
- **weaker cap at a sink demanding the parent's full authority — REJECTS**
  (`authority c ⋢ authority c'`, strict): the weakened cap is **insufficient**.
  *This is the net:* under a backwards `⊑` it would **wrongly accept** (a
  weakened cap passing a strong sink — privilege escalation). The pair flips
  green↔red on exactly the orientation bug; a single accept case cannot.

So the headline soundness property is **kernel-backed in its bound yet
conformance-netted in its orientation** — both nets required, for *different*
reasons than Sec1 (there the rule is erased; here the bound is degenerate at the
meet).

## 4. Revocation — transitive, fail-closed (static contract; runtime deferred)

Authority must be **revocable** at a boundary: a delegated capability can be
withdrawn, and **everything attenuated from it** withdrawn with it.

- A **revocable capability** is tied to a controlling `space` cell (`36 §4`)
  whose validity gates the capability; **revoking flips the cell**, after which
  the capability **and every capability `⊑`-derived from it** (§3) **fails
  closed** — a subsequent `perform` through it is denied.
- Revocation is **transitive**: because `attenuate` derives a child *from* the
  parent (§2.2 route 3), the child's validity is **bounded by the parent's** —
  the membrane bounds the whole sub-delegation. Revoking `c` revokes every
  `c' = attenuate c w` (and their descendants), to any depth.

**Two faces — Sec2 delivers the static one (named split, not a buried gap).**

- **Static face (delivered, this WP).** The **typed revoke interface** and the
  **transitivity property statement**: an attenuated capability's validity is
  *derived from* its parent's, so "`c` revoked ⇒ everything attenuated from `c`
  revoked" is a static contract over the delegation tree. Sec2 pins this
  contract and the interface shape.
- **Runtime face (DEFERRED → `40-runtime`, `OQ-Space`).** The **mechanism** that
  *realizes* fail-closed at evaluation — forwarder / membrane / validity-indexed
  / region lifetime in the controlling space — is a downstream runtime WP. It is
  **`(oracle)`-tagged**, named not omitted: the static contract is built so the
  runtime realization rides it. ADR 0004 requires that mechanism to carry a
  *stated, proven* isolation property (`OQ-Space`), resting on the
  shared-nothing guarantee (`36 §4.4`).

## 5. Audit at trust boundaries — statically known

Authority exercised across a trust boundary is **auditable**, and the audit
points are **static**:

- A trust boundary — a `space` edge, FFI (`38 §3`), a **declassification**
  (`61 §4`), a capability **delegation** — is **exactly** a `Vis` node the
  function's type declares (`36 §3.1`: every authority-relevant act is a `Vis`
  node; nothing effectful hides between nodes). So the set of audit points is
  **recoverable from the type**, and an **un-audited boundary effect is
  impossible**: you cannot perform an effect the row did not declare (`36 §1.4`
  escape check), and a no-row `view` is inert (§1). The **statically-known**
  property is therefore kernel-backed by the row discipline; what each record
  *contains* — *what* authority, *by whom*, *what* effect — is the audit-record
  shape this chapter fixes.
- **Declassification (`61 §4`) is a capability whose use is audited.** Each
  `declassify` is a recorded event at a trust boundary, and the declassification
  authority a dependency holds appears in its **`trusted_base_delta`** (`63`,
  `25 §3`) — the landed `check_declassify_in_delta` (`ifc.rs`) is exactly this
  check. A package that downgrades secrets **cannot hide it**.

**Static face vs runtime face (the same split as §4).** Sec2 delivers the
*static* audit surface — the boundary set is type-determined, and declassify's
every-use audit point is a static site. The *runtime emission* of records
(serialization, tamper-evidence / append-only log) is a runtime/`Ward` concern,
`(oracle)`-tagged here — named, not absorbed.

## 6. Relationship to effects and flow — authority + flow compose

Authority and flow compose: a capability **gates an effect** (you may write to
`Net` only with `Cap_Net`), and the sink that capability opens **carries a
clearance label** (`61 §3`, you may write only data `⊑` that clearance). So a
single typed arrow expresses both *may this code act* (capability, this chapter)
and *may this data flow here* (label, `61`):

```
view send (c : Cap_Net) (s : Socket κ) (msg : Bytes @ ℓ) : Unit  visits [Net]
  -- well-formed iff  c present (authority)  AND  ℓ ⊔ pc ⊑ κ  (flow, 61 §3.1 L-SINK)
```

**Both** concessions are required and they are **independent**: dropping the
capability is a missing-capability error (§1); dropping the flow check is an
`IFC-FLOW` error (`61 §3.1`). A `Secret` datum to a `Cap_Net` sink at `Public`
clearance is rejected **even with** `Cap_Net` held — authority does not buy
clearance, and clearance does not buy authority. Pure-by-default +
least-authority + upward-only-flow together make "an AI-written helper leaks a
secret to the network" require **three** explicit, visible, audited concessions
— each a place a reviewer or policy can say no.

## 7. Worked examples

```
-- No ambient authority: a no-cap/no-row view is inert by its type.
view classify (x : Record) : Tag = …          -- cannot touch FS/Net/anything (§1)

-- A world-action REQUIRES the capability + the declared effect.
view save (c : Cap_FS) (p : Path) (d : Bytes) : Unit  visits [FS]
  = write_at c p d
view save_bad (p : Path) (d : Bytes) : Unit   visits [FS]
  = write_at ??? p d                            -- REJECTED: missing Cap_FS (§1)

-- Attenuation: derive a strictly weaker cap; never a stronger one.
view sandbox (c : Cap_FS) : Unit  visits [FS]
  = let c_tmp = attenuate c (only_dir "/tmp")  -- c_tmp : Cap_FS, authority ⊑ /tmp
    in helper c_tmp                             -- helper gets exactly /tmp
       -- emits  authority c_tmp ⊑ authority c ⊓ only_dir "/tmp"  (kernel-re-checked, §3.1)

-- THE order-dual pair (AC3): the SAME attenuated cap, two sinks, verdict FLIPS.
write_at c_tmp (path "/tmp/a") d   -- ACCEPTS: weak sink, authority "/tmp" suffices
write_at c_tmp (path "/etc/passwd") d
       -- REJECTED: a "/etc"-demanding sink needs authority ⊒ "/etc"; c_tmp has "/tmp" ⊏ that.
       --   Under a BACKWARDS ⊑ this would WRONGLY accept (escalation) — the net.

-- No amplification: there is no operation to call.
attenuate c (full_authority)   -- still ⊑ authority c — CANNOT exceed the parent
-- strengthen c …              -- DOES NOT EXIST (no such typed arrow; Cap is unforgeable §2.2)

-- Revocation transitive (static contract; runtime fail-closed deferred §4).
view supervise (c : Cap_Net) : Unit  visits [Net, Revoke]
  = let c_child = attenuate c (only_host "api.example.com")
    in do { delegate c_child;  revoke c }       -- revoking c fails-closed c_child too

-- Authority + flow compose (AC6): BOTH concessions, independent.
view exfil (c : Cap_Net) (s : Socket Public) (secret : Bytes @ Secret) : Unit  visits [Net]
  = send c s secret    -- REJECTED: has Cap_Net, but Secret ⋢ Public (61 L-SINK).
                       --   Authority present, FLOW denied — dropping EITHER concession rejects.
```

A CISO reads these and sees no-ambient confinement, least authority,
non-amplifiable delegation, transitive revocation, and audited boundaries as
**typed, compile-time** properties — the controls ("this generated component
cannot reach the network / the disk beyond X") enforced by construction, not by
review.

## H. Honest limits — kernel-backed vs trusted vs deferred

Per `64 §4`: **a verified language that over-claims is itself a security risk.**
Ken states its authority boundaries exactly. **None of this enlarges the trusted
kernel** — capabilities are ordinary Π (`36 §2.5`), the authority order is
an ordinary lattice (§2.1), and real-value attenuation bounds are `21 §2`
obligations re-checked by the *same* small kernel (the declassify-edge bound
excepted — it is over erased labels, §3.1) (ADR 0004 Decision 3, ADR 0001).

| Aspect | Status | Detail |
|---|---|---|
| No ambient authority — a `perform_E` needs `Cap E` in scope | **kernel-backed** | the cap is a real Π parameter (`36 §2.5`); a world-action with no matching cap denotes to an unbound reference the kernel rejects (§1). The elaborator adds only the source-located **missing-capability** diagnostic |
| Least by default — a function holds exactly the caps it is passed | **kernel-backed** | same mechanism — using an un-passed capability is an unbound reference; default authority is `∅` |
| Attenuation **monotone bound** `authority c' ⊑ authority c ⊓ w` (real-value authority) | **kernel-backed (refinement obligation) — but direction-degenerate** | a `34 §5`/`21 §2` obligation, kernel-re-checked (§3.1) — *stronger* than Sec1's erased flow rules. **Yet** the meet-witness discharges both `⊑` orientations by refl, so the **orientation** is netted by the non-degenerate conformance pair, not the kernel (§3.2) |
| Attenuation bound of the **declassify** cap `ℓ' ⊑ ℓ` | **trusted-by-typing** | its authority is an **IFC label edge** and labels are erased before the kernel (`61 §3`/§9 N1), so the bound is the landed elaborator check `DeclassifyCap.is_valid`, **not** a kernel obligation — exactly Sec1's erased-label posture (§3.1) |
| Use-site **sufficiency** `a ⊑ authority c` | **kernel-backed (refinement obligation)** | a sink refines its cap parameter `{c | a ⊑ authority c}`; each call emits the obligation (§3.1) |
| **No amplification** | **trusted by enumerated absence** | no `strengthen`/`amplify`/public `Cap` constructor exists — there is *nothing to call*; conformance asserts the **absence** (§3.2), the kernel cannot witness a non-existent operation |
| **Unforgeability** of `Cap E` | **abstraction-boundary** | `Cap E` is opaque (no public constructor); minting is confined to handlers/runtime primitives. The kernel rejects a user-side construction (no constructor in scope); *which* code is privileged is an elaborator/module discipline (§2.2) |
| Revocation **transitivity** (the contract) | **static contract — kernel-backed property** | "`c` revoked ⇒ everything attenuated from `c` revoked" follows from the delegation derivation (§4) |
| Revocation **mechanism** (fail-closed at eval) | **deferred → `40-runtime` / `OQ-Space`** | the forwarder/membrane is `(oracle)`-tagged; named, not omitted (§4) |
| Audit points **statically known** | **kernel-backed** | the boundary set = the `Vis` nodes the type declares; an un-audited declared effect is impossible (`36 §1.4`, §5) |
| Audit-record **emission** (log, tamper-evidence) | **deferred → runtime / `Ward`** | the static surface is fixed; runtime serialization is `(oracle)`-tagged (§5) |
| Authority + flow **compose** | **kernel-backed (authority) ∧ trusted-by-typing (flow)** | dropping the cap is kernel-caught (§1); dropping the flow check is the Sec1 erased-label rule (`61 §H`); both required (§6) |
| The **policy** (which paths/hosts/edges an authority lattice has) | **assumed** | a wrong policy ⇒ a wrong guarantee — the `64 §4.1` spec≠intent analog; the policy (`65`) is the human-reviewed boundary, exactly as for IFC (`61 §H`) |

**The Sec2 vs Sec1 contrast (worth stating, the design payoff).** Sec1's IFC
labels are **erased** before the kernel, so its flow rules are *trusted* and the
conformance corpus is the **sole** net (`61 §9 N1`). Sec2's capabilities are
**real values**, so the attenuation *bound* and use-site *sufficiency* over
**kernel-visible** authority are **kernel-backed** obligations — a strictly
smaller trusted surface. What remains trusted is **narrower and named**: the
*orientation* of `⊑` (degenerate at meet → conformance pair), the **absence**
of amplification, **unforgeability** (abstraction boundary), the **declassify**
cap's bound (over erased labels → trusted-by-typing, the one Sec1-style
exception, §3.1), and the **runtime** revocation/audit faces (deferred to
`40-runtime`/`Ward`).

## 8. What is committed vs. open

- **Committed:** no ambient authority (§1); static + visible capabilities, least
  by default (§2); **attenuation** monotone-downward with a **kernel-re-checked
  bound** (§3); **no amplifying operation** (§3.2); **transitive revocation** as
  a static contract (§4); statically-known **boundary audit** (§5); capabilities
  **gate effects and compose with clearance** (§6).
- **Decided (`OQ-8a`):** capabilities are first-class tokens, handler-or-row
  supplied, attenuable/revocable/audited, distinct from logical `requires`
  (`36 §3`).
- **Decided (`OQ-Space`):** shared-nothing message-passing over encapsulated
  non-aliased `space` cells (`36 §4`); revocation via the space model
  (forwarder/membrane in a controlling space); **runtime realization deferred to
  `40-runtime`** (§4). The *security requirement* — attenuable, revocable,
  audited, least — is **fixed regardless of the runtime construct form**.
- **Deferred (named, `(oracle)`-tagged):** the runtime revocation **membrane**
  (§4) and runtime audit-record **emission** (§5) — both `40-runtime`/`Ward`,
  riding the static contracts this chapter pins.

## 9. What Team Verify must deliver here (Sec2)

The deliverable is the elaboration above, made impl-ready. Each item is
a concrete, codeable section; an implementer builds from these and the kernel
re-checks the emitted core (the elaborator is **not** in the TCB, `36 §7`):

1. **No-ambient enforcement** — the capability-passing translation gate (§1,
   building on landed `CapParam`/`cap_set`): a world-action requires its `Cap E`
   parameter + declared row; a no-cap/no-row `view` is inert. *AC1.*
2. **Capability tokens in the type** — `Cap E` as an opaque token, the
   signature-as-manifest, least-by-default (§2), and the `Authority` lattice +
   `⊑` order as a `61 §2` lattice value (§2.1). *AC2.*
3. **Attenuation** — the typed `attenuate` producing `{c' | authority c' ⊑
   authority c ⊓ w}` as a kernel-re-checked refinement obligation (§3.1), the
   use-site sufficiency refinement, and the **enumerated absence** of any
   amplifying operation (§3.2). *AC3 — the headline.*
4. **Revocation (static contract)** — the typed revoke interface + transitivity
   (§4); **defer the runtime membrane** to `40-runtime` (`OQ-Space`), oracle-tag
   the mechanism, pin the contract. *AC4.*
5. **Audit points** — the static boundary set + the audit-record shape;
   declassification every-use-audited and in `trusted_base_delta` (§5). *AC5.*
6. **Authority + flow composition** — a capability gates an effect **and** the
   sink carries a clearance label; dropping either rejects (§6). *AC6.*

### Level reconciliation (the soundness check — before the Architect handoff)

The authority constructs add **no new level rule** — only instances of existing
formation (`36 §7.4`, `61 §9`):

| Construct | Level | Rule |
|---|---|---|
| `Cap E` (capability token) | `Type ℓ_op` | a value type (`36 §2.5`); opaque (§2.2), no new former |
| `Authority` (carrier + ops record) | `Type (suc ℓ)` | record / Σ-Form (`13 §1`), laws at `Ω` — a `61 §2.1` lattice value |
| `authority : Cap E → Authority` | ordinary Π | a projection; no new rule |
| `{ c' : Cap E \| authority c' ⊑ authority c ⊓ w }` | `level(Cap E) = ℓ_op` | refinement = carrier + obligation (`21 §2`, `34 §5`); **predicative** (`12 §2`), **non-cumulative** (`12 §3`), same level as the carrier — adds no Σ over `Ω` |
| `authority c' ⊑ authority c ⊓ w` | `Ω` | an ordinary `Ω`-valued obligation (`22 §1`, `16 §1`) |
| revoke interface / validity tie | ordinary value | a `space`-cell-gated value (`36 §4`); no new former |

Every level is the **predicative `max`** of its parts (`12 §2`), non-cumulative
(`12 §3`); the elaborator emits explicit levels and the kernel re-checks them
(`12 §4`). The authority discipline is impredicative nowhere — it reuses
Π/Σ/inductive/refinement, adding **no new level rule**.

Conformance: `../../conformance/security/capabilities/` — AC1–AC6 with
**discriminating** cases (COORDINATION §7; every negative case **flips** on the
bug it targets). **AC3 is the order-dual distinguishing pair** (weaker-accepts /
stronger-rejects on the **same** cap shape, **non-degenerate** authorities, real
`Cap` values through the real `authority`-`⊑` check — never a synthetic flag),
plus the cross-case sweep (the no-ambient class agrees; every authority bound is
kernel-backed; the orientation is netted by the pair).
