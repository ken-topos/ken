# WP K-api — Typing judgment + the stable kernel API (the TCB boundary)

> **Status:** Steward frame — **next enclave WP** (the kernel theory K1/K2/
> K2c-s1+s2/K1.5 is now fully spec'd). spec-leader elaborates
> `spec/10-kernel/18-judgments.md` (DRAFT → implementation-ready), then the
> **Kernel team** stabilizes the API in `ken-kernel`.
>
> **Team:** Kernel · **Deps:** full **K2c** (series-1+2 spec'd; the *build* half
> needs `wp/K2c-series2` implemented — pipelines after series-2-build) + **K1.5**
> (admission, in build) · **Size:** M · **Risk:** ★★★ (**the TCB boundary** — the
> contract every other component trusts) · ► Capstone of **WS-K**; feeds **G1**
> and gates V2/Sec3/Sec4.

## Objective

Complete `18-judgments.md` — the chapter that, with `11`–`17`, **closes the
kernel contract (WS-K)**: the full **typing judgment** (the four judgment forms
relative to the global env), the **conversion/"switch" rule** tying typing to
`17`, the **bidirectional `check`/`infer` algorithm**, the kernel's **stable Rust
API surface** (the trusted boundary other components call), the **trusted-base
accounting** (what is in the TCB and why it is small), and the **metatheory
status** (what is proven vs. assumed). The deliverable is the *contract* the
elaborator (V0), interpreter (X1), prover (V-series), and tooling code against —
and the surface a published kernel audit (Sec4) will scrutinize.

## The framing that sets the risk level

This **defines the trusted boundary** — ★★★. Everything outside the kernel is
re-checked by it (the de Bruijn criterion, `docs/PRINCIPLES.md`); the API is the
line between trusted and untrusted. So the load-bearing properties are: the API
is the **minimal sufficient surface** (small auditable TCB — every exported
function is justified; nothing trusted that could be re-checked instead); each
entry point has a **precise pre/post-condition** (what it assumes, what it
guarantees — e.g. `check` returns Ok only for well-typed core terms; `admit`
enforces positivity + SCT + W-style admission); and the **trusted-base accounting
is honest** (the metatheory status states plainly what is proven and what is a
standing assumption). The Architect review is load-bearing.

## Scope

**IN:**
- **The four judgment forms** + the **conversion/switch rule** (typing ↔ `17`
  conversion), stated normatively.
- **The bidirectional algorithm** — `check`/`infer` to the level of rigor V0
  already codes against, reconciled with `18`'s existing draft and the landed
  `ken-kernel` (verify against the code — see Discipline).
- **The stable Rust API surface** — enumerate the trusted entry points
  (`check`/`infer`/`admit`(decl)/`convert`/`whnf`/env ops…) with a **contract per
  entry** (preconditions, guarantees, error modes). This is the *interface freeze*
  other WPs build against; call out what is stable vs. internal.
- **Trusted-base accounting** — what is in the TCB (and its rough size), what is
  deliberately outside (re-checked), and why the boundary sits where it does.
- **Metatheory status** — proven vs. assumed (subject reduction, canonicity,
  decidability of conversion via the SCT gate, consistency), honestly tagged.

**OUT — do not (re)do here:**
- Re-elaborating `11`–`17` or the reduction rules (K1/K2/K2c done) — `18` *cites*
  them, doesn't restate them.
- The **verification** API (obligation generation `22`/V2, prover `23`/V-series)
  — K-api is the *kernel* boundary; the prover rides on top.
- Any **new** kernel feature or relaxation — K-api formalizes the existing
  contract, it does not extend the theory.

## The elaboration this needs (spec-leader → spec-author + Architect)

Elaborate `18` to builder rigor: the judgment + switch rule precise; the
`check`/`infer` algorithm as defensive pseudocode (write it as it computes); the
**API contract table** (entry → pre/post/errors); the TCB accounting + metatheory
status as explicit, honest prose. Conformance (`conformance/kernel/judgments/`):
the API contracts are **testable** — `check` accepts a well-typed core term and
**rejects** each ill-typed class (verdict-flip); `admit` enforces every gate
(positivity, SCT, W-style, respect) — one **invoking** test per gate (the
"invoke every TCB guard" discipline); the switch rule decides at the boundary.
Level-discipline reconcile any judgment that computes a level.

## Acceptance (testable)

1. **Judgment + switch:** the four judgment forms + the conversion/switch rule are
   stated and each has a conformance case that exercises the boundary (a term that
   checks only via a conversion step).
2. **Algorithm:** `check`/`infer` pseudocode is implementable as-is; a corpus of
   well-typed terms checks and each ill-typed class is rejected (verdict-flip).
3. **API contract:** every exported trusted entry has a pre/post/error contract;
   each `admit` gate (positivity, SCT, W-style admission, quotient respect) has at
   least one **invoking** test that flips accept↔reject on the gate condition.
4. **TCB accounting:** the trusted surface is enumerated and justified as
   **minimal** (nothing trusted that the de Bruijn criterion lets us re-check);
   the metatheory status honestly separates proven from assumed.
5. **No regression:** the API as specified matches what V0/X1 already call (the
   contract is a *freeze + completion* of the working surface, not a redesign).

## Sequencing

Next enclave WP (kernel theory fully spec'd). The **spec** elaborates now; the
**build** (stabilizing the API in `ken-kernel`) pipelines after K1.5-build +
K2c-series-2-build land (the API must reflect the complete kernel). Unblocks
**V1 → V2** (the verification spine) and **Sec3/Sec4**. Build/boundary queries →
Architect; behavioral contract → Spec. **Discipline:** reconstruct `18`'s
current state + the API surface from the **landed `ken-kernel` code**, not this
frame's prose or a stale draft (the K2c-s2 perishable-frame lesson) — read the
exported functions, diff against `18`'s draft, flag any divergence as a scope
checkpoint before drafting.
