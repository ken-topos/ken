# Authority, capabilities, and least privilege

> Status: **Sec2 elaborated** — implementation-ready for Team Verify (WS-Sec).
> **Normative for the authority discipline**: no-ambient enforcement,
> in-type capability tokens, **monotone-downward attenuation**, **transitive
> revocation** (static contract plus §4's bounded OS-operation behavior;
> runtime mechanism deferred), and
> **statically-known audit points** (§1–§6, §H). The concrete surface
> *spelling* stays proposal-level (`OQ-syntax`). This is the **authority** half
> of Ken's tier-1 security story (ADR 0004); the **flow** half is
> `61-information-flow.md`, with which it composes (§6). **Settled inputs (do
> *not* reopen): `OQ-8a` DECIDED** — capabilities are first-class value tokens,
> distinct from logical `requires` (`../30-surface/36 §3`); **`OQ-Space`
> DECIDED** — shared-nothing message-passing over `space` cells. Section 4 pins
> the current implicit root space's OS-operation revocation behavior; general
> multi-space and cross-space realization remains deferred to `40-runtime`
> (`../30-surface/36 §4`).
>
> **No new kernel rule.** No-ambient is the L5 **capability-passing
> translation** (`36 §2.5`: a `perform_E` is ordinary Π/λ over the `Cap E`
> value); attenuation rides the **landed refinement types**
> (`../30-surface/34 §5`, `../20-verification/21 §2`) as a kernel-re-checked
> obligation (§3); revocation combines the static lineage contract with §4's
> bounded runtime behavior, while its mechanism remains deferred; audit points
> are the **`Vis` nodes the type already declares** (§5). The kernel gains
> **nothing** — capabilities are ordinary typed values and the authority order
> is an ordinary `61 §2` lattice value (§2).
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
`a₂`." Attenuation moves **down** toward `⊥_auth` (§3). The concrete authority
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
3. **Attenuated by the trusted runner/host** — derived `⊑`-downward from a held
   capability and then supplied through an existing privileged route (§3).

Unforgeability is **load-bearing for §3**: monotone-downward attenuation guards
nothing if user code can *fabricate* a `⊤_auth` capability or invoke the raw
attenuation action itself. The opaque-type boundary (no public introduction
form; minting and attenuation confined to trusted handler/runner/runtime
machinery) makes the trusted attenuation route exclusive. This is an
**abstraction-boundary** property (§H) — the kernel rejects a user-side `Cap E`
construction because no constructor is in scope, while I-4 also requires the
raw names `attenuate` and `revoke` to be unbound in Ken (`38 §1.3.1`). Which
code is privileged to mint or derive a capability is a runner/module
discipline, not a kernel rule.

## 3. Attenuation — hand a child a strictly weaker token (the headline)

A trusted runner/host can derive a **weaker** capability from one already held —
**attenuation** — and **never a stronger one**. Semantically, attenuating parent
`c` by bound `w` produces a child `c'` only if `authority c' ⊑ authority c ⊓ w`.
This relation is **not** a Ken declaration or callable signature.

- Attenuation **narrows**: a smaller scope (one directory, not the filesystem;
  one host, not the network), a lower clearance, a tighter quota, a shorter
  validity window. The result authority is bounded by **both** the parent's
  authority **and** `w` — their meet `authority c ⊓ w` (`⊓-glb`, `61 §2.1`).
- A child therefore **cannot exceed** the authority its parent delegated, *by
  construction*. "This AI-generated helper must not reach the network beyond
  `api.example.com`" becomes a **compile-time fact**, not a code-review hope.

### 3.1 The encoding — a kernel-re-checked refinement obligation

The child exposed through an existing privileged capability path carries the
**landed refinement** (`34 §5`, `21 §2`): the **carrier `Cap E`** with predicate
`authority c' ⊑ authority c ⊓ w`. Supplying the child emits the obligation
`authority c' ⊑ authority c ⊓ w` (`22 §2.1`), discharged by the prover and
**re-checked by the kernel** (`23 §1`, `18 §4`). The trusted action establishes
`authority c' = authority c ⊓ w`, so the obligation is `(authority c ⊓ w) ⊑
(authority c ⊓ w)` — discharged by `⊑-refl`. This does not introduce a public
capability-producing wrapper (§4, `38 §1.3.1`).

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
authority is `⊒` the demand. Both the trusted *production* of a weaker cap and
its *consumption* at a sufficiency-demanding sink ride the same
refinement-obligation machinery; only the latter is directly Ken-callable.

### 3.2 No amplification — assert the absence, and net the orientation

**There is no Ken operation that amplifies or attenuates authority.** No
`strengthen`, `amplify`, `attenuate`, `revoke`, or public `Cap` constructor or
producer is bound in the Ken environment (§2.2, §4, `38 §1.3.1`). The trusted
runner/host's only derivation action is the raw, `⊑`-bounded attenuation
relation above. **Soundness of "downward-only" is the conjunction of three
facts**: (a) the attenuation bound (§3.1, kernel-backed when the authority is
kernel-visible); (b) the **enumerated absence** of any source operation that can
produce or alter a capability; and (c) **unforgeability** (§2.2).

**The order-dual soundness net (`[Sec1-dual]` trap-class).** `⊑` on `Authority`
is a **direction** — getting it **backwards** (writing the bound or the
sufficiency check as `⊒`) **silently inverts** attenuation-weakens into
attenuation-strengthens, exactly the taint-axis orientation hazard of Sec1's
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

## 4. Revocation — transitive, fail-closed, and bounded at runtime

Authority is **revocable**: a delegated capability can be withdrawn, and
**everything attenuated from it** is withdrawn with it. The trusted runner/host
maintains a non-Ken-visible revocation identity for each grant. Copying a
capability preserves that identity. The raw management action `attenuate`
creates a child identity linked to its parent; raw `revoke` closes the selected
identity and every descendant, to any depth, but not its parent or siblings.
A resource acquired under that authority, and any duplicate of that resource,
remains governed by the same lineage unless a future explicit reauthorization
establishes a different sponsor. Consuming a resource token therefore cannot
bypass revocation.

These raw actions are **management semantics, not Ken terms**. Neither
`attenuate` nor `revoke` is a Ken global, capability constructor, effect
producer, or capability-producing wrapper. They are absent from the Ken name
environment, as is every public `Cap` constructor or producer. Existing
`ProgramCaps`, `readFile`, and `writeFile` remain the source-facing capability
path (`38 §1.3.1`). A program observes revocation only when a later, existing
capability-consuming operation is denied; it cannot invoke or directly inspect
either management action or the identity they govern.

### 4.1 One denial identity, two public projections

Revocation has one semantic denial identity with two exact, type-local public
projections. The existing result families remain distinct:

- a path/capability operation returns
  `MkFileError <operation> <path> Revoked`, where `Revoked` is a new `IOError`
  cause beside and distinct from `CapabilityDenied`;
- a resource-token operation returns the nullary constructor
  `ResourceError.Revoked`, distinct from `Closed`, `MalformedResource`,
  `RightNotHeld`, `ResourceKindMismatch`, and `ResourceHostIO _`.

The runtime/host mapping preserves this correspondence. It must not collapse
either projection into `CapabilityDenied`,
`ResourceHostIO CapabilityDenied`, `Closed`, malformed capability/resource,
stale-generation, `RightNotHeld`, or a host I/O error. The discriminator applies
when revocation is the reason an otherwise well-formed, live,
sufficiently-righted operation is refused; this supplies a non-degenerate
control for each neighbouring denial without choosing precedence for an input
that is invalid in several independent ways.

### 4.2 Admission and settlement

**Admission is the linearization point.** Admission succeeds only while the
addressed identity and every ancestor are live. It separates exactly two
observable outcomes:

- **revoke before admission:** the capability-consuming operation returns the
  appropriate `Revoked` projection from §4.1, and no guarded OS backend
  operation occurs;
- **admission before revoke:** the admitted operation may finish and returns its
  real result, whether success or its actual non-revocation error. A later
  revoke does not rewrite that result to `Revoked`, and a side effect may already
  have committed.

Revocation promises **neither rollback nor cancellation**. Cancellation is a
separate operation and cannot be inferred from revocation. Revocation closes
new admissions immediately, while an already-admitted operation settles
normally. An owned OS resource is settled only after all such operations finish;
its close success or `ReleaseFailed` outcome remains recorded exactly once under
ADR 0021's resource identity and settlement discipline. Settlement failure does
not reopen authority.

### 4.3 Honest runtime boundary

This section closes only the **current OS-operation runtime face** for Ken's
implicit root execution space. It does **not** claim general runtime realization
of surface `space` (`36 §4`), separate runtime spaces, cross-space forwarders,
transport, cross-space or distributed revocation, or distributed isolation.
`44 §3` already realizes the memory/reclamation projection of each surface
`space` as a store `Space`; this contract neither denies that realization nor
claims its missing state/effect, authority, isolation, or transport projections
are delivered.

The runtime representation and isolation argument remain a `40-runtime` ADR
choice. A controlling space cell, forwarder, validity index, or region lifetime
is not normative here. Whatever mechanism is chosen must preserve the lineage,
descendant closure, admission boundary, two `Revoked` projections, and
settlement observations above.

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

-- An attenuated child is supplied through an existing privileged route; Ken
-- receives it but cannot invoke the raw action that created it.
view sandbox
    (c : Cap_FS)
    (c_tmp : { c' : Cap_FS | authority c' ⊑ authority c ⊓ only_dir "/tmp" })
    : Unit  visits [FS]
  = helper c_tmp                                -- helper gets exactly /tmp

-- THE order-dual pair (AC3): the SAME attenuated cap, two sinks, verdict FLIPS.
write_at c_tmp (path "/tmp/a") d   -- ACCEPTS: weak sink, authority "/tmp" suffices
write_at c_tmp (path "/etc/passwd") d
       -- REJECTED: a "/etc"-demanding sink needs authority ⊒ "/etc"; c_tmp has "/tmp" ⊏ that.
       --   Under a BACKWARDS ⊑ this would WRONGLY accept (escalation) — the net.

-- No public production or management action: all three names are absent.
attenuate c (full_authority)   -- REJECTED: UnboundName (I-4)
revoke c                       -- REJECTED: UnboundName (§4)
strengthen c …                 -- REJECTED: UnboundName (§3.2)

-- Revocation is observed through an existing capability-consuming operation.
-- If the trusted host revoked c_child or an ancestor before admission, the
-- call returns MkFileError ReadFile path Revoked and performs no backend read.
view use_child (c_child : Cap_FS) (path : Bytes)
    : Result FileError Bytes  visits [FS]
  = readFile APartial c_child path

-- Authority + flow compose (AC6): BOTH concessions, independent.
view exfil (c : Cap_Net) (s : Socket Public) (secret : Bytes @ Secret) : Unit  visits [Net]
  = send c s secret    -- REJECTED: has Cap_Net, but Secret ⋢ Public (61 L-SINK).
                       --   Authority present, FLOW denied — dropping EITHER concession rejects.
```

A CISO reads these and sees no-ambient confinement, least authority,
non-amplifiable delegation, and audited boundaries in the typed surface, plus
transitive, fail-visible revocation at the runtime boundary. The static controls
("this generated component cannot reach the network / the disk beyond X") are
enforced by construction; §4 names the separate runtime-trusted guarantee
without presenting it as a kernel theorem.

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
| **No amplification / source attenuation** | **trusted by enumerated absence** | no `strengthen`/`amplify`/`attenuate`/`revoke` or public `Cap` constructor/producer exists — there is nothing to call; conformance asserts the positive wrappers plus this complete absence (§3.2, §4), which the kernel cannot witness |
| **Unforgeability** of `Cap E` | **abstraction-boundary** | `Cap E` is opaque; minting and raw management are confined to trusted handler/runner/runtime machinery. The kernel rejects a user-side construction (no constructor in scope); I-4 separately nets the absence of every producer/management name (§2.2) |
| Revocation **lineage + bounded OS-operation behavior** | **runtime-trusted contract; mechanism deferred** | raw management is not Ken-callable; descendant closure, admission linearization, the two distinct `Revoked` projections, and settlement are normative for the current implicit root space (§4) |
| Revocation **mechanism + general space realization** | **deferred → `40-runtime` / `OQ-Space`** | representation and isolation argument are ADR-owned; no general multi-space, cross-space, transport, or distributed claim (§4.3) |
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
exception, §3.1), §4's runtime-trusted revocation contract and deferred
mechanism, and runtime audit emission (`40-runtime`/`Ward`).

## 8. What is committed vs. open

- **Committed:** no ambient authority (§1); static + visible capabilities, least
  by default (§2); **attenuation** monotone-downward with a **kernel-re-checked
  bound** (§3); **no public producing, attenuating, revoking, or amplifying
  operation** (§3.2/§4); transitive revocation plus the bounded OS-operation
  contract for the implicit root space (§4); statically-known **boundary audit**
  (§5); capabilities **gate effects and compose with clearance** (§6).
- **Decided (`OQ-8a`):** capabilities are first-class tokens, handler-or-row
  supplied, attenuable/revocable/audited, distinct from logical `requires`
  (`36 §3`).
- **Decided (`OQ-Space`):** shared-nothing message-passing over encapsulated,
  non-aliased `space` cells (`36 §4`). Section 4 pins the current implicit root
  space's OS-operation revocation behavior without choosing a runtime
  representation. General multi-space/cross-space realization remains deferred
  to `40-runtime`. The *security requirement* — attenuable, revocable, audited,
  least — is fixed regardless of runtime construct form.
- **Deferred (named, `(oracle)`-tagged):** the implementation and isolation
  argument for §4's bounded contract, general space realization, and runtime
  audit-record **emission** (§5) — `40-runtime`/`Ward`, riding the contracts this
  chapter pins.

## 9. What Team Verify must deliver here (Sec2)

The Sec2 deliverable is the elaboration below, made impl-ready. Each item is a
concrete, codeable section; an implementer builds from these and the kernel
re-checks the emitted core (the elaborator is **not** in the TCB, `36 §7`).
Section 4's ABI-REVOKE runtime additions remain Runtime-owned; recording them
here does not reassign their implementation to Team Verify:

1. **No-ambient enforcement** — the capability-passing translation gate (§1,
   building on landed `CapParam`/`cap_set`): a world-action requires its `Cap E`
   parameter + declared row; a no-cap/no-row `view` is inert. *AC1.*
2. **Capability tokens in the type** — `Cap E` as an opaque token, the
   signature-as-manifest, least-by-default (§2), and the `Authority` lattice +
   `⊑` order as a `61 §2` lattice value (§2.1). *AC2.*
3. **Attenuation** — the trusted raw derivation supplies a child satisfying
   `{c' | authority c' ⊑ authority c ⊓ w}` as a kernel-re-checked refinement
   obligation (§3.1), while the Ken environment exposes no capability producer,
   `attenuate`, `revoke`, or amplifying operation (§3.2/§4). *AC3 — the
   headline.*
4. **Revocation split** — Sec2 supplies the non-Ken-visible raw-management
   boundary and lineage contract. ABI-REVOKE supplies the Runtime-owned two
   public `Revoked` projections, admission linearization, and settlement for the
   bounded implicit-root OS-operation face (§4). The mechanism, ADR isolation
   argument, and general space realization remain `40-runtime` work. *AC4.*
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
| `Authority` (carrier + ops record) | `Type (suc ℓ)` | record / Σ-Form (`13 §2`), laws at `Ω` — a `61 §2.1` lattice value |
| `authority : Cap E → Authority` | ordinary Π | a projection; no new rule |
| `{ c' : Cap E \| authority c' ⊑ authority c ⊓ w }` | `level(Cap E) = ℓ_op` | refinement = carrier + obligation (`21 §2`, `34 §5`); **predicative** (`12 §2`), **non-cumulative** (`12 §3`), same level as the carrier — adds no Σ over `Ω` |
| `authority c' ⊑ authority c ⊓ w` | `Ω` | an ordinary `Ω`-valued obligation (`22 §1`, `16 §1`) |
| raw attenuation/revocation identity | non-Ken-visible runtime state | no source former, global, constructor, producer, or new kernel rule (§4) |

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
