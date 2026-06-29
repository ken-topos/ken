# WP V0 — Minimal elaborator (surface → kernel)

> **Status:** Steward frame — **awaiting spec-leader elaboration** (2nd in the
> fan-out serial chain, after K2c). spec-leader elaborates `spec/30-surface/
> 39-elaboration.md` (+ the minimal slice of `31-lexical`/`32-grammar`) to
> implementation-ready rigor, then the **Verify team** builds.
>
> **Team:** Verify · **Deps:** K1 (done) · **Size:** M · **Risk:** ★★ · **Parallel**
> with X1. ► **Feeds G1** (the vertical slice) together with X1. Foundational for
> the whole Verify stream (V0 → V1 → V2 → V3 → V4) and for L-classes / L-fmt.

## Objective

The **minimal elaborator**: parse a small surface program, **elaborate it to
kernel core terms**, and have the K1 kernel **check** them. Just enough surface to
prove the G1 slice end-to-end — *the pipeline*, not the full language.

## The framing that sets the risk level

**The elaborator's output is re-checked by the kernel** (the de Bruijn criterion,
`docs/PRINCIPLES.md`) — V0 is **not in the TCB**. A bug in V0 yields a *rejected
valid program* or *bad diagnostics*, **not unsoundness** (the kernel rejects
ill-typed core terms regardless). So V0 is ★★, not ★★★: build it correct and
ergonomic, but the soundness backstop is the kernel, not V0. (This is exactly why
elaboration lives outside the trusted core.)

## Scope

**IN:** a parser for a **minimal** surface (the subset of `31`/`32` needed for a
trivial typed program — `let`/`λ`/application/a base type or two/a simple
dependent function); **elaboration to kernel terms** per `39` (name resolution →
de Bruijn, implicit-argument handling at the minimal level, the surface→core
mapping); the **surface → elaborate → kernel-check pipeline**; parse + elaborate
tests; the `elaborator` crate (minimal).

**OUT — these are *other teams'* WPs, do not build them here:** the full surface
language (numbers `35`, sum/match `34`, strings/collections `37`, modules `33`,
effects `36`, FFI `38` — **Team Language's** L1–L8); **specification syntax**
(`requires`/`ensures` — **V1**); the **prover** (V2/V3); the mandated formatter
(L-fmt); typeclass coherence (L-classes); rich diagnostics polish (later). V0 is
the *minimum viable* surface→kernel path.

## Acceptance (testable)

1. **A trivial program elaborates and kernel-checks** — e.g. a small dependently-
   typed function parses → elaborates to a core term → `infer`/`check` returns Ok
   (the G1-slice precondition).
2. **Round-trip on the minimal surface:** the supported forms (`let`, `λ`,
   application, the base type(s), a dependent `Π`) each parse + elaborate + check.
3. **Ill-typed surface is rejected** — by the **kernel** on the elaborated term
   (V0 elaborates; the kernel judges) — with the error surfaced, not swallowed.
4. **Names resolve correctly** to de Bruijn indices (no capture, correct scoping)
   — the one place V0 can silently corrupt a *well-typed-looking* term.

## Guardrails

- **Elaborate to the kernel's actual API** (K1's `Term`/`check`/`infer` — on
  `main`); do not reimplement or fork kernel checking in the elaborator. The
  kernel is the judge.
- **Minimal means minimal** — resist absorbing Language-team surface scope; if a
  G1-trivial program needs a form, add just that form, and flag the rest as the
  owning L-WP.
- Name resolution / de Bruijn conversion is the correctness-sensitive core (a
  capture bug yields a wrong-but-checkable term) — exercise it with **nested
  binders + shadowing**, per the invoke-every-guard discipline (COORDINATION §7).

## Logistics

Branch `wp/V0-elaborator` cut from `origin/main`. Verify team (`verify-leader` +
`verify-implementer` [Sonnet, **high** effort — Verify is soundness-adjacent] +
`verify-qa`). `scripts/ken-cargo -p <elaborator-crate>`. Ring: implementer builds
→ QA verifies independently → merge Decision (**Architect** + **Spec** on `/spec`
+`/conformance`) → Integrator → retros. Elaboration-strategy / surface-subset Qs →
Architect; behavioral-contract Qs → Spec. Coordinate the **G1 slice** join with
Team Runtime's X1 when both land.
