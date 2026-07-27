---
id: CONF-SEC4-REFL-PAIR
title: "Sec4's C1/C2 refl pair is stale against ADR-0013: the true arm is unreachable and the false arm is green for the wrong reason"
status: draft
owner: spec-enclave
size: S
gate: none
depends_on: []
blocks: []
github: null
origin: "Raised by verify-implementer's SEC4-TCB D2 census hard stop (evt_31jhd3mc169gk, 2026-07-27) at a78a7dae; mechanism verified independently by the Steward at obs.rs:113. Ruled out of SEC4-TCB's scope in evt_ff4m551h40fz and filed here."
---

⛔ **`status: draft` is deliberate.** `SEC4-TCB` is in flight and is bound to
the **re-scoped** pair; this node repairs the seed itself, which is the
enclave's artifact. ⛔ It does not gate `SEC4-TCB`'s close.

## The measurement

`conformance/security/trust-model/seed-trust-model.md`, group **C**
(Authorship-independence, AC3 ★), rows
`false-proposition-certificate-rejected` (C1) and `genuine-proof-accepted`
(C2).

Landed behavior, `crates/ken-kernel/src/obs.rs:113`
(`eq_at_registered_literal`, ADR-0013 Layer 2):

```
Eq ty (IntLit m) (IntLit n)  ⇝  Top     if m == n
                             ⇝  Bottom  if m != n
```

| row | seed operand | seed `expect` | landed behavior |
|---|---|---|---|
| C2 | `refl` at `Id Nat 0 0` | accepts | ⛔ **unreachable** — goal ⇝ `Top`, so `check.rs:434`'s `Term::Eq` arm never fires |
| C1 | `refl` at `Id Nat 0 1` | rejects *"conversion fails, `0 ≢ 1`"* | ⚠ **rejects, but conversion is never reached** — goal ⇝ `Bottom` |

## ⭐⭐ Why C1 is the worse half

C2 fails loudly and gets looked at. **C1 passes.** Its stated mechanism —
conversion failure — is not the mechanism that produces its verdict, and
nothing in the corpus distinguishes the two. ⭐ A rejection control that passes
for any reason is not a control.

## ⭐ The defect is the PAIR, not either row

The seed's own `why` states the invariant: *"the **only** difference is the
proposition's truth."* Against landed behavior **both** arms bypass `Refl`'s
conversion check entirely and differ only in which constant the reducer picks.

⇒ The pair no longer isolates AC3. What it now demonstrates — that the reducer
decides closed literal equalities before the certificate is consulted — is true
and interesting, and is **not** authorship-independence.

## The repair (not prescribed — this node exists to scope it)

⭐ The property is sound and still bindable. `eq_at_registered_literal` returns
**neutral** when either operand is not a literal, so abstract binders keep the
goal unreduced; the landed precedent
`ds6c_intlit_elaborator_emission.rs::refl_still_accepted_on_a_genuinely_abstract_eq_shaped_goal`
proves the accept arm is reachable.

⚠ **But the framings differ and the repair must say so.** `Equal Int x y` is
**unprovable**, not **false**; the seed is framed on the proposition's truth.
⛔ Do not silently re-point the rows and leave the `why` text intact.

⭐ Consider retaining the closed arms as their own rows asserting what they
actually do (`Top`/`Bottom` collapse) — that is real landed behavior worth
pinning, and it is the honest home for the mechanism C1 currently mislabels.

## ⭐⭐ Third instance of one class in a day

**A conformance row whose required operand cannot be constructed is
byte-identical, to any reader, to one not yet built.** See
[[CONF-FMT8-LEVELTOK]] (no Level token kind) and [[SEC1-IFC-R3]] (no production
route to `Disproved`).

⇒ ⭐ **This one adds a new and worse face: the *sibling* row stayed green while
measuring a different mechanism.** The other two are silent-red; this is
silent-green. A sweep keyed only on never-green rows would not have found it.
