# Authority, capabilities, and least privilege

> Status: **DRAFT v0**. Normative for the model; the construct form is
> **OQ-8a**. How Ken confines *what code is allowed to do* — the authority half
> of the security story (the flow half is `61-information-flow.md`). Extends
> `../30-surface/36 §3`; ADR 0004 Decision 4.

## 1. No ambient authority

Ken has **no ambient authority**: there is no global `open`, no implicit
filesystem or network, no process-wide mutable singletons reachable from
anywhere. A computation can act on the world *only* with an authority it was
**explicitly given** (a capability) and *only* via an effect its type declares
(`../30-surface/36`). A `view` with no effect row and no capability arguments
is, by its type, **inert** — it can compute, nothing else. This is the
structural precondition for every authority claim below.

## 2. Capabilities are static, visible, and least

A **capability** is an unforgeable authority token a computation must hold to
perform the corresponding effect (`Cap_FS`, `Cap_Net`, `Cap_declassify[ℓ→ℓ']`,
…). Per `36 §3`:

- **Static + visible.** A capability is part of a function's type, so a
  function's signature *is* its authority manifest: you can read, per function,
  exactly what it is permitted to touch. (Not the prototype's runtime-only
  gate.)
- **Least by default.** Because authority is never ambient, the default
  authority of any function is **none**; it receives exactly the capabilities
  its callers pass. This makes the principle of least authority (PoLA) the *path
  of least resistance*, not a discipline to remember.

## 3. Attenuation (hand a child a weaker token)

A capability holder can derive a **strictly weaker** capability to pass onward —
**attenuation** — and never a stronger one:

```
attenuate : (c : Cap) (w : Authority)
          → { c' : Cap | authority c' ⊑ authority c ⊓ w }
```

- Attenuation **narrows**: a smaller scope (one directory, not the filesystem;
  one host, not the network), a lower clearance label (`61`), a tighter
  rate/quota, a shorter validity window.
- Attenuation is **monotone downward** — there is no operation that amplifies
  authority. A child component therefore cannot exceed the authority its parent
  chose to delegate, *by construction*.
- This is how a trust boundary is drawn in Ken: a supervisor holds broad
  authority and hands each component the minimal attenuated slice it needs — the
  AI-era control "this generated helper must not reach the network beyond
  `api.example.com`" becomes a compile-time fact, not a code-review hope.

## 4. Revocation

Authority must be **revocable** at a boundary (a delegated capability can be
withdrawn):

- A **revocable capability** is mediated by a forwarder/membrane whose validity
  is held in a controlling `space` cell (`../30-surface/36 §4`); revoking flips
  the cell, after which the forwarded capability (and everything attenuated from
  it) fails closed.
- Revocation is **transitive**: revoking a capability revokes the authorities
  derived from it (the membrane bounds the whole sub-delegation).
- The exact mechanism (membrane vs. validity-indexed capability vs. region
  lifetime) ties to the isolation model — **OQ-Space** — which ADR 0004 requires
  to carry a *stated, proven* isolation property, not "deliberate choice, not
  inherited."

## 5. Audit at trust boundaries

Authority exercised across a trust boundary is **auditable**:

- A boundary (a `space` edge, an FFI call, a declassification, a capability
  delegation) can emit a **tamper-evident audit record** — what authority was
  used, by whom, to what effect. Because authority is explicit and effects are
  typed, the audit points are *statically known* (you cannot perform an
  un-audited effect that the type didn't declare).
- **Declassification (`61 §4`) is a capability whose every use is audited here**
  — the answer to "where did PII get released, by what authority, under what
  proven condition" is a query over this log, not a forensic reconstruction.

## 6. Relationship to effects and flow

Authority and flow compose: a capability **gates an effect** (you may write to
`Net` only with `Cap_Net`), and the sink that capability opens **carries a
clearance label** (`61 §3`, you may write only data `⊑` that clearance). So a
single typed arrow expresses both *may this code act* (capability) and *may this
data flow here* (label). Pure-by-default + least-authority + upward-only-flow
together make "an AI-written helper leaks a secret to the network" require
**three** explicit, visible, audited concessions — each a place a reviewer or
policy can say no.

## 7. What is committed vs. open

- **Committed:** no ambient authority; static + visible capabilities; least by
  default; **attenuation** (monotone-weakening); **revocation** (transitive);
  boundary **audit**; capabilities gate effects and carry clearance.
- **Open (`OQ-8a`, `OQ-Space`):** whether a capability is a distinct construct
  or a specific effect; the exact revocation/isolation mechanism. The *security
  requirement* (attenuable, revocable, audited, least) is fixed regardless of
  the construct form.

## 8. What WS-L must deliver here

The capability discipline: no-ambient-authority enforcement; capabilities in the
type (authority manifest); `attenuate` (monotone-weakening); revocation at
boundaries; the boundary audit log; and the effect↔capability↔label composition
(`61`). Acceptance: a function's permitted authority is readable from its type;
a child cannot exceed an attenuated capability; revocation fails the delegate
closed. Conformance: `../../conformance/security/authority/`.
